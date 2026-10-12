// mir/transform/inline.rs — Function inlining pass
//
// Inlines trivial functions:
// - Single basic block (no control flow)
// - No recursive calls
// - Small body (fewer than a configurable threshold of statements)
// Also inlines single-use extracted combinators from float_combinators.

use std::collections::HashMap;

use crate::mir::{
    MirProgram, MirBasicBlock, MirStmt, Terminator,
    Place, Operand, Rvalue, MirConstant, LocalId, BlockId, MirLocal,
};
use crate::span::Span;
use crate::types::TypeInterner;
use super::MirPass;

/// Maximum number of statements for a function to be considered trivial.
const INLINE_THRESHOLD: usize = 10;

pub struct InlineTrivial;

impl MirPass for InlineTrivial {
    fn name(&self) -> &'static str {
        "inline"
    }

    fn run(&self, program: &mut MirProgram, _interner: &TypeInterner) -> bool {
        // Phase 1: Identify inlinable functions
        let inline_candidates = find_inline_candidates(program);

        // Phase 2: Inline calls to those functions
        if !inline_candidates.is_empty() {
            inline_calls(program, &inline_candidates);
        }
        true
    }
}

/// A function that's a candidate for inlining.
#[derive(Clone)]
struct InlineCandidate {
    block: MirBasicBlock,
    params: Vec<MirLocal>,
    locals: Vec<MirLocal>,
}

/// Find functions suitable for inlining.
fn find_inline_candidates(program: &MirProgram) -> HashMap<String, InlineCandidate> {
    let mut candidates = HashMap::new();

    // Count call sites for each function
    let mut call_counts: HashMap<String, usize> = HashMap::new();
    for func in &program.functions {
        for block in &func.basic_blocks {
            if let Terminator::Call { func: Operand::Constant(MirConstant::String(name)), .. } = &block.terminator {
                *call_counts.entry(name.clone()).or_insert(0) += 1;
            }
        }
    }

    for func in &program.functions {
        // Don't inline main
        if func.name == "main" {
            continue;
        }

        // Must have exactly one basic block
        if func.basic_blocks.len() != 1 {
            continue;
        }

        let block = &func.basic_blocks[0];

        // Must not exceed statement threshold
        if block.stmts.len() > INLINE_THRESHOLD {
            continue;
        }

        // Must not be recursive (no calls to self)
        let is_recursive = match &block.terminator {
            Terminator::Call { func: Operand::Constant(MirConstant::String(name)), .. } => {
                *name == func.name
            }
            _ => false,
        };
        if is_recursive {
            continue;
        }

        // Must terminate with Return (not a call to another function in its only block)
        if !matches!(block.terminator, Terminator::Return) {
            continue;
        }

        // Prefer inlining: extracted combinators (always), or single-use functions
        let is_combinator = func.name.starts_with("__combinator_");
        let is_single_use = call_counts.get(&func.name).copied().unwrap_or(0) <= 1;

        if is_combinator || is_single_use || block.stmts.len() <= 3 {
            candidates.insert(func.name.clone(), InlineCandidate {
                block: block.clone(),
                params: func.params.clone(),
                locals: func.locals.clone(),
            });
        }
    }

    candidates
}

/// Inline calls to candidate functions.
fn inline_calls(program: &mut MirProgram, candidates: &HashMap<String, InlineCandidate>) {
    for func in &mut program.functions {
        let mut new_blocks: Vec<MirBasicBlock> = Vec::new();
        let mut blocks_to_process: Vec<MirBasicBlock> = func.basic_blocks.drain(..).collect();

        for block in blocks_to_process.drain(..) {
            if let Terminator::Call {
                func: Operand::Constant(MirConstant::String(ref callee_name)),
                ref args,
                ref destination,
                target,
            } = block.terminator.clone()
            {
                if let Some(candidate) = candidates.get(callee_name) {
                    // Inline this call
                    let inlined = inline_single_call(
                        &mut func.locals,
                        &block,
                        candidate,
                        args,
                        destination,
                        target,
                    );
                    new_blocks.extend(inlined);
                    continue;
                }
            }
            new_blocks.push(block);
        }

        func.basic_blocks = new_blocks;
    }

    // Remove inlined functions from the program
    let candidate_names: std::collections::HashSet<&String> = candidates.keys().collect();
    program.functions.retain(|f| !candidate_names.contains(&f.name));
}

