// mir/transform/mod.rs — MIR optimization pass infrastructure

pub mod dead_code;
pub mod const_fold;

use crate::mir::MirProgram;
use crate::types::TypeInterner;

/// A transformation pass over MIR.
pub trait MirPass {
    /// Human-readable pass name.
    fn name(&self) -> &'static str;

    /// Run the pass, mutating the MIR program in place.
    fn run(&self, program: &mut MirProgram, interner: &TypeInterner);
}

/// Manages and runs a sequence of MIR passes.
pub struct PassManager {
    passes: Vec<Box<dyn MirPass>>,
}

impl PassManager {
    pub fn new() -> Self {
        PassManager { passes: Vec::new() }
    }

    pub fn add_pass(&mut self, pass: Box<dyn MirPass>) {
        self.passes.push(pass);
    }

    /// Run all registered passes in order.
    pub fn run_all(&self, program: &mut MirProgram, interner: &TypeInterner) {
        for pass in &self.passes {
            pass.run(program, interner);
        }
    }

    /// Create a PassManager with default passes for a given optimization level.
    pub fn with_default_passes(opt_level: u8) -> Self {
        let mut pm = Self::new();
        if opt_level >= 1 {
            pm.add_pass(Box::new(dead_code::DeadCodeElimination));
            pm.add_pass(Box::new(const_fold::ConstantFolding));
        }
        pm
    }

    pub fn pass_count(&self) -> usize {
        self.passes.len()
    }
}

impl Default for PassManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pass_manager_empty() {
        let pm = PassManager::new();
        assert_eq!(pm.pass_count(), 0);
    }

    #[test]
    fn test_pass_manager_default_o0() {
        let pm = PassManager::with_default_passes(0);
        assert_eq!(pm.pass_count(), 0);
    }

    #[test]
    fn test_pass_manager_default_o1() {
        let pm = PassManager::with_default_passes(1);
        assert_eq!(pm.pass_count(), 2);
    }

    #[test]
    fn test_pass_manager_default_o2() {
        let pm = PassManager::with_default_passes(2);
        assert_eq!(pm.pass_count(), 2);
    }
}
