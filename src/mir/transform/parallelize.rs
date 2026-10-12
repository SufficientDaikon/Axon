// mir/transform/parallelize.rs — Automatic tensor parallelism pass
//
// Analyzes data dependencies between tensor operations within each basic
// block and identifies groups of independent operations that can execute
// concurrently. Inserts ParallelBegin/ParallelEnd markers around these
// groups so the LLVM codegen can emit thread pool dispatch calls.
//
// Safety contract:
// - Only pure tensor ops with no shared mutable state are parallelized
// - Side-effecting ops (print, file IO, RNG) are never parallelized
// - Parallel results match sequential within floating-point epsilon

use std::collections::{HashMap, HashSet};

use crate::mir::{
    MirProgram, MirBasicBlock, MirStmt, Rvalue, TensorOpKind,
    Operand, LocalId,
};
use crate::types::TypeInterner;
use super::MirPass;

pub struct AutoParallelize;

impl MirPass for AutoParallelize {
    fn name(&self) -> &'static str {
        "parallelize"
    }

    fn run(&self, program: &mut MirProgram, _interner: &TypeInterner) -> bool {
        for func in &mut program.functions {
            for block in &mut func.basic_blocks {
                parallelize_block(block);
            }
        }
        true
    }
}

/// A node in the tensor operation DAG.
#[derive(Debug)]
struct DagNode {
    /// Index into the original statement list.
    stmt_idx: usize,
    /// Locals read by this operation.
    reads: HashSet<LocalId>,
    /// Local written by this operation.
    writes: Option<LocalId>,
    /// Whether this is a pure tensor op (safe to parallelize).
    is_pure_tensor: bool,
}

/// Build a DAG of tensor operations and their data dependencies,
/// then insert parallel region markers around independent groups.
fn parallelize_block(block: &mut MirBasicBlock) {
    // Step 1: Build DAG nodes for tensor ops
    let nodes = build_dag_nodes(&block.stmts);

    if nodes.len() < 2 {
        return; // Nothing to parallelize
    }

    // Step 2: Build adjacency (dependency edges)
    // An edge from node A → B means B depends on A (B reads what A writes)
    let deps = build_dependencies(&nodes);

    // Step 3: Topological sort into levels (parallel layers)
    let levels = topological_levels(&nodes, &deps);

    // Step 4: Find levels with width >= 2 (parallelizable groups)
    let parallel_groups: Vec<&Vec<usize>> = levels
        .iter()
        .filter(|level| {
            level.len() >= 2
                && level.iter().all(|&ni| nodes[ni].is_pure_tensor)
        })
        .collect();

    if parallel_groups.is_empty() {
        return;
    }

    // Step 5: Insert ParallelBegin/End markers
    // We need to rebuild the statement list with markers inserted.
    // Track which original statement indices are in parallel groups.
    let mut parallel_regions: Vec<(usize, usize, usize)> = Vec::new(); // (first_stmt_idx, last_stmt_idx, count)

    for group in &parallel_groups {
        let mut stmt_indices: Vec<usize> = group
            .iter()
            .map(|&ni| nodes[ni].stmt_idx)
            .collect();
        stmt_indices.sort();

        if stmt_indices.is_empty() {
            continue;
        }

        // Verify the statements are contiguous (or close enough)
        let first = stmt_indices[0];
        let Some(&last) = stmt_indices.last() else { continue; };
        let count = stmt_indices.len();

        // Only parallelize if the group members are actually adjacent
        // (allowing at most N-1 gaps of Nop/StorageLive/Dead between them)
        let non_gap_count = stmt_indices.len();
        let span = last - first + 1;
        if span <= non_gap_count * 2 {
            parallel_regions.push((first, last, count));
        }
    }

    if parallel_regions.is_empty() {
        return;
    }

    // Sort regions by starting index, then insert markers
    parallel_regions.sort_by_key(|&(first, _, _)| first);

    // Rebuild statements with markers
    let mut new_stmts = Vec::with_capacity(block.stmts.len() + parallel_regions.len() * 2);
    let mut region_iter = parallel_regions.iter().peekable();
    let mut i = 0;

    while i < block.stmts.len() {
        if let Some(&&(first, last, count)) = region_iter.peek() {
            if i == first {
                new_stmts.push(MirStmt::ParallelBegin { count });
                // Copy all statements in the region
                for j in first..=last {
                    new_stmts.push(block.stmts[j].clone());
                }
                new_stmts.push(MirStmt::ParallelEnd);
                region_iter.next();
                i = last + 1;
                continue;
            }
        }
        new_stmts.push(block.stmts[i].clone());
        i += 1;
    }

    block.stmts = new_stmts;
}