/// Inline a single call site.
fn inline_single_call(
    caller_locals: &mut Vec<MirLocal>,
    call_block: &MirBasicBlock,
    candidate: &InlineCandidate,
    args: &[Operand],
    destination: &Place,
    continuation: BlockId,
) -> Vec<MirBasicBlock> {
    // Create remapping from callee locals to new caller locals
    let mut local_remap: HashMap<LocalId, LocalId> = HashMap::new();

    // Map callee's return place (LocalId(0)) to the call destination
    local_remap.insert(LocalId(0), destination.local);

    // Map parameters to the argument locals
    for (i, param) in candidate.params.iter().enumerate() {
        if i < args.len() {
            if let Operand::Place(place) = &args[i] {
                local_remap.insert(param.id, place.local);
            } else {
                // Constant argument — create a temp local and assign
                let new_id = LocalId(caller_locals.len() as u32);
                caller_locals.push(MirLocal {
                    id: new_id,
                    name: param.name.clone(),
                    ty: param.ty,
                    mutable: false,
                });
                local_remap.insert(param.id, new_id);
            }
        }
    }

    // Map remaining callee locals to new caller locals
    for local in &candidate.locals {
        if !local_remap.contains_key(&local.id) {
            let new_id = LocalId(caller_locals.len() as u32);
            caller_locals.push(MirLocal {
                id: new_id,
                name: local.name.clone(),
                ty: local.ty,
                mutable: local.mutable,
            });
            local_remap.insert(local.id, new_id);
        }
    }

    // Build the inlined block: original call_block stmts + remapped callee stmts
    let mut inlined_stmts = call_block.stmts.clone();

    // Add constant argument assignments
    for (i, param) in candidate.params.iter().enumerate() {
        if i < args.len() {
            if let Operand::Constant(c) = &args[i] {
                let target_local = local_remap[&param.id];
                inlined_stmts.push(MirStmt::Assign {
                    place: Place { local: target_local, projections: Vec::new() },
                    rvalue: Rvalue::Use(Operand::Constant(c.clone())),
                    span: Span::dummy(),
                });
            }
        }
    }

    // Add remapped callee statements
    for stmt in &candidate.block.stmts {
        let mut new_stmt = stmt.clone();
        remap_stmt_locals(&mut new_stmt, &local_remap);
        inlined_stmts.push(new_stmt);
    }

    vec![MirBasicBlock {
        id: call_block.id,
        stmts: inlined_stmts,
        terminator: Terminator::Goto { target: continuation },
    }]
}

fn remap_stmt_locals(stmt: &mut MirStmt, remap: &HashMap<LocalId, LocalId>) {
    match stmt {
        MirStmt::Assign { place, rvalue, .. } => {
            remap_place(place, remap);
            remap_rvalue(rvalue, remap);
        }
        MirStmt::Drop { place, .. } => {
            remap_place(place, remap);
        }
        MirStmt::StorageLive { local } | MirStmt::StorageDead { local } => {
            if let Some(&new_id) = remap.get(local) {
                *local = new_id;
            }
        }
        MirStmt::Nop | MirStmt::ParallelBegin { .. } | MirStmt::ParallelEnd => {}
    }
}

fn remap_place(place: &mut Place, remap: &HashMap<LocalId, LocalId>) {
    if let Some(&new_id) = remap.get(&place.local) {
        place.local = new_id;
    }
}

fn remap_operand(op: &mut Operand, remap: &HashMap<LocalId, LocalId>) {
    if let Operand::Place(place) = op {
        remap_place(place, remap);
    }
}

