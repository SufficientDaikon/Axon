// mir/transform/graph_rewrite.rs — Algebraic simplification and CSE for tensor operations
//
// Peephole rewrites on MIR tensor operations:
//
//  1. Identity elimination:  x - x → broadcast (zero-like)
//  2. Double unary:          transpose(transpose(x)) → x
//  3. Reshape collapse:      reshape(reshape(x, s1), s2) → reshape(x, s2)
//  4. Common subexpression elimination (CSE) for pure tensor ops

use std::collections::HashMap;

use crate::mir::{
    MirProgram, MirBasicBlock, MirStmt,
    Operand, Rvalue, TensorOpKind, LocalId,
};
use crate::types::TypeInterner;
use super::MirPass;

pub struct GraphRewrite;

impl MirPass for GraphRewrite {
    fn name(&self) -> &'static str {
        "graph_rewrite"
    }

    fn run(&self, program: &mut MirProgram, _interner: &TypeInterner) -> bool {
        for func in &mut program.functions {
            for block in &mut func.basic_blocks {
                eliminate_identities(block);
                eliminate_double_unary(block);
                collapse_reshapes(block);
                common_subexpression_elimination(block);
            }
        }
        true
    }
}

// ── Helpers ──────────────────────────────────────────────────

/// Extract the LocalId from an Operand if it's a simple place reference.
fn operand_local(op: &Operand) -> Option<LocalId> {
    match op {
        Operand::Place(place) => Some(place.local),
        _ => None,
    }
}

/// Build a map: LocalId → (stmt_idx, TensorOpKind, operands)
fn build_def_map(block: &MirBasicBlock) -> HashMap<LocalId, (usize, TensorOpKind, Vec<Operand>)> {
    let mut defs = HashMap::new();
    for (idx, stmt) in block.stmts.iter().enumerate() {
        if let MirStmt::Assign {
            place,
            rvalue: Rvalue::TensorOp { kind, operands },
            ..
        } = stmt
        {
            defs.insert(place.local, (idx, kind.clone(), operands.clone()));
        }
    }
    defs
}

/// Collect all locals used as operands, incrementing their count in the map.
fn collect_used_locals(rvalue: &Rvalue, counts: &mut HashMap<LocalId, usize>) {
    let mut bump = |op: &Operand| {
        if let Some(id) = operand_local(op) {
            *counts.entry(id).or_insert(0) += 1;
        }
    };
    match rvalue {
        Rvalue::Use(op) => bump(op),
        Rvalue::BinaryOp { left, right, .. } => { bump(left); bump(right); }
        Rvalue::UnaryOp { operand, .. } => bump(operand),
        Rvalue::TensorOp { operands, .. } => { for op in operands { bump(op); } }
        Rvalue::Aggregate { fields, .. } => { for op in fields { bump(op); } }
        _ => {}
    }
}

// ── Pattern 1: Identity elimination ─────────────────────────

/// Rewrites: x - x → broadcast (zero-like)
fn eliminate_identities(block: &mut MirBasicBlock) {
    for stmt in &mut block.stmts {
        if let MirStmt::Assign {
            rvalue: Rvalue::TensorOp { kind, operands },
            ..
        } = stmt
        {
            if operands.len() != 2 { continue; }

            let left_id = operand_local(&operands[0]);
            let right_id = operand_local(&operands[1]);

            // x - x → broadcast (zero propagation)
            if matches!(kind, TensorOpKind::Sub) && left_id.is_some() && left_id == right_id {
                *kind = TensorOpKind::Broadcast;
                operands.truncate(1);
            }
        }
    }
}

// ── Pattern 2: Double unary elimination ─────────────────────

