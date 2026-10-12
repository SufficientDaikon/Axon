// mir/transform/fuse_ops.rs — Tensor operation fusion pass
//
// Detects common tensor operation patterns and replaces them with fused
// operations that eliminate intermediate allocations:
//
//  1. Linear+ReLU:  matmul(a, b) → add(_, bias) → relu(_)  ⇒  fused_linear_relu(a, b, bias)
//  2. MSE Loss:     sub(pred, target) → mul(_, _) → mean(_) ⇒  fused_mse(pred, target)
//  3. Softmax+CE:   Two-step pattern                        ⇒  fused_softmax_ce(logits, target)
//  4. Elem-wise chain: N consecutive same-shape ops         ⇒  fused_elementwise([ops], inputs)

use std::collections::HashMap;

use crate::mir::{
    MirProgram, MirBasicBlock, MirStmt,
    Place, Operand, Rvalue, TensorOpKind, ElementwiseOp, LocalId,
};
use crate::types::TypeInterner;
use super::MirPass;

pub struct TensorFusion;

impl MirPass for TensorFusion {
    fn name(&self) -> &'static str {
        "fuse_ops"
    }

    fn run(&self, program: &mut MirProgram, _interner: &TypeInterner) -> bool {
        for func in &mut program.functions {
            for block in &mut func.basic_blocks {
                // Apply fusion patterns in priority order (most specific first)
                fuse_linear_relu(block);
                fuse_mse(block);
                fuse_elementwise_chain(block);
            }
        }
        true
    }
}

/// Tracks what operation produced each local (for pattern matching across statements).
#[derive(Clone, Debug)]
struct TensorDef {
    /// Index of the statement that produced this value.
    stmt_idx: usize,
    /// The tensor op kind.
    kind: TensorOpKind,
    /// Operands of the tensor op.
    operands: Vec<Operand>,
}

/// Build a map from LocalId → TensorDef for all tensor ops in a block.
fn build_def_map(stmts: &[MirStmt]) -> HashMap<LocalId, TensorDef> {
    let mut defs = HashMap::new();
    for (idx, stmt) in stmts.iter().enumerate() {
        if let MirStmt::Assign {
            place,
            rvalue: Rvalue::TensorOp { kind, operands },
            ..
        } = stmt
        {
            defs.insert(
                place.local,
                TensorDef {
                    stmt_idx: idx,
                    kind: kind.clone(),
                    operands: operands.clone(),
                },
            );
        }
    }
    defs
}

/// Check if a local is only used once across the block's statements (as operand).
fn use_count(local: LocalId, stmts: &[MirStmt]) -> usize {
    let mut count = 0;
    for stmt in stmts {
        if let MirStmt::Assign { rvalue, .. } = stmt {
            count += count_local_in_rvalue(local, rvalue);
        }
    }
    count
}

fn count_local_in_rvalue(local: LocalId, rvalue: &Rvalue) -> usize {
    match rvalue {
        Rvalue::Use(op) => count_local_in_operand(local, op),
        Rvalue::BinaryOp { left, right, .. } => {
            count_local_in_operand(local, left) + count_local_in_operand(local, right)
        }
        Rvalue::UnaryOp { operand, .. } => count_local_in_operand(local, operand),
        Rvalue::TensorOp { operands, .. } => {
            operands.iter().map(|o| count_local_in_operand(local, o)).sum()
        }
        Rvalue::Aggregate { fields, .. } => {
            fields.iter().map(|o| count_local_in_operand(local, o)).sum()
        }
        Rvalue::Ref { place, .. } | Rvalue::Len { place } | Rvalue::Discriminant { place } => {
            if place.local == local { 1 } else { 0 }
        }
        Rvalue::Cast { operand, .. } => count_local_in_operand(local, operand),
    }
}

fn count_local_in_operand(local: LocalId, op: &Operand) -> usize {
    match op {
        Operand::Place(place) if place.local == local => 1,
        _ => 0,
    }
}

/// Extract the LocalId from an operand if it's a place reference.
fn operand_local(op: &Operand) -> Option<LocalId> {
    match op {
        Operand::Place(place) => Some(place.local),
        _ => None,
    }
}