fn remap_rvalue(rvalue: &mut Rvalue, remap: &HashMap<LocalId, LocalId>) {
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

    fn make_trivial_function(name: &str, stmts: Vec<MirStmt>) -> MirFunction {
        MirFunction {
            name: name.to_string(),
            mangled_name: name.to_string(),
            params: vec![],
            return_ty: TypeId::UNIT,
            locals: vec![MirLocal { id: LocalId(0), name: None, ty: TypeId::UNIT, mutable: false }],
            basic_blocks: vec![MirBasicBlock {
                id: BlockId(0),
                stmts,
                terminator: Terminator::Return,
            }],
            attributes: Vec::new(),
            span: Span::dummy(),
        }
    }

    #[test]
    fn test_trivial_function_is_candidate() {
        let program = MirProgram {
            functions: vec![
                make_trivial_function("helper", vec![MirStmt::Nop]),
                MirFunction {
                    name: "main".to_string(),
                    mangled_name: "main".to_string(),
                    params: vec![],
                    return_ty: TypeId::UNIT,
                    locals: vec![MirLocal { id: LocalId(0), name: None, ty: TypeId::UNIT, mutable: false }],
                    basic_blocks: vec![MirBasicBlock {
                        id: BlockId(0),
                        stmts: vec![],
                        terminator: Terminator::Call {
                            func: Operand::Constant(MirConstant::String("helper".to_string())),
                            args: vec![],
                            destination: Place { local: LocalId(0), projections: vec![] },
                            target: BlockId(1),
                        },
                    }, MirBasicBlock {
                        id: BlockId(1),
                        stmts: vec![],
                        terminator: Terminator::Return,
                    }],
                    attributes: Vec::new(),
                    span: Span::dummy(),
                },
            ],
            statics: vec![],
        };

        let candidates = find_inline_candidates(&program);
        assert!(candidates.contains_key("helper"), "Trivial function should be inlinable");
    }

    #[test]
    fn test_main_not_inlined() {
        let program = MirProgram {
            functions: vec![
                make_trivial_function("main", vec![MirStmt::Nop]),
            ],
            statics: vec![],
        };

        let candidates = find_inline_candidates(&program);
        assert!(!candidates.contains_key("main"), "main should never be inlined");
    }

    #[test]
    fn test_multi_block_not_inlined() {
        let program = MirProgram {
            functions: vec![
                MirFunction {
                    name: "complex".to_string(),
                    mangled_name: "complex".to_string(),
                    params: vec![],
                    return_ty: TypeId::UNIT,
                    locals: vec![MirLocal { id: LocalId(0), name: None, ty: TypeId::UNIT, mutable: false }],
                    basic_blocks: vec![
                        MirBasicBlock {
                            id: BlockId(0),
                            stmts: vec![],
                            terminator: Terminator::Goto { target: BlockId(1) },
                        },
                        MirBasicBlock {
                            id: BlockId(1),
                            stmts: vec![],
                            terminator: Terminator::Return,
                        },
                    ],
                    attributes: Vec::new(),
                    span: Span::dummy(),
                },
            ],
            statics: vec![],
        };

        let candidates = find_inline_candidates(&program);
        assert!(!candidates.contains_key("complex"), "Multi-block function should not be inlined");
    }

    #[test]
    fn test_combinator_always_inlined() {
        let program = MirProgram {
            functions: vec![
                make_trivial_function("__combinator_test_0", vec![MirStmt::Nop, MirStmt::Nop]),
            ],
            statics: vec![],
        };

        let candidates = find_inline_candidates(&program);
        assert!(candidates.contains_key("__combinator_test_0"),
            "Extracted combinators should always be inlinable");
    }

    #[test]
    fn test_inline_removes_function() {
        let mut program = MirProgram {
            functions: vec![
                make_trivial_function("helper", vec![MirStmt::Nop]),
                MirFunction {
                    name: "main".to_string(),
                    mangled_name: "main".to_string(),
                    params: vec![],
                    return_ty: TypeId::UNIT,
                    locals: vec![
                        MirLocal { id: LocalId(0), name: None, ty: TypeId::UNIT, mutable: false },
                    ],
                    basic_blocks: vec![
                        MirBasicBlock {
                            id: BlockId(0),
                            stmts: vec![],
                            terminator: Terminator::Call {
                                func: Operand::Constant(MirConstant::String("helper".to_string())),
                                args: vec![],
                                destination: Place { local: LocalId(0), projections: vec![] },
                                target: BlockId(1),
                            },
                        },
                        MirBasicBlock {
                            id: BlockId(1),
                            stmts: vec![],
                            terminator: Terminator::Return,
                        },
                    ],
                    attributes: Vec::new(),
                    span: Span::dummy(),
                },
            ],
            statics: vec![],
        };

        let candidates = find_inline_candidates(&program);
        inline_calls(&mut program, &candidates);

        // helper should be removed after inlining
        assert!(!program.functions.iter().any(|f| f.name == "helper"),
            "Inlined function should be removed");
        assert!(program.functions.iter().any(|f| f.name == "main"),
            "main should still exist");
    }

    #[test]
    fn test_large_function_not_inlined() {
        let stmts: Vec<MirStmt> = (0..INLINE_THRESHOLD + 1).map(|_| MirStmt::Nop).collect();
        let program = MirProgram {
            functions: vec![
                make_trivial_function("big", stmts),
            ],
            statics: vec![],
        };

        let candidates = find_inline_candidates(&program);
        assert!(!candidates.contains_key("big"),
            "Large function should not be inlined");
    }

    #[test]
    fn test_empty_program_no_panic() {
        let mut program = MirProgram { functions: vec![], statics: vec![] };
        let candidates = find_inline_candidates(&program);
        inline_calls(&mut program, &candidates);
        assert_eq!(program.functions.len(), 0);
    }
}
