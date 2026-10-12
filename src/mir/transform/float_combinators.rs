// mir/transform/float_combinators.rs — Combinator floating pass
//
// Identifies closed tensor subexpressions (tensor operations that only
// depend on their inputs, not on local state) and extracts them to
// top-level functions. This is a prerequisite for fusion and parallelism,
// since extracted combinators are easier to pattern-match and schedule.

use std::collections::HashSet;

use crate::mir::{
    MirProgram, MirFunction, MirBasicBlock, MirStmt, Terminator,
    Place, Operand, Rvalue, LocalId, BlockId, MirLocal,
};
use crate::span::Span;
use crate::types::{TypeId, TypeInterner};
use super::MirPass;

pub struct CombinatorFloating;

impl MirPass for CombinatorFloating {
    fn name(&self) -> &'static str {
        "float_combinators"
    }

    fn run(&self, program: &mut MirProgram, _interner: &TypeInterner) -> bool {
        let mut new_functions: Vec<MirFunction> = Vec::new();
        let mut counter: u32 = 0;

        for func in &mut program.functions {
            let extracted = extract_tensor_combinators(func, &mut counter);
            new_functions.extend(extracted);
        }

        program.functions.extend(new_functions);
        true
    }
}

/// Analysis result for a basic block — which locals are read/written,
/// and whether the block contains only tensor operations.
struct BlockAnalysis {
    /// Locals that are read (used as operands)
    reads: HashSet<LocalId>,
    /// Locals that are written (assigned to)
    writes: HashSet<LocalId>,
    /// Whether all statements in the block are tensor operations
    all_tensor_ops: bool,
    /// Number of tensor ops in this block
    tensor_op_count: usize,
}

fn analyze_block(block: &MirBasicBlock) -> BlockAnalysis {
    let mut reads = HashSet::new();
    let mut writes = HashSet::new();
    let mut all_tensor_ops = true;
    let mut tensor_op_count = 0;

    for stmt in &block.stmts {
        match stmt {
            MirStmt::Assign { place, rvalue, .. } => {
                writes.insert(place.local);
                collect_rvalue_reads(rvalue, &mut reads);

                if matches!(rvalue, Rvalue::TensorOp { .. }) {
                    tensor_op_count += 1;
                } else if !matches!(rvalue, Rvalue::Use(_)) {
                    all_tensor_ops = false;
                }
            }
            MirStmt::Nop => {}
            _ => {
                all_tensor_ops = false;
            }
        }
    }

    // Also check terminator reads
    collect_terminator_reads(&block.terminator, &mut reads);

    BlockAnalysis {
        reads,
        writes,
        all_tensor_ops,
        tensor_op_count,
    }
}

fn collect_operand_reads(op: &Operand, reads: &mut HashSet<LocalId>) {
    if let Operand::Place(place) = op {
        reads.insert(place.local);
    }
}

fn collect_rvalue_reads(rvalue: &Rvalue, reads: &mut HashSet<LocalId>) {
    match rvalue {
        Rvalue::Use(op) => collect_operand_reads(op, reads),
        Rvalue::BinaryOp { left, right, .. } => {
            collect_operand_reads(left, reads);
            collect_operand_reads(right, reads);
        }
        Rvalue::UnaryOp { operand, .. } => {
            collect_operand_reads(operand, reads);
        }
        Rvalue::TensorOp { operands, .. } => {
            for op in operands {
                collect_operand_reads(op, reads);
            }
        }
        Rvalue::Aggregate { fields, .. } => {
            for op in fields {
                collect_operand_reads(op, reads);
            }
        }
        Rvalue::Ref { place, .. } => {
            reads.insert(place.local);
        }
        Rvalue::Cast { operand, .. } => {
            collect_operand_reads(operand, reads);
        }
        Rvalue::Len { place } => {
            reads.insert(place.local);
        }
        Rvalue::Discriminant { place } => {
            reads.insert(place.local);
        }
    }
}

fn collect_terminator_reads(term: &Terminator, reads: &mut HashSet<LocalId>) {
    match term {
        Terminator::SwitchInt { value, .. } => {
            collect_operand_reads(value, reads);
        }
        Terminator::Call { func, args, .. } => {
            collect_operand_reads(func, reads);
            for arg in args {
                collect_operand_reads(arg, reads);
            }
        }
        Terminator::Assert { cond, .. } => {
            collect_operand_reads(cond, reads);
        }
        _ => {}
    }
}