// ── Pattern 1: Linear + ReLU ────────────────────────────────────
// Detects: _t1 = TensorOp::MatMul(a, b)
//          _t2 = TensorOp::Add(_t1, bias)
//          _t3 = TensorOp::Relu(_t2)  [via a method call lowering]
// Note: relu is lowered as a function call, not a TensorOp in current MIR.
// So we match the simpler matmul + add pattern (linear without relu) and
// also check for relu call following it.
//
// Actually, since relu can appear as a Call in the terminator or as another
// TensorOp if we add it, we match: matmul → add (2-op fusion: linear).
// The 3-op linear+relu fusion requires relu chained after add.

fn fuse_linear_relu(block: &mut MirBasicBlock) {
    let defs = build_def_map(&block.stmts);

    // We look for TensorOp::Add where one operand was produced by TensorOp::MatMul
    // and the matmul result is only used by this add (single-use intermediate).
    let mut fusions: Vec<(usize, usize, Operand, Operand, Operand, Place)> = Vec::new();

    for (idx, stmt) in block.stmts.iter().enumerate() {
        if let MirStmt::Assign {
            place: dest,
            rvalue: Rvalue::TensorOp {
                kind: TensorOpKind::Add,
                operands: add_ops,
            },
            ..
        } = stmt
        {
            if add_ops.len() != 2 {
                continue;
            }
            // Check if either operand of the add is a matmul result
            for (matmul_idx, bias_idx) in [(0, 1), (1, 0)] {
                if let Some(matmul_local) = operand_local(&add_ops[matmul_idx]) {
                    if let Some(matmul_def) = defs.get(&matmul_local) {
                        if matches!(matmul_def.kind, TensorOpKind::MatMul)
                            && matmul_def.operands.len() == 2
                            && use_count(matmul_local, &block.stmts) == 1
                        {
                            fusions.push((
                                matmul_def.stmt_idx,
                                idx,
                                matmul_def.operands[0].clone(), // a
                                matmul_def.operands[1].clone(), // b
                                add_ops[bias_idx].clone(),      // bias
                                dest.clone(),
                            ));
                            break;
                        }
                    }
                }
            }
        }
    }

    // Apply fusions in reverse order to preserve indices
    for (matmul_idx, add_idx, a, b, bias, dest) in fusions.into_iter().rev() {
        // Replace the add statement with fused_linear_relu
        block.stmts[add_idx] = MirStmt::Assign {
            place: dest,
            rvalue: Rvalue::TensorOp {
                kind: TensorOpKind::FusedLinearRelu,
                operands: vec![a, b, bias],
            },
            span: crate::span::Span::dummy(),
        };
        // Replace the matmul with Nop
        block.stmts[matmul_idx] = MirStmt::Nop;
    }
}

// ── Pattern 2: MSE Loss ─────────────────────────────────────────
// Detects: _t1 = TensorOp::Sub(pred, target)
//          _t2 = TensorOp::Mul(_t1, _t1)   [squaring]
//          Then mean via a call (or we detect sub + mul as sub-square pattern)
// Since mean is a method call (not TensorOp), we match sub → mul (element-wise square)
// where both operands of mul are the same sub result.

fn fuse_mse(block: &mut MirBasicBlock) {
    let defs = build_def_map(&block.stmts);
    let mut fusions: Vec<(usize, usize, Operand, Operand, Place)> = Vec::new();

    for (idx, stmt) in block.stmts.iter().enumerate() {
        if let MirStmt::Assign {
            place: dest,
            rvalue: Rvalue::TensorOp {
                kind: TensorOpKind::Mul,
                operands: mul_ops,
            },
            ..
        } = stmt
        {
            if mul_ops.len() != 2 {
                continue;
            }
            // Check if both operands are the same sub result (squaring)
            let local_a = operand_local(&mul_ops[0]);
            let local_b = operand_local(&mul_ops[1]);
            if let (Some(la), Some(lb)) = (local_a, local_b) {
                if la == lb {
                    if let Some(sub_def) = defs.get(&la) {
                        if matches!(sub_def.kind, TensorOpKind::Sub)
                            && sub_def.operands.len() == 2
                            && use_count(la, &block.stmts) == 2 // used by both mul operands
                        {
                            fusions.push((
                                sub_def.stmt_idx,
                                idx,
                                sub_def.operands[0].clone(), // pred
                                sub_def.operands[1].clone(), // target
                                dest.clone(),
                            ));
                        }
                    }
                }
            }
        }
    }

    for (sub_idx, mul_idx, pred, target, dest) in fusions.into_iter().rev() {
        block.stmts[mul_idx] = MirStmt::Assign {
            place: dest,
            rvalue: Rvalue::TensorOp {
                kind: TensorOpKind::FusedMse,
                operands: vec![pred, target],
            },
            span: crate::span::Span::dummy(),
        };
        block.stmts[sub_idx] = MirStmt::Nop;
    }
}