/// Rewrites: transpose(transpose(x)) → broadcast(x) (identity passthrough)
fn eliminate_double_unary(block: &mut MirBasicBlock) {
    let defs = build_def_map(block);

    // Pre-compute use counts to avoid borrow conflict
    let use_counts: HashMap<LocalId, usize> = {
        let mut counts = HashMap::new();
        for stmt in &block.stmts {
            if let MirStmt::Assign { rvalue, .. } = stmt {
                collect_used_locals(rvalue, &mut counts);
            }
        }
        counts
    };

    for stmt in &mut block.stmts {
        if let MirStmt::Assign {
            rvalue: Rvalue::TensorOp { kind, operands },
            ..
        } = stmt
        {
            if operands.len() != 1 { continue; }

            let inner_id = match operand_local(&operands[0]) {
                Some(id) => id,
                None => continue,
            };

            // Only eliminate if the inner result is used exactly once (this use)
            if use_counts.get(&inner_id).copied().unwrap_or(0) != 1 { continue; }

            if let Some((_, inner_kind, inner_operands)) = defs.get(&inner_id) {
                // transpose(transpose(x)) → x
                if matches!(kind, TensorOpKind::Transpose)
                    && matches!(inner_kind, TensorOpKind::Transpose)
                    && inner_operands.len() == 1
                {
                    operands[0] = inner_operands[0].clone();
                    *kind = TensorOpKind::Broadcast; // identity passthrough
                }
            }
        }
    }
}

// ── Pattern 3: Reshape collapse ─────────────────────────────

/// Rewrites: reshape(reshape(x, s1), s2) → reshape(x, s2)
fn collapse_reshapes(block: &mut MirBasicBlock) {
    let defs = build_def_map(block);

    // Pre-compute use counts to avoid borrow conflict
    let use_counts: HashMap<LocalId, usize> = {
        let mut counts = HashMap::new();
        for stmt in &block.stmts {
            if let MirStmt::Assign { rvalue, .. } = stmt {
                collect_used_locals(rvalue, &mut counts);
            }
        }
        counts
    };

    for stmt in &mut block.stmts {
        if let MirStmt::Assign {
            rvalue: Rvalue::TensorOp { kind: TensorOpKind::Reshape(_), operands },
            ..
        } = stmt
        {
            if operands.len() != 1 { continue; }

            let inner_id = match operand_local(&operands[0]) {
                Some(id) => id,
                None => continue,
            };

            // Only collapse if the inner reshape is used exactly once
            if use_counts.get(&inner_id).copied().unwrap_or(0) != 1 { continue; }

            if let Some((_, TensorOpKind::Reshape(_), inner_operands)) = defs.get(&inner_id) {
                if inner_operands.len() == 1 {
                    // Skip the intermediate reshape
                    operands[0] = inner_operands[0].clone();
                }
            }
        }
    }
}

// ── Pattern 4: Common subexpression elimination ─────────────

/// A hashable key for a tensor operation (kind + operand locals).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct TensorOpKey {
    kind_tag: String,
    operand_ids: Vec<Option<LocalId>>,
}

fn make_tensor_op_key(kind: &TensorOpKind, operands: &[Operand]) -> TensorOpKey {
    TensorOpKey {
        kind_tag: format!("{:?}", kind),
        operand_ids: operands.iter().map(operand_local).collect(),
    }
}

/// Check if a tensor op is safe for CSE (deterministic and side-effect-free).
fn is_pure_for_cse(kind: &TensorOpKind) -> bool {
    matches!(
        kind,
        TensorOpKind::MatMul
            | TensorOpKind::Add
            | TensorOpKind::Sub
            | TensorOpKind::Mul
            | TensorOpKind::Div
            | TensorOpKind::Reshape(_)
            | TensorOpKind::Transpose
            | TensorOpKind::Broadcast
    )
}