/// Build DAG nodes from statements, filtering to tensor operations.
fn build_dag_nodes(stmts: &[MirStmt]) -> Vec<DagNode> {
    let mut nodes = Vec::new();

    for (idx, stmt) in stmts.iter().enumerate() {
        if let MirStmt::Assign {
            place,
            rvalue,
            ..
        } = stmt
        {
            let reads = collect_reads(rvalue);
            let is_pure_tensor = is_pure_tensor_op(rvalue);

            if is_pure_tensor {
                nodes.push(DagNode {
                    stmt_idx: idx,
                    reads,
                    writes: Some(place.local),
                    is_pure_tensor,
                });
            }
        }
    }

    nodes
}

/// Collect all locals read by an rvalue.
fn collect_reads(rvalue: &Rvalue) -> HashSet<LocalId> {
    let mut reads = HashSet::new();
    match rvalue {
        Rvalue::Use(op) => collect_operand_reads(op, &mut reads),
        Rvalue::BinaryOp { left, right, .. } => {
            collect_operand_reads(left, &mut reads);
            collect_operand_reads(right, &mut reads);
        }
        Rvalue::UnaryOp { operand, .. } => collect_operand_reads(operand, &mut reads),
        Rvalue::TensorOp { operands, .. } => {
            for op in operands {
                collect_operand_reads(op, &mut reads);
            }
        }
        Rvalue::Aggregate { fields, .. } => {
            for op in fields {
                collect_operand_reads(op, &mut reads);
            }
        }
        Rvalue::Ref { place, .. } | Rvalue::Len { place } | Rvalue::Discriminant { place } => {
            reads.insert(place.local);
        }
        Rvalue::Cast { operand, .. } => collect_operand_reads(operand, &mut reads),
    }
    reads
}

fn collect_operand_reads(op: &Operand, reads: &mut HashSet<LocalId>) {
    if let Operand::Place(place) = op {
        reads.insert(place.local);
    }
}

/// Check if an rvalue is a pure tensor operation (no side effects).
fn is_pure_tensor_op(rvalue: &Rvalue) -> bool {
    match rvalue {
        Rvalue::TensorOp { kind, .. } => {
            // All current tensor ops are pure computations
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
                    | TensorOpKind::FusedLinearRelu
                    | TensorOpKind::FusedMse
                    | TensorOpKind::FusedSoftmaxCe
                    | TensorOpKind::FusedElementwise(_)
            )
        }
        _ => false,
    }
}

/// Build dependency edges: deps[i] = set of node indices that node i depends on.
fn build_dependencies(nodes: &[DagNode]) -> Vec<HashSet<usize>> {
    let mut deps = vec![HashSet::new(); nodes.len()];

    // Build a map: which node writes to which local
    let mut writer_map: HashMap<LocalId, usize> = HashMap::new();
    for (ni, node) in nodes.iter().enumerate() {
        if let Some(w) = node.writes {
            writer_map.insert(w, ni);
        }
    }

    // For each node, check if any of its reads were written by another node
    for (ni, node) in nodes.iter().enumerate() {
        for &read_local in &node.reads {
            if let Some(&writer_ni) = writer_map.get(&read_local) {
                if writer_ni != ni {
                    deps[ni].insert(writer_ni);
                }
            }
        }
    }

    deps
}