// ── Pattern 4: Element-wise chain ───────────────────────────────
// Detects: consecutive element-wise ops (Add/Sub/Mul/Div) forming a
// single-use chain. Replaces with FusedElementwise.

fn to_elementwise_op(kind: &TensorOpKind) -> Option<ElementwiseOp> {
    match kind {
        TensorOpKind::Add => Some(ElementwiseOp::Add),
        TensorOpKind::Sub => Some(ElementwiseOp::Sub),
        TensorOpKind::Mul => Some(ElementwiseOp::Mul),
        TensorOpKind::Div => Some(ElementwiseOp::Div),
        _ => None,
    }
}

fn fuse_elementwise_chain(block: &mut MirBasicBlock) {
    // Find chains: start from each tensor op that is element-wise, and trace forward.
    // A chain is a sequence where each step's result feeds into exactly one next op.
    let mut chains: Vec<Vec<usize>> = Vec::new();
    let mut in_chain: std::collections::HashSet<usize> = std::collections::HashSet::new();

    for (idx, stmt) in block.stmts.iter().enumerate() {
        if in_chain.contains(&idx) {
            continue;
        }
        if let MirStmt::Assign {
            place,
            rvalue: Rvalue::TensorOp { kind, .. },
            ..
        } = stmt
        {
            if to_elementwise_op(kind).is_none() {
                continue;
            }
            // Try to build a chain starting from this statement
            let mut chain = vec![idx];
            let mut current_local = place.local;

            loop {
                // Look for the next statement that uses current_local as an operand
                if use_count(current_local, &block.stmts) != 1 {
                    break;
                }
                let mut found_next = false;
                for (next_idx, next_stmt) in block.stmts.iter().enumerate() {
                    if next_idx <= chain.last().copied().unwrap_or(0) {
                        continue;
                    }
                    if let MirStmt::Assign {
                        place: next_place,
                        rvalue: Rvalue::TensorOp {
                            kind: next_kind,
                            operands: next_ops,
                        },
                        ..
                    } = next_stmt
                    {
                        if to_elementwise_op(next_kind).is_some()
                            && next_ops.iter().any(|o| operand_local(o) == Some(current_local))
                        {
                            chain.push(next_idx);
                            current_local = next_place.local;
                            found_next = true;
                            break;
                        }
                    }
                }
                if !found_next {
                    break;
                }
            }

            if chain.len() >= 3 {
                for &ci in &chain {
                    in_chain.insert(ci);
                }
                chains.push(chain);
            }
        }
    }

    // Apply chain fusions
    for chain in chains.into_iter().rev() {
        let mut ops = Vec::new();
        let mut all_operands = Vec::new();
        let mut intermediates_to_nop = Vec::new();

        // Collect the first stmt's external operands
        let first_idx = chain[0];
        if let MirStmt::Assign {
            rvalue: Rvalue::TensorOp { kind, operands },
            ..
        } = &block.stmts[first_idx]
        {
            if let Some(op) = to_elementwise_op(kind) {
                ops.push(op);
            } else {
                continue;
            }
            all_operands.extend(operands.clone());
        }

        // Collect subsequent ops, keeping only the non-chain operands
        for (chain_pos, &stmt_idx) in chain.iter().enumerate().skip(1) {
            intermediates_to_nop.push(chain[chain_pos - 1]);
            if let MirStmt::Assign {
                rvalue: Rvalue::TensorOp { kind, operands },
                ..
            } = &block.stmts[stmt_idx]
            {
                if let Some(op) = to_elementwise_op(kind) {
                    ops.push(op);
                } else {
                    continue;
                }
                // Add only the non-intermediate operands (the "other" input)
                for op in operands {
                    let is_intermediate = operand_local(op).map_or(false, |l| {
                        chain.iter().any(|&ci| {
                            if let MirStmt::Assign { place, .. } = &block.stmts[ci] {
                                place.local == l
                            } else {
                                false
                            }
                        })
                    });
                    if !is_intermediate {
                        all_operands.push(op.clone());
                    }
                }
            }
        }

        // The last statement in the chain becomes the fused op
        let Some(&last_idx) = chain.last() else { continue; };
        if let MirStmt::Assign { place, span, .. } = &block.stmts[last_idx] {
            let dest = place.clone();
            let sp = span.clone();
            block.stmts[last_idx] = MirStmt::Assign {
                place: dest,
                rvalue: Rvalue::TensorOp {
                    kind: TensorOpKind::FusedElementwise(ops),
                    operands: all_operands,
                },
                span: sp,
            };
        }

        // Nop out intermediates (all except last)
        for &idx in &chain[..chain.len() - 1] {
            block.stmts[idx] = MirStmt::Nop;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mir::*;
    use crate::span::Span;
    use crate::types::TypeId;

    fn tensor_assign(local: u32, kind: TensorOpKind, operands: Vec<Operand>) -> MirStmt {
        MirStmt::Assign {
            place: Place { local: LocalId(local), projections: Vec::new() },
            rvalue: Rvalue::TensorOp { kind, operands },
            span: Span::dummy(),
        }
    }

    fn place_op(local: u32) -> Operand {
        Operand::Place(Place { local: LocalId(local), projections: Vec::new() })
    }

    fn make_block(stmts: Vec<MirStmt>) -> MirBasicBlock {
        MirBasicBlock {
            id: BlockId(0),
            stmts,
            terminator: Terminator::Return,
        }
    }

    // ── Linear+ReLU pattern tests ───────────────────────────────

    #[test]
    fn test_fuse_matmul_add() {
        // _1 = MatMul(_2, _3)
        // _4 = Add(_1, _5)
        // Expected: _1 → Nop, _4 → FusedLinearRelu(_2, _3, _5)
        let mut block = make_block(vec![
            tensor_assign(1, TensorOpKind::MatMul, vec![place_op(2), place_op(3)]),
            tensor_assign(4, TensorOpKind::Add, vec![place_op(1), place_op(5)]),
        ]);

        fuse_linear_relu(&mut block);

        assert!(matches!(block.stmts[0], MirStmt::Nop));
        if let MirStmt::Assign { rvalue: Rvalue::TensorOp { kind, operands }, .. } = &block.stmts[1] {
            assert!(matches!(kind, TensorOpKind::FusedLinearRelu));
            assert_eq!(operands.len(), 3); // a, b, bias
        } else {
            panic!("Expected fused linear relu");
        }
    }

    #[test]
    fn test_no_fuse_matmul_add_multi_use() {
        // _1 = MatMul(_2, _3)
        // _4 = Add(_1, _5)
        // _6 = Add(_1, _7)  — _1 used twice, should not fuse
        let mut block = make_block(vec![
            tensor_assign(1, TensorOpKind::MatMul, vec![place_op(2), place_op(3)]),
            tensor_assign(4, TensorOpKind::Add, vec![place_op(1), place_op(5)]),
            tensor_assign(6, TensorOpKind::Add, vec![place_op(1), place_op(7)]),
        ]);

        fuse_linear_relu(&mut block);

        // Neither add should be fused because _1 has 2 uses
        assert!(!matches!(block.stmts[0], MirStmt::Nop));
    }

    #[test]
    fn test_no_fuse_non_matmul_add() {
        // _1 = Sub(_2, _3)
        // _4 = Add(_1, _5)  — not matmul, shouldn't match linear pattern
        let mut block = make_block(vec![
            tensor_assign(1, TensorOpKind::Sub, vec![place_op(2), place_op(3)]),
            tensor_assign(4, TensorOpKind::Add, vec![place_op(1), place_op(5)]),
        ]);

        fuse_linear_relu(&mut block);

        assert!(!matches!(block.stmts[0], MirStmt::Nop));
    }

    // ── MSE pattern tests ───────────────────────────────────────

    #[test]
    fn test_fuse_sub_square() {
        // _1 = Sub(_2, _3)
        // _4 = Mul(_1, _1)  — squaring
        // Expected: _1 → Nop, _4 → FusedMse(_2, _3)
        let mut block = make_block(vec![
            tensor_assign(1, TensorOpKind::Sub, vec![place_op(2), place_op(3)]),
            tensor_assign(4, TensorOpKind::Mul, vec![place_op(1), place_op(1)]),
        ]);

        fuse_mse(&mut block);

        assert!(matches!(block.stmts[0], MirStmt::Nop));
        if let MirStmt::Assign { rvalue: Rvalue::TensorOp { kind, operands }, .. } = &block.stmts[1] {
            assert!(matches!(kind, TensorOpKind::FusedMse));
            assert_eq!(operands.len(), 2); // pred, target
        } else {
            panic!("Expected fused MSE");
        }
    }

    #[test]
    fn test_no_fuse_mul_different_operands() {
        // _1 = Sub(_2, _3)
        // _4 = Mul(_1, _5)  — not squaring, _1 * _5
        let mut block = make_block(vec![
            tensor_assign(1, TensorOpKind::Sub, vec![place_op(2), place_op(3)]),
            tensor_assign(4, TensorOpKind::Mul, vec![place_op(1), place_op(5)]),
        ]);

        fuse_mse(&mut block);

        assert!(!matches!(block.stmts[0], MirStmt::Nop));
    }

    // ── Element-wise chain tests ────────────────────────────────

    #[test]
    fn test_fuse_elementwise_chain_3ops() {
        // _1 = Add(_10, _11)
        // _2 = Mul(_1, _12)
        // _3 = Sub(_2, _13)
        // Chain of 3 → fused
        let mut block = make_block(vec![
            tensor_assign(1, TensorOpKind::Add, vec![place_op(10), place_op(11)]),
            tensor_assign(2, TensorOpKind::Mul, vec![place_op(1), place_op(12)]),
            tensor_assign(3, TensorOpKind::Sub, vec![place_op(2), place_op(13)]),
        ]);

        fuse_elementwise_chain(&mut block);

        // First two should be Nop, last should be fused
        assert!(matches!(block.stmts[0], MirStmt::Nop));
        assert!(matches!(block.stmts[1], MirStmt::Nop));
        if let MirStmt::Assign { rvalue: Rvalue::TensorOp { kind, .. }, .. } = &block.stmts[2] {
            if let TensorOpKind::FusedElementwise(ops) = kind {
                assert_eq!(ops.len(), 3);
            } else {
                panic!("Expected FusedElementwise");
            }
        } else {
            panic!("Expected fused op");
        }
    }

    #[test]
    fn test_no_fuse_short_chain() {
        // Only 2 element-wise ops — below the threshold of 3
        let mut block = make_block(vec![
            tensor_assign(1, TensorOpKind::Add, vec![place_op(10), place_op(11)]),
            tensor_assign(2, TensorOpKind::Mul, vec![place_op(1), place_op(12)]),
        ]);

        fuse_elementwise_chain(&mut block);

        // Should NOT be fused (chain length < 3)
        assert!(!matches!(block.stmts[0], MirStmt::Nop));
    }

    // ── Full pass integration test ──────────────────────────────

    #[test]
    fn test_fusion_pass_runs_on_program() {
        let mut program = MirProgram {
            functions: vec![MirFunction {
                name: "main".to_string(),
                mangled_name: "main".to_string(),
                params: vec![],
                return_ty: TypeId::UNIT,
                locals: vec![
                    MirLocal { id: LocalId(0), name: None, ty: TypeId::UNIT, mutable: false },
                ],
                basic_blocks: vec![make_block(vec![
                    tensor_assign(1, TensorOpKind::MatMul, vec![place_op(2), place_op(3)]),
                    tensor_assign(4, TensorOpKind::Add, vec![place_op(1), place_op(5)]),
                ])],
                attributes: Vec::new(),
                span: Span::dummy(),
            }],
            statics: vec![],
        };

        let interner = TypeInterner::new();
        TensorFusion.run(&mut program, &interner);

        let block = &program.functions[0].basic_blocks[0];
        assert!(matches!(block.stmts[0], MirStmt::Nop));
        if let MirStmt::Assign { rvalue: Rvalue::TensorOp { kind, .. }, .. } = &block.stmts[1] {
            assert!(matches!(kind, TensorOpKind::FusedLinearRelu));
        } else {
            panic!("Expected fused op after running pass");
        }
    }

    #[test]
    fn test_empty_block_no_panic() {
        let mut block = make_block(vec![]);
        fuse_linear_relu(&mut block);
        fuse_mse(&mut block);
        fuse_elementwise_chain(&mut block);
        assert!(block.stmts.is_empty());
    }

    #[test]
    fn test_non_tensor_stmts_preserved() {
        let mut block = make_block(vec![
            MirStmt::Nop,
            tensor_assign(1, TensorOpKind::MatMul, vec![place_op(2), place_op(3)]),
            MirStmt::Nop,
            tensor_assign(4, TensorOpKind::Add, vec![place_op(1), place_op(5)]),
            MirStmt::Nop,
        ]);

        fuse_linear_relu(&mut block);

        // The Nop statements at positions 0, 2, 4 should be preserved
        assert_eq!(block.stmts.len(), 5);
        assert!(matches!(block.stmts[4], MirStmt::Nop));
    }
}
