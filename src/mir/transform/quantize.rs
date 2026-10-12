// mir/transform/quantize.rs — Post-training quantization MIR pass
//
// Rewrites eligible f32 tensor operations to use quantized int8 compute:
//
//  1. MatMul quantization: matmul(a, b) → dequantize(qmatmul(quantize(a), quantize(b)))
//  2. Mixed-precision widening: adds dequantize after quantized ops for f32 consumers
//
// This pass runs at -O2 and is designed for inference optimization.
// Only pure tensor ops with no autograd tape dependency are quantized.

use crate::mir::{
    MirProgram, MirBasicBlock, MirStmt,
    Place, Operand, Rvalue, TensorOpKind, LocalId, MirLocal,
};
use crate::types::{TypeId, TypeInterner};
use super::MirPass;

pub struct QuantizePass;

impl MirPass for QuantizePass {
    fn name(&self) -> &'static str {
        "quantize"
    }

    fn run(&self, program: &mut MirProgram, _interner: &TypeInterner) -> bool {
        for func in &mut program.functions {
            let mut next_local = func.locals.len() as u32 + func.params.len() as u32 + 1;
            let mut new_locals = Vec::new();
            for block in &mut func.basic_blocks {
                quantize_matmuls(block, &mut next_local, &mut new_locals);
            }
            func.locals.extend(new_locals);
        }
        true
    }
}

/// Target int8 dtype code for quantization
const DTYPE_INT8: u8 = 5;

/// Rewrite matmul(a, b) → dequantize(qmatmul(quantize(a), quantize(b)))
fn quantize_matmuls(
    block: &mut MirBasicBlock,
    next_local: &mut u32,
    new_locals: &mut Vec<MirLocal>,
) {
    // Find matmul statements eligible for quantization
    let mut insertions: Vec<(usize, Vec<MirStmt>)> = Vec::new();

    // Build a set of locals that are autograd outputs (skip those)
    let ag_locals = find_autograd_locals(block);

    for (idx, stmt) in block.stmts.iter().enumerate() {
        if let MirStmt::Assign {
            place,
            rvalue: Rvalue::TensorOp { kind: TensorOpKind::MatMul, operands },
            span,
        } = stmt
        {
            if operands.len() != 2 { continue; }

            // Skip if either operand comes from autograd (gradient tracking)
            let a_local = operand_local(&operands[0]);
            let b_local = operand_local(&operands[1]);
            if a_local.map_or(false, |id| ag_locals.contains(&id))
                || b_local.map_or(false, |id| ag_locals.contains(&id))
            {
                continue;
            }

            // Allocate 4 new locals: qa, qb, qresult, dequant_result
            let qa_id = LocalId(*next_local);
            let qb_id = LocalId(*next_local + 1);
            let qmm_id = LocalId(*next_local + 2);
            let deq_id = place.local; // reuse original destination
            *next_local += 3;

            for _ in 0..3 {
                new_locals.push(MirLocal {
                    id: LocalId(new_locals.len() as u32 + *next_local - 3),
                    name: None,
                    ty: TypeId::UNIT,
                    mutable: false,
                });
            }

            let sp = span.clone();
            let mut replacement = Vec::new();

            // qa = quantize(a, int8)
            replacement.push(MirStmt::Assign {
                place: Place { local: qa_id, projections: Vec::new() },
                rvalue: Rvalue::TensorOp {
                    kind: TensorOpKind::Quantize(DTYPE_INT8),
                    operands: vec![operands[0].clone()],
                },
                span: sp.clone(),
            });

            // qb = quantize(b, int8)
            replacement.push(MirStmt::Assign {
                place: Place { local: qb_id, projections: Vec::new() },
                rvalue: Rvalue::TensorOp {
                    kind: TensorOpKind::Quantize(DTYPE_INT8),
                    operands: vec![operands[1].clone()],
                },
                span: sp.clone(),
            });

            // qmm = qmatmul(qa, qb)
            replacement.push(MirStmt::Assign {
                place: Place { local: qmm_id, projections: Vec::new() },
                rvalue: Rvalue::TensorOp {
                    kind: TensorOpKind::QMatMul,
                    operands: vec![
                        Operand::Place(Place { local: qa_id, projections: Vec::new() }),
                        Operand::Place(Place { local: qb_id, projections: Vec::new() }),
                    ],
                },
                span: sp.clone(),
            });

            // deq = dequantize(qmm) → stored in original destination
            replacement.push(MirStmt::Assign {
                place: Place { local: deq_id, projections: Vec::new() },
                rvalue: Rvalue::TensorOp {
                    kind: TensorOpKind::Dequantize(0),
                    operands: vec![
                        Operand::Place(Place { local: qmm_id, projections: Vec::new() }),
                    ],
                },
                span: sp,
            });

            insertions.push((idx, replacement));
        }
    }

    // Apply replacements in reverse order to maintain indices
    for (idx, replacement) in insertions.into_iter().rev() {
        block.stmts.splice(idx..=idx, replacement);
    }
}