/// Compute topological levels (layers for parallel execution).
/// Each level contains nodes whose dependencies are all in earlier levels.
fn topological_levels(nodes: &[DagNode], deps: &[HashSet<usize>]) -> Vec<Vec<usize>> {
    let n = nodes.len();
    let mut levels = Vec::new();
    let mut assigned: HashSet<usize> = HashSet::new();

    loop {
        let mut current_level = Vec::new();
        for ni in 0..n {
            if assigned.contains(&ni) {
                continue;
            }
            // Check if all dependencies are already assigned
            if deps[ni].iter().all(|dep| assigned.contains(dep)) {
                current_level.push(ni);
            }
        }

        if current_level.is_empty() {
            break;
        }

        for &ni in &current_level {
            assigned.insert(ni);
        }
        levels.push(current_level);
    }

    levels
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

    #[test]
    fn test_independent_ops_parallelized() {
        // _1 = MatMul(_10, _11) — independent
        // _2 = MatMul(_12, _13) — independent
        // Both read different locals, so they can run in parallel.
        let mut block = make_block(vec![
            tensor_assign(1, TensorOpKind::MatMul, vec![place_op(10), place_op(11)]),
            tensor_assign(2, TensorOpKind::MatMul, vec![place_op(12), place_op(13)]),
        ]);

        parallelize_block(&mut block);

        // Should have: ParallelBegin(2), stmt, stmt, ParallelEnd
        assert_eq!(block.stmts.len(), 4);
        assert!(matches!(block.stmts[0], MirStmt::ParallelBegin { count: 2 }));
        assert!(matches!(block.stmts[3], MirStmt::ParallelEnd));
    }

    #[test]
    fn test_dependent_ops_not_parallelized() {
        // _1 = MatMul(_10, _11)
        // _2 = Add(_1, _12) — depends on _1
        let mut block = make_block(vec![
            tensor_assign(1, TensorOpKind::MatMul, vec![place_op(10), place_op(11)]),
            tensor_assign(2, TensorOpKind::Add, vec![place_op(1), place_op(12)]),
        ]);

        parallelize_block(&mut block);

        // No parallel markers — ops form a chain
        assert_eq!(block.stmts.len(), 2);
        assert!(!block.stmts.iter().any(|s| matches!(s, MirStmt::ParallelBegin { .. })));
    }

    #[test]
    fn test_mixed_independent_and_dependent() {
        // _1 = MatMul(_10, _11) — independent from _2
        // _2 = MatMul(_12, _13) — independent from _1
        // _3 = Add(_1, _2) — depends on both
        let mut block = make_block(vec![
            tensor_assign(1, TensorOpKind::MatMul, vec![place_op(10), place_op(11)]),
            tensor_assign(2, TensorOpKind::MatMul, vec![place_op(12), place_op(13)]),
            tensor_assign(3, TensorOpKind::Add, vec![place_op(1), place_op(2)]),
        ]);

        parallelize_block(&mut block);

        // _1 and _2 should be in a parallel region, _3 should be sequential after
        let has_begin = block.stmts.iter().any(|s| matches!(s, MirStmt::ParallelBegin { count: 2 }));
        let has_end = block.stmts.iter().any(|s| matches!(s, MirStmt::ParallelEnd));
        assert!(has_begin, "Should have parallel begin for independent ops");
        assert!(has_end, "Should have parallel end");
    }

    #[test]
    fn test_single_op_not_parallelized() {
        let mut block = make_block(vec![
            tensor_assign(1, TensorOpKind::MatMul, vec![place_op(10), place_op(11)]),
        ]);

        parallelize_block(&mut block);

        assert_eq!(block.stmts.len(), 1);
        assert!(!block.stmts.iter().any(|s| matches!(s, MirStmt::ParallelBegin { .. })));
    }

    #[test]
    fn test_empty_block_no_panic() {
        let mut block = make_block(vec![]);
        parallelize_block(&mut block);
        assert!(block.stmts.is_empty());
    }

    #[test]
    fn test_non_tensor_ops_ignored() {
        // Non-tensor Assign statements should not be parallelized
        let mut block = make_block(vec![
            MirStmt::Assign {
                place: Place { local: LocalId(1), projections: Vec::new() },
                rvalue: Rvalue::Use(Operand::Constant(MirConstant::Int(42))),
                span: Span::dummy(),
            },
            MirStmt::Assign {
                place: Place { local: LocalId(2), projections: Vec::new() },
                rvalue: Rvalue::Use(Operand::Constant(MirConstant::Int(43))),
                span: Span::dummy(),
            },
        ]);

        parallelize_block(&mut block);

        // No parallel markers — only tensor ops are parallelized
        assert_eq!(block.stmts.len(), 2);
        assert!(!block.stmts.iter().any(|s| matches!(s, MirStmt::ParallelBegin { .. })));
    }

    #[test]
    fn test_dag_node_construction() {
        let stmts = vec![
            tensor_assign(1, TensorOpKind::MatMul, vec![place_op(10), place_op(11)]),
            tensor_assign(2, TensorOpKind::Add, vec![place_op(1), place_op(12)]),
        ];

        let nodes = build_dag_nodes(&stmts);
        assert_eq!(nodes.len(), 2);
        assert_eq!(nodes[0].writes, Some(LocalId(1)));
        assert!(nodes[0].reads.contains(&LocalId(10)));
        assert!(nodes[0].reads.contains(&LocalId(11)));
        assert!(nodes[1].reads.contains(&LocalId(1)));
    }

    #[test]
    fn test_dependency_analysis() {
        let stmts = vec![
            tensor_assign(1, TensorOpKind::MatMul, vec![place_op(10), place_op(11)]),
            tensor_assign(2, TensorOpKind::Add, vec![place_op(1), place_op(12)]),
        ];

        let nodes = build_dag_nodes(&stmts);
        let deps = build_dependencies(&nodes);

        // Node 1 (Add) depends on node 0 (MatMul) because it reads _1
        assert!(deps[1].contains(&0));
        // Node 0 has no dependencies
        assert!(deps[0].is_empty());
    }

    #[test]
    fn test_topological_levels_independent() {
        let stmts = vec![
            tensor_assign(1, TensorOpKind::MatMul, vec![place_op(10), place_op(11)]),
            tensor_assign(2, TensorOpKind::MatMul, vec![place_op(12), place_op(13)]),
        ];

        let nodes = build_dag_nodes(&stmts);
        let deps = build_dependencies(&nodes);
        let levels = topological_levels(&nodes, &deps);

        // Both ops should be in the same level (independent)
        assert_eq!(levels.len(), 1);
        assert_eq!(levels[0].len(), 2);
    }

    #[test]
    fn test_topological_levels_chain() {
        let stmts = vec![
            tensor_assign(1, TensorOpKind::MatMul, vec![place_op(10), place_op(11)]),
            tensor_assign(2, TensorOpKind::Add, vec![place_op(1), place_op(12)]),
            tensor_assign(3, TensorOpKind::Mul, vec![place_op(2), place_op(13)]),
        ];

        let nodes = build_dag_nodes(&stmts);
        let deps = build_dependencies(&nodes);
        let levels = topological_levels(&nodes, &deps);

        // Each op depends on the previous, so 3 levels of width 1
        assert_eq!(levels.len(), 3);
        assert_eq!(levels[0].len(), 1);
        assert_eq!(levels[1].len(), 1);
        assert_eq!(levels[2].len(), 1);
    }

    #[test]
    fn test_full_pass_on_program() {
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
                    tensor_assign(1, TensorOpKind::MatMul, vec![place_op(10), place_op(11)]),
                    tensor_assign(2, TensorOpKind::MatMul, vec![place_op(12), place_op(13)]),
                ])],
                attributes: Vec::new(),
                span: Span::dummy(),
            }],
            statics: vec![],
        };

        let interner = TypeInterner::new();
        AutoParallelize.run(&mut program, &interner);

        let block = &program.functions[0].basic_blocks[0];
        assert!(block.stmts.iter().any(|s| matches!(s, MirStmt::ParallelBegin { .. })));
    }

    #[test]
    fn test_three_independent_ops() {
        let mut block = make_block(vec![
            tensor_assign(1, TensorOpKind::MatMul, vec![place_op(10), place_op(11)]),
            tensor_assign(2, TensorOpKind::MatMul, vec![place_op(12), place_op(13)]),
            tensor_assign(3, TensorOpKind::MatMul, vec![place_op(14), place_op(15)]),
        ]);

        parallelize_block(&mut block);

        // All 3 ops should be in one parallel region
        assert!(block.stmts.iter().any(|s| matches!(s, MirStmt::ParallelBegin { count: 3 })));
    }

    #[test]
    fn test_is_pure_tensor_op() {
        assert!(is_pure_tensor_op(&Rvalue::TensorOp {
            kind: TensorOpKind::MatMul,
            operands: vec![],
        }));
        assert!(is_pure_tensor_op(&Rvalue::TensorOp {
            kind: TensorOpKind::FusedLinearRelu,
            operands: vec![],
        }));
        assert!(!is_pure_tensor_op(&Rvalue::Use(Operand::Constant(MirConstant::Int(42)))));
    }
}
