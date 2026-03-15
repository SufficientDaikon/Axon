// mir/transform/dead_code.rs — Dead code elimination pass
//
// Removes unreachable functions (not called from main) and
// unreachable basic blocks within functions.

use std::collections::HashSet;

use crate::mir::{MirProgram, Operand, MirConstant, Terminator};
use crate::types::TypeInterner;
use super::MirPass;

pub struct DeadCodeElimination;

impl MirPass for DeadCodeElimination {
    fn name(&self) -> &'static str {
        "dead_code"
    }

    fn run(&self, program: &mut MirProgram, _interner: &TypeInterner) {
        eliminate_unreachable_functions(program);
        for func in &mut program.functions {
            eliminate_unreachable_blocks(func);
        }
    }
}

/// Remove functions not reachable from `main` via the call graph.
fn eliminate_unreachable_functions(program: &mut MirProgram) {
    if program.functions.is_empty() {
        return;
    }

    // Build call graph: collect all functions called from each function
    let mut reachable: HashSet<String> = HashSet::new();

    // Seed with main (the entry point is always "main")
    let has_main = program.functions.iter().any(|f| f.name == "main");
    if !has_main {
        return; // No main function, nothing to eliminate
    }
    reachable.insert("main".to_string());

    // BFS through the call graph
    let mut worklist: Vec<String> = vec!["main".to_string()];

    while let Some(func_name) = worklist.pop() {
        // Find the function
        if let Some(func) = program.functions.iter().find(|f| f.name == func_name) {
            for block in &func.basic_blocks {
                // Check terminator for calls
                if let Terminator::Call { func: callee, .. } = &block.terminator {
                    if let Operand::Constant(MirConstant::String(name)) = callee {
                        if reachable.insert(name.clone()) {
                            worklist.push(name.clone());
                        }
                    }
                }
                // Also check statements for calls embedded in rvalues
                // (currently calls are only in terminators, but future-proof)
            }
        }
    }

    // Remove unreachable functions
    program.functions.retain(|f| reachable.contains(&f.name));
}

/// Remove basic blocks not reachable from block 0 within a function.
fn eliminate_unreachable_blocks(func: &mut crate::mir::MirFunction) {
    if func.basic_blocks.is_empty() {
        return;
    }

    let mut reachable: HashSet<u32> = HashSet::new();
    let mut worklist: Vec<u32> = vec![0]; // Block 0 is always the entry
    reachable.insert(0);

    while let Some(block_id) = worklist.pop() {
        if let Some(block) = func.basic_blocks.iter().find(|b| b.id.0 == block_id) {
            let targets = terminator_targets(&block.terminator);
            for t in targets {
                if reachable.insert(t) {
                    worklist.push(t);
                }
            }
        }
    }

    func.basic_blocks.retain(|b| reachable.contains(&b.id.0));
}

/// Get all block IDs that a terminator can jump to.
fn terminator_targets(term: &Terminator) -> Vec<u32> {
    match term {
        Terminator::Goto { target } => vec![target.0],
        Terminator::SwitchInt { targets, otherwise, .. } => {
            let mut result: Vec<u32> = targets.iter().map(|(_, b)| b.0).collect();
            result.push(otherwise.0);
            result
        }
        Terminator::Return => vec![],
        Terminator::Call { target, .. } => vec![target.0],
        Terminator::Assert { target, .. } => vec![target.0],
        Terminator::Unreachable => vec![],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mir::*;
    use crate::span::Span;
    use crate::types::TypeId;

    fn make_simple_function(name: &str, calls: &[&str]) -> MirFunction {
        let mut stmts = Vec::new();
        // Add a Nop to have some content
        stmts.push(MirStmt::Nop);

        let mut blocks = Vec::new();

        if calls.is_empty() {
            blocks.push(MirBasicBlock {
                id: BlockId(0),
                stmts,
                terminator: Terminator::Return,
            });
        } else {
            // Create a call for each callee, chaining blocks
            for (i, callee) in calls.iter().enumerate() {
                let next_block = (i + 1) as u32;
                blocks.push(MirBasicBlock {
                    id: BlockId(i as u32),
                    stmts: vec![MirStmt::Nop],
                    terminator: Terminator::Call {
                        func: Operand::Constant(MirConstant::String(callee.to_string())),
                        args: vec![],
                        destination: Place { local: LocalId(0), projections: vec![] },
                        target: BlockId(next_block),
                    },
                });
            }
            blocks.push(MirBasicBlock {
                id: BlockId(calls.len() as u32),
                stmts: vec![],
                terminator: Terminator::Return,
            });
        }

        MirFunction {
            name: name.to_string(),
            mangled_name: name.to_string(),
            params: vec![],
            return_ty: TypeId::UNIT,
            locals: vec![MirLocal { id: LocalId(0), name: None, ty: TypeId::UNIT, mutable: false }],
            basic_blocks: blocks,
            attributes: vec![],
            span: Span::dummy(),
        }
    }

    #[test]
    fn test_removes_unreachable_function() {
        let mut program = MirProgram {
            functions: vec![
                make_simple_function("main", &[]),
                make_simple_function("unused", &[]),
            ],
            statics: vec![],
        };

        eliminate_unreachable_functions(&mut program);
        assert_eq!(program.functions.len(), 1);
        assert_eq!(program.functions[0].name, "main");
    }

    #[test]
    fn test_keeps_called_function() {
        let mut program = MirProgram {
            functions: vec![
                make_simple_function("main", &["helper"]),
                make_simple_function("helper", &[]),
            ],
            statics: vec![],
        };

        eliminate_unreachable_functions(&mut program);
        assert_eq!(program.functions.len(), 2);
    }

    #[test]
    fn test_transitive_call_kept() {
        let mut program = MirProgram {
            functions: vec![
                make_simple_function("main", &["a"]),
                make_simple_function("a", &["b"]),
                make_simple_function("b", &[]),
                make_simple_function("orphan", &[]),
            ],
            statics: vec![],
        };

        eliminate_unreachable_functions(&mut program);
        assert_eq!(program.functions.len(), 3);
        let names: Vec<&str> = program.functions.iter().map(|f| f.name.as_str()).collect();
        assert!(names.contains(&"main"));
        assert!(names.contains(&"a"));
        assert!(names.contains(&"b"));
        assert!(!names.contains(&"orphan"));
    }

    #[test]
    fn test_empty_program() {
        let mut program = MirProgram { functions: vec![], statics: vec![] };
        eliminate_unreachable_functions(&mut program);
        assert_eq!(program.functions.len(), 0);
    }

    #[test]
    fn test_removes_unreachable_blocks() {
        let mut func = MirFunction {
            name: "test".to_string(),
            mangled_name: "test".to_string(),
            params: vec![],
            return_ty: TypeId::UNIT,
            locals: vec![],
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
                MirBasicBlock {
                    id: BlockId(2), // unreachable
                    stmts: vec![MirStmt::Nop],
                    terminator: Terminator::Return,
                },
            ],
            attributes: vec![],
            span: Span::dummy(),
        };

        eliminate_unreachable_blocks(&mut func);
        assert_eq!(func.basic_blocks.len(), 2);
        let ids: Vec<u32> = func.basic_blocks.iter().map(|b| b.id.0).collect();
        assert!(ids.contains(&0));
        assert!(ids.contains(&1));
        assert!(!ids.contains(&2));
    }
}