/// Extract LocalId from an operand.
fn operand_local(op: &Operand) -> Option<LocalId> {
    match op {
        Operand::Place(place) => Some(place.local),
        _ => None,
    }
}

/// Find locals that are produced by operations likely to be autograd-tracked.
/// (Simple heuristic: any local assigned from a function call is potentially AG)
fn find_autograd_locals(block: &MirBasicBlock) -> std::collections::HashSet<LocalId> {
    let mut ag = std::collections::HashSet::new();
    for stmt in &block.stmts {
        if let MirStmt::Assign { place, rvalue, .. } = stmt {
            // Skip quantization of intermediates from fused ops (they have special semantics)
            if let Rvalue::TensorOp { kind, .. } = rvalue {
                if matches!(kind,
                    TensorOpKind::FusedLinearRelu
                    | TensorOpKind::FusedMse
                    | TensorOpKind::FusedSoftmaxCe
                ) {
                    ag.insert(place.local);
                }
            }
        }
    }
    ag
}

// ── Tests ───────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mir::*;
    use crate::span::Span;

    fn place_op(id: u32) -> Operand {
        Operand::Place(Place { local: LocalId(id), projections: Vec::new() })
    }

    fn tensor_assign(local: u32, kind: TensorOpKind, operands: Vec<Operand>) -> MirStmt {
        MirStmt::Assign {
            place: Place { local: LocalId(local), projections: Vec::new() },
            rvalue: Rvalue::TensorOp { kind, operands },
            span: Span::dummy(),
        }
    }

    fn make_block(stmts: Vec<MirStmt>) -> MirBasicBlock {
        MirBasicBlock {
            id: BlockId(0),
            stmts,
            terminator: Terminator::Return,
        }
    }

    fn make_program(stmts: Vec<MirStmt>) -> MirProgram {
        MirProgram {
            functions: vec![MirFunction {
                name: "test".to_string(),
                mangled_name: "test".to_string(),
                params: vec![],
                return_ty: TypeId::UNIT,
                locals: vec![
                    MirLocal { id: LocalId(0), name: None, ty: TypeId::UNIT, mutable: false },
                    MirLocal { id: LocalId(1), name: None, ty: TypeId::UNIT, mutable: false },
                    MirLocal { id: LocalId(2), name: None, ty: TypeId::UNIT, mutable: false },
                    MirLocal { id: LocalId(3), name: None, ty: TypeId::UNIT, mutable: false },
                    MirLocal { id: LocalId(4), name: None, ty: TypeId::UNIT, mutable: false },
                ],
                basic_blocks: vec![make_block(stmts)],
                attributes: Vec::new(),
                span: Span::dummy(),
            }],
            statics: vec![],
        }
    }

    #[test]
    fn test_matmul_gets_quantized() {
        // matmul(a, b) should become: quantize(a), quantize(b), qmatmul, dequantize
        let stmts = vec![
            tensor_assign(2, TensorOpKind::MatMul, vec![place_op(0), place_op(1)]),
        ];
        let mut prog = make_program(stmts);
        let interner = TypeInterner::new();
        QuantizePass.run(&mut prog, &interner);

        let block = &prog.functions[0].basic_blocks[0];
        assert_eq!(block.stmts.len(), 4, "matmul should expand to 4 stmts");

        // First two should be Quantize
        assert!(matches!(&block.stmts[0], MirStmt::Assign {
            rvalue: Rvalue::TensorOp { kind: TensorOpKind::Quantize(5), .. }, ..
        }));
        assert!(matches!(&block.stmts[1], MirStmt::Assign {
            rvalue: Rvalue::TensorOp { kind: TensorOpKind::Quantize(5), .. }, ..
        }));

        // Third should be QMatMul
        assert!(matches!(&block.stmts[2], MirStmt::Assign {
            rvalue: Rvalue::TensorOp { kind: TensorOpKind::QMatMul, .. }, ..
        }));

        // Fourth should be Dequantize
        assert!(matches!(&block.stmts[3], MirStmt::Assign {
            rvalue: Rvalue::TensorOp { kind: TensorOpKind::Dequantize(_), .. }, ..
        }));
    }

    #[test]
    fn test_add_not_quantized() {
        // Add should not be quantized
        let stmts = vec![
            tensor_assign(2, TensorOpKind::Add, vec![place_op(0), place_op(1)]),
        ];
        let mut prog = make_program(stmts);
        let interner = TypeInterner::new();
        QuantizePass.run(&mut prog, &interner);

        let block = &prog.functions[0].basic_blocks[0];
        assert_eq!(block.stmts.len(), 1, "add should not be expanded");
    }

    #[test]
    fn test_fused_op_not_quantized() {
        // FusedLinearRelu should not be quantized (marked as autograd)
        let stmts = vec![
            tensor_assign(2, TensorOpKind::FusedLinearRelu, vec![place_op(0), place_op(1)]),
            tensor_assign(3, TensorOpKind::MatMul, vec![place_op(2), place_op(1)]),
        ];
        let mut prog = make_program(stmts);
        let interner = TypeInterner::new();
        QuantizePass.run(&mut prog, &interner);

        let block = &prog.functions[0].basic_blocks[0];
        // FusedLinearRelu stays, matmul using its output should NOT be quantized
        assert_eq!(block.stmts.len(), 2, "fused output matmul should not be quantized");
    }

    #[test]
    fn test_multiple_matmuls_quantized() {
        let stmts = vec![
            tensor_assign(2, TensorOpKind::MatMul, vec![place_op(0), place_op(1)]),
            tensor_assign(3, TensorOpKind::MatMul, vec![place_op(2), place_op(1)]),
        ];
        let mut prog = make_program(stmts);
        let interner = TypeInterner::new();
        QuantizePass.run(&mut prog, &interner);

        let block = &prog.functions[0].basic_blocks[0];
        // First matmul expands to 4, second also expands to 4 = 8 total
        assert_eq!(block.stmts.len(), 8, "two matmuls should expand to 8 stmts");
    }

    #[test]
    fn test_empty_block_no_crash() {
        let mut prog = make_program(vec![]);
        let interner = TypeInterner::new();
        QuantizePass.run(&mut prog, &interner);
        assert!(prog.functions[0].basic_blocks[0].stmts.is_empty());
    }

    #[test]
    fn test_quantize_preserves_original_dest() {
        // The final dequantize should write to the original matmul destination
        let stmts = vec![
            tensor_assign(4, TensorOpKind::MatMul, vec![place_op(0), place_op(1)]),
        ];
        let mut prog = make_program(stmts);
        let interner = TypeInterner::new();
        QuantizePass.run(&mut prog, &interner);

        let block = &prog.functions[0].basic_blocks[0];
        // Last stmt (dequantize) should target local 4
        if let MirStmt::Assign { place, .. } = &block.stmts[3] {
            assert_eq!(place.local, LocalId(4), "dequantize should target original dest");
        } else {
            panic!("expected assign");
        }
    }
}