/// Replaces duplicate tensor ops with references to the first occurrence.
fn common_subexpression_elimination(block: &mut MirBasicBlock) {
    // First pass: find all tensor op assignments and their keys
    let mut seen: HashMap<TensorOpKey, LocalId> = HashMap::new();
    let mut replacements: HashMap<LocalId, LocalId> = HashMap::new();

    for stmt in &block.stmts {
        if let MirStmt::Assign {
            place,
            rvalue: Rvalue::TensorOp { kind, operands },
            ..
        } = stmt
        {
            if !is_pure_for_cse(kind) { continue; }

            let key = make_tensor_op_key(kind, operands);
            if let Some(&existing_id) = seen.get(&key) {
                replacements.insert(place.local, existing_id);
            } else {
                seen.insert(key, place.local);
            }
        }
    }

    if replacements.is_empty() { return; }

    // Second pass: replace uses of duplicated locals
    for stmt in &mut block.stmts {
        if let MirStmt::Assign { rvalue, .. } = stmt {
            replace_operands_in_rvalue(rvalue, &replacements);
        }
    }

    // Third pass: replace duplicate assignments with Nop
    for stmt in &mut block.stmts {
        if let MirStmt::Assign { place, .. } = stmt {
            if replacements.contains_key(&place.local) {
                *stmt = MirStmt::Nop;
            }
        }
    }
}

fn replace_operands_in_rvalue(rvalue: &mut Rvalue, replacements: &HashMap<LocalId, LocalId>) {
    match rvalue {
        Rvalue::TensorOp { operands, .. } => {
            for op in operands.iter_mut() {
                replace_operand(op, replacements);
            }
        }
        Rvalue::BinaryOp { left, right, .. } => {
            replace_operand(left, replacements);
            replace_operand(right, replacements);
        }
        Rvalue::UnaryOp { operand, .. } => {
            replace_operand(operand, replacements);
        }
        Rvalue::Use(op) => {
            replace_operand(op, replacements);
        }
        _ => {}
    }
}

fn replace_operand(op: &mut Operand, replacements: &HashMap<LocalId, LocalId>) {
    if let Operand::Place(place) = op {
        if let Some(&new_id) = replacements.get(&place.local) {
            place.local = new_id;
        }
    }
}