/// Compute the free variables of a block: locals that are read but not
/// written within the same block (i.e., they come from outside).
fn free_variables(analysis: &BlockAnalysis) -> HashSet<LocalId> {
    analysis.reads.difference(&analysis.writes).cloned().collect()
}

/// Extract tensor combinator sequences from a function.
/// Returns a list of newly created functions.
fn extract_tensor_combinators(
    func: &mut MirFunction,
    counter: &mut u32,
) -> Vec<MirFunction> {
    let mut extracted = Vec::new();

    // Look for blocks that contain only tensor operations (2+ ops)
    // and are "closed" (all inputs come from function parameters or are constants)
    let block_count = func.basic_blocks.len();
    let mut to_replace: Vec<(usize, String, Vec<LocalId>)> = Vec::new();

    for i in 0..block_count {
        let analysis = analyze_block(&func.basic_blocks[i]);

        // Only extract if: has >=2 tensor ops, all operations are tensor ops or Use
        if analysis.tensor_op_count >= 2 && analysis.all_tensor_ops {
            let free_vars = free_variables(&analysis);

            // Check that free variables are all function params or constants
            let param_locals: HashSet<LocalId> = func.params.iter().map(|p| p.id).collect();
            let is_closed = free_vars.iter().all(|v| param_locals.contains(v));

            if is_closed && !free_vars.is_empty() {
                let combinator_name = format!("__combinator_{}_{}", func.name, counter);
                *counter += 1;
                to_replace.push((i, combinator_name.clone(), free_vars.iter().cloned().collect()));

                // Create the extracted function
                let mut new_locals = Vec::new();
                let mut local_remap: std::collections::HashMap<LocalId, LocalId> = std::collections::HashMap::new();

                // Return place — LocalId(0)
                let return_local = LocalId(0);
                new_locals.push(MirLocal {
                    id: return_local,
                    name: None,
                    ty: TypeId::UNIT, // will be set from the last write
                    mutable: false,
                });

                // Create params from free variables
                let mut param_list = Vec::new();
                for (j, &fv) in free_vars.iter().enumerate() {
                    let new_id = LocalId((j + 1) as u32);
                    local_remap.insert(fv, new_id);
                    let orig = func.locals.iter().find(|l| l.id == fv);
                    let ty = orig.map(|l| l.ty).unwrap_or(TypeId::UNIT);
                    let local = MirLocal {
                        id: new_id,
                        name: orig.and_then(|l| l.name.clone()),
                        ty,
                        mutable: false,
                    };
                    param_list.push(local.clone());
                    new_locals.push(local);
                }

                // Remap the writes too
                for &w in &analysis.writes {
                    if !local_remap.contains_key(&w) {
                        let new_id = LocalId(new_locals.len() as u32);
                        local_remap.insert(w, new_id);
                        let orig = func.locals.iter().find(|l| l.id == w);
                        let ty = orig.map(|l| l.ty).unwrap_or(TypeId::UNIT);
                        new_locals.push(MirLocal {
                            id: new_id,
                            name: orig.and_then(|l| l.name.clone()),
                            ty,
                            mutable: true,
                        });
                    }
                }

                // Clone and remap the block
                let mut new_block = func.basic_blocks[i].clone();
                new_block.id = BlockId(0);
                remap_block_locals(&mut new_block, &local_remap);
                new_block.terminator = Terminator::Return;

                // Set return type from last assignment
                let return_ty = if let Some(last_write) = analysis.writes.iter().next() {
                    func.locals.iter().find(|l| l.id == *last_write)
                        .map(|l| l.ty).unwrap_or(TypeId::UNIT)
                } else {
                    TypeId::UNIT
                };
                new_locals[0].ty = return_ty;

                let new_func = MirFunction {
                    name: combinator_name,
                    mangled_name: format!("__combinator_{}_{}", func.name, *counter - 1),
                    params: param_list,
                    return_ty,
                    locals: new_locals,
                    basic_blocks: vec![new_block],
                    attributes: Vec::new(),
                    span: Span::dummy(),
                };

                extracted.push(new_func);
            }
        }
    }

    // Replace extracted blocks with calls to the combinator functions
    for (block_idx, combinator_name, free_vars) in to_replace.into_iter().rev() {
        // Compute call target BEFORE mutably borrowing the block
        let call_target = BlockId(func.basic_blocks.len() as u32);

        let block = &mut func.basic_blocks[block_idx];
        let analysis = analyze_block(block);

        // Find the result local (first write)
        let result_local = analysis.writes.iter().next().cloned().unwrap_or(LocalId(0));

        // Build args from free variables
        let args: Vec<Operand> = free_vars.iter().map(|&fv| {
            Operand::Place(Place { local: fv, projections: Vec::new() })
        }).collect();

        // Replace block contents with a single call
        let old_terminator = block.terminator.clone();
        block.stmts.clear();

        block.terminator = Terminator::Call {
            func: Operand::Constant(crate::mir::MirConstant::String(combinator_name)),
            args,
            destination: Place { local: result_local, projections: Vec::new() },
            target: call_target,
        };

        // Add continuation block with old terminator
        func.basic_blocks.push(MirBasicBlock {
            id: call_target,
            stmts: Vec::new(),
            terminator: old_terminator,
        });
    }

    extracted
}

fn remap_block_locals(
    block: &mut MirBasicBlock,
    remap: &std::collections::HashMap<LocalId, LocalId>,
) {
    for stmt in &mut block.stmts {
        if let MirStmt::Assign { place, rvalue, .. } = stmt {
            remap_place(place, remap);
            remap_rvalue(rvalue, remap);
        }
    }
}

fn remap_place(place: &mut Place, remap: &std::collections::HashMap<LocalId, LocalId>) {
    if let Some(&new_id) = remap.get(&place.local) {
        place.local = new_id;
    }
}

fn remap_operand(op: &mut Operand, remap: &std::collections::HashMap<LocalId, LocalId>) {
    if let Operand::Place(place) = op {
        remap_place(place, remap);
    }
}

fn remap_rvalue(rvalue: &mut Rvalue, remap: &std::collections::HashMap<LocalId, LocalId>) {
    match rvalue {
        Rvalue::Use(op) => remap_operand(op, remap),
        Rvalue::BinaryOp { left, right, .. } => {
            remap_operand(left, remap);
            remap_operand(right, remap);
        }
        Rvalue::UnaryOp { operand, .. } => remap_operand(operand, remap),
        Rvalue::TensorOp { operands, .. } => {
            for op in operands {
                remap_operand(op, remap);
            }
        }
        Rvalue::Aggregate { fields, .. } => {
            for op in fields {
                remap_operand(op, remap);
            }
        }
        Rvalue::Ref { place, .. } => remap_place(place, remap),
        Rvalue::Cast { operand, .. } => remap_operand(operand, remap),
        Rvalue::Len { place } => remap_place(place, remap),
        Rvalue::Discriminant { place } => remap_place(place, remap),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mir::*;
    use crate::types::TypeId;

    fn make_tensor_op_block(id: u32, ops: Vec<(LocalId, TensorOpKind, Vec<Operand>)>) -> MirBasicBlock {
        let stmts = ops.into_iter().map(|(dest, kind, operands)| {
            MirStmt::Assign {
                place: Place { local: dest, projections: Vec::new() },
                rvalue: Rvalue::TensorOp { kind, operands },
                span: Span::dummy(),
            }
        }).collect();

        MirBasicBlock {
            id: BlockId(id),
            stmts,
            terminator: Terminator::Return,
        }
    }

    #[test]
    fn test_analyze_block_detects_tensor_ops() {
        let block = make_tensor_op_block(0, vec![
            (LocalId(2), TensorOpKind::MatMul, vec![
                Operand::Place(Place { local: LocalId(0), projections: vec![] }),
                Operand::Place(Place { local: LocalId(1), projections: vec![] }),
            ]),
            (LocalId(3), TensorOpKind::Add, vec![
                Operand::Place(Place { local: LocalId(2), projections: vec![] }),
                Operand::Place(Place { local: LocalId(1), projections: vec![] }),
            ]),
        ]);

        let analysis = analyze_block(&block);
        assert_eq!(analysis.tensor_op_count, 2);
        assert!(analysis.all_tensor_ops);
        assert!(analysis.reads.contains(&LocalId(0)));
        assert!(analysis.reads.contains(&LocalId(1)));
        assert!(analysis.writes.contains(&LocalId(2)));
        assert!(analysis.writes.contains(&LocalId(3)));
    }

    #[test]
    fn test_free_variables() {
        let block = make_tensor_op_block(0, vec![
            (LocalId(2), TensorOpKind::MatMul, vec![
                Operand::Place(Place { local: LocalId(0), projections: vec![] }),
                Operand::Place(Place { local: LocalId(1), projections: vec![] }),
            ]),
        ]);

        let analysis = analyze_block(&block);
        let fv = free_variables(&analysis);
        assert!(fv.contains(&LocalId(0)));
        assert!(fv.contains(&LocalId(1)));
        assert!(!fv.contains(&LocalId(2)));
    }

    #[test]
    fn test_non_tensor_ops_not_extracted() {
        let block = MirBasicBlock {
            id: BlockId(0),
            stmts: vec![
                MirStmt::Assign {
                    place: Place { local: LocalId(2), projections: vec![] },
                    rvalue: Rvalue::BinaryOp {
                        op: MirBinOp::Add,
                        left: Operand::Constant(MirConstant::Int(1)),
                        right: Operand::Constant(MirConstant::Int(2)),
                    },
                    span: Span::dummy(),
                },
            ],
            terminator: Terminator::Return,
        };

        let analysis = analyze_block(&block);
        assert_eq!(analysis.tensor_op_count, 0);
        assert!(!analysis.all_tensor_ops);
    }

    #[test]
    fn test_extract_creates_new_function() {
        let mut func = MirFunction {
            name: "test".to_string(),
            mangled_name: "test".to_string(),
            params: vec![
                MirLocal { id: LocalId(0), name: None, ty: TypeId::UNIT, mutable: false },
                MirLocal { id: LocalId(1), name: None, ty: TypeId::UNIT, mutable: false },
            ],
            return_ty: TypeId::UNIT,
            locals: vec![
                MirLocal { id: LocalId(0), name: None, ty: TypeId::UNIT, mutable: false },
                MirLocal { id: LocalId(1), name: None, ty: TypeId::UNIT, mutable: false },
                MirLocal { id: LocalId(2), name: None, ty: TypeId::UNIT, mutable: true },
                MirLocal { id: LocalId(3), name: None, ty: TypeId::UNIT, mutable: true },
            ],
            basic_blocks: vec![
                make_tensor_op_block(0, vec![
                    (LocalId(2), TensorOpKind::MatMul, vec![
                        Operand::Place(Place { local: LocalId(0), projections: vec![] }),
                        Operand::Place(Place { local: LocalId(1), projections: vec![] }),
                    ]),
                    (LocalId(3), TensorOpKind::Add, vec![
                        Operand::Place(Place { local: LocalId(2), projections: vec![] }),
                        Operand::Place(Place { local: LocalId(1), projections: vec![] }),
                    ]),
                ]),
            ],
            attributes: Vec::new(),
            span: Span::dummy(),
        };

        let mut counter = 0;
        let extracted = extract_tensor_combinators(&mut func, &mut counter);
        assert!(!extracted.is_empty(), "Should extract at least one combinator");
        assert!(extracted[0].name.starts_with("__combinator_"));
    }

    #[test]
    fn test_single_tensor_op_not_extracted() {
        let mut func = MirFunction {
            name: "test".to_string(),
            mangled_name: "test".to_string(),
            params: vec![
                MirLocal { id: LocalId(0), name: None, ty: TypeId::UNIT, mutable: false },
            ],
            return_ty: TypeId::UNIT,
            locals: vec![
                MirLocal { id: LocalId(0), name: None, ty: TypeId::UNIT, mutable: false },
                MirLocal { id: LocalId(1), name: None, ty: TypeId::UNIT, mutable: true },
            ],
            basic_blocks: vec![
                make_tensor_op_block(0, vec![
                    (LocalId(1), TensorOpKind::MatMul, vec![
                        Operand::Place(Place { local: LocalId(0), projections: vec![] }),
                        Operand::Place(Place { local: LocalId(0), projections: vec![] }),
                    ]),
                ]),
            ],
            attributes: Vec::new(),
            span: Span::dummy(),
        };

        let mut counter = 0;
        let extracted = extract_tensor_combinators(&mut func, &mut counter);
        assert!(extracted.is_empty(), "Single tensor op should not be extracted");
    }

    #[test]
    fn test_block_with_non_param_free_vars_not_extracted() {
        // Block reads from a local that isn't a param — not closed
        let mut func = MirFunction {
            name: "test".to_string(),
            mangled_name: "test".to_string(),
            params: vec![
                MirLocal { id: LocalId(0), name: None, ty: TypeId::UNIT, mutable: false },
            ],
            return_ty: TypeId::UNIT,
            locals: vec![
                MirLocal { id: LocalId(0), name: None, ty: TypeId::UNIT, mutable: false },
                MirLocal { id: LocalId(1), name: None, ty: TypeId::UNIT, mutable: false }, // not a param
                MirLocal { id: LocalId(2), name: None, ty: TypeId::UNIT, mutable: true },
                MirLocal { id: LocalId(3), name: None, ty: TypeId::UNIT, mutable: true },
            ],
            basic_blocks: vec![
                make_tensor_op_block(0, vec![
                    (LocalId(2), TensorOpKind::MatMul, vec![
                        Operand::Place(Place { local: LocalId(0), projections: vec![] }),
                        Operand::Place(Place { local: LocalId(1), projections: vec![] }),
                    ]),
                    (LocalId(3), TensorOpKind::Add, vec![
                        Operand::Place(Place { local: LocalId(2), projections: vec![] }),
                        Operand::Place(Place { local: LocalId(1), projections: vec![] }),
                    ]),
                ]),
            ],
            attributes: Vec::new(),
            span: Span::dummy(),
        };

        let mut counter = 0;
        let extracted = extract_tensor_combinators(&mut func, &mut counter);
        assert!(extracted.is_empty(), "Non-closed block should not be extracted");
    }

    #[test]
    fn test_combinator_preserves_semantics() {
        // After extraction, the original function should have a Call to the combinator
        let mut func = MirFunction {
            name: "test".to_string(),
            mangled_name: "test".to_string(),
            params: vec![
                MirLocal { id: LocalId(0), name: None, ty: TypeId::UNIT, mutable: false },
                MirLocal { id: LocalId(1), name: None, ty: TypeId::UNIT, mutable: false },
            ],
            return_ty: TypeId::UNIT,
            locals: vec![
                MirLocal { id: LocalId(0), name: None, ty: TypeId::UNIT, mutable: false },
                MirLocal { id: LocalId(1), name: None, ty: TypeId::UNIT, mutable: false },
                MirLocal { id: LocalId(2), name: None, ty: TypeId::UNIT, mutable: true },
                MirLocal { id: LocalId(3), name: None, ty: TypeId::UNIT, mutable: true },
            ],
            basic_blocks: vec![
                make_tensor_op_block(0, vec![
                    (LocalId(2), TensorOpKind::MatMul, vec![
                        Operand::Place(Place { local: LocalId(0), projections: vec![] }),
                        Operand::Place(Place { local: LocalId(1), projections: vec![] }),
                    ]),
                    (LocalId(3), TensorOpKind::Add, vec![
                        Operand::Place(Place { local: LocalId(2), projections: vec![] }),
                        Operand::Place(Place { local: LocalId(1), projections: vec![] }),
                    ]),
                ]),
            ],
            attributes: Vec::new(),
            span: Span::dummy(),
        };

        let mut counter = 0;
        let extracted = extract_tensor_combinators(&mut func, &mut counter);

        // Original function should now have a Call terminator
        let has_call = func.basic_blocks.iter().any(|b| {
            matches!(&b.terminator, Terminator::Call { func: Operand::Constant(MirConstant::String(name)), .. } if name.starts_with("__combinator_"))
        });
        assert!(has_call, "Original function should call the extracted combinator");
        assert_eq!(extracted.len(), 1, "Should extract exactly one combinator");
    }
}