// ── Tests ───────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mir::*;
    use crate::span::Span;
    use crate::types::TypeId;

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
                locals: vec![],
                basic_blocks: vec![make_block(stmts)],
                attributes: Vec::new(),
                span: Span::dummy(),
            }],
            statics: vec![],
        }
    }

    #[test]
    fn test_self_subtraction_eliminated() {
        // x - x → broadcast (zero-like)
        let stmts = vec![
            tensor_assign(1, TensorOpKind::Sub, vec![place_op(0), place_op(0)]),
        ];
        let mut prog = make_program(stmts);
        let interner = TypeInterner::new();
        GraphRewrite.run(&mut prog, &interner);

        let stmt = &prog.functions[0].basic_blocks[0].stmts[0];
        if let MirStmt::Assign { rvalue: Rvalue::TensorOp { kind, operands }, .. } = stmt {
            assert!(matches!(kind, TensorOpKind::Broadcast));
            assert_eq!(operands.len(), 1);
        } else {
            panic!("expected tensor op");
        }
    }

    #[test]
    fn test_double_transpose_eliminated() {
        // transpose(transpose(x)) → broadcast (identity)
        let stmts = vec![
            tensor_assign(1, TensorOpKind::Transpose, vec![place_op(0)]),
            tensor_assign(2, TensorOpKind::Transpose, vec![place_op(1)]),
        ];
        let mut prog = make_program(stmts);
        let interner = TypeInterner::new();
        GraphRewrite.run(&mut prog, &interner);

        let stmt = &prog.functions[0].basic_blocks[0].stmts[1];
        if let MirStmt::Assign { rvalue: Rvalue::TensorOp { kind, operands }, .. } = stmt {
            assert!(matches!(kind, TensorOpKind::Broadcast));
            if let Operand::Place(place) = &operands[0] {
                assert_eq!(place.local.0, 0, "should reference original input, not intermediate");
            }
        } else {
            panic!("expected tensor op");
        }
    }

    #[test]
    fn test_reshape_collapse() {
        // reshape(reshape(x, [2,3]), [6]) → reshape(x, [6])
        let stmts = vec![
            tensor_assign(1, TensorOpKind::Reshape(vec![2, 3]), vec![place_op(0)]),
            tensor_assign(2, TensorOpKind::Reshape(vec![6]), vec![place_op(1)]),
        ];
        let mut prog = make_program(stmts);
        let interner = TypeInterner::new();
        GraphRewrite.run(&mut prog, &interner);

        let stmt = &prog.functions[0].basic_blocks[0].stmts[1];
        if let MirStmt::Assign { rvalue: Rvalue::TensorOp { kind, operands }, .. } = stmt {
            assert!(matches!(kind, TensorOpKind::Reshape(s) if s == &[6]));
            if let Operand::Place(place) = &operands[0] {
                assert_eq!(place.local.0, 0, "should skip intermediate reshape");
            }
        } else {
            panic!("expected tensor op");
        }
    }

    #[test]
    fn test_cse_deduplicates_tensor_ops() {
        // Two identical matmul(a, b) → second replaced with Nop
        let stmts = vec![
            tensor_assign(2, TensorOpKind::MatMul, vec![place_op(0), place_op(1)]),
            tensor_assign(3, TensorOpKind::MatMul, vec![place_op(0), place_op(1)]),
        ];
        let mut prog = make_program(stmts);
        let interner = TypeInterner::new();
        GraphRewrite.run(&mut prog, &interner);

        assert!(matches!(&prog.functions[0].basic_blocks[0].stmts[1], MirStmt::Nop));
    }

    #[test]
    fn test_cse_different_ops_not_deduplicated() {
        // matmul(a, b) and add(a, b) should NOT be deduplicated
        let stmts = vec![
            tensor_assign(2, TensorOpKind::MatMul, vec![place_op(0), place_op(1)]),
            tensor_assign(3, TensorOpKind::Add, vec![place_op(0), place_op(1)]),
        ];
        let mut prog = make_program(stmts);
        let interner = TypeInterner::new();
        GraphRewrite.run(&mut prog, &interner);

        assert!(!matches!(&prog.functions[0].basic_blocks[0].stmts[1], MirStmt::Nop));
    }

    #[test]
    fn test_cse_rewires_users() {
        // c = matmul(a, b), d = matmul(a, b), e = add(d, a)
        // After CSE: d becomes Nop, e uses c instead of d
        let stmts = vec![
            tensor_assign(2, TensorOpKind::MatMul, vec![place_op(0), place_op(1)]),
            tensor_assign(3, TensorOpKind::MatMul, vec![place_op(0), place_op(1)]),
            tensor_assign(4, TensorOpKind::Add, vec![place_op(3), place_op(0)]),
        ];
        let mut prog = make_program(stmts);
        let interner = TypeInterner::new();
        GraphRewrite.run(&mut prog, &interner);

        // The add should now use local(2) instead of local(3)
        let add_stmt = &prog.functions[0].basic_blocks[0].stmts[2];
        if let MirStmt::Assign { rvalue: Rvalue::TensorOp { operands, .. }, .. } = add_stmt {
            if let Operand::Place(place) = &operands[0] {
                assert_eq!(place.local.0, 2, "CSE should rewire to first occurrence");
            }
        }
    }

    #[test]
    fn test_empty_block_no_crash() {
        let mut prog = make_program(vec![]);
        let interner = TypeInterner::new();
        GraphRewrite.run(&mut prog, &interner);
        assert!(prog.functions[0].basic_blocks[0].stmts.is_empty());
    }

    #[test]
    fn test_non_pure_ops_not_cse() {
        // FusedLinearRelu is not in the pure list, should not be CSE'd
        let stmts = vec![
            tensor_assign(2, TensorOpKind::FusedLinearRelu, vec![place_op(0), place_op(1)]),
            tensor_assign(3, TensorOpKind::FusedLinearRelu, vec![place_op(0), place_op(1)]),
        ];
        let mut prog = make_program(stmts);
        let interner = TypeInterner::new();
        GraphRewrite.run(&mut prog, &interner);

        assert!(!matches!(&prog.functions[0].basic_blocks[0].stmts[1], MirStmt::Nop));
    }
}
