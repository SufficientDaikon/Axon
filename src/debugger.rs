// debugger.rs — Debugger skeleton (DAP architecture planned for future release)
//
// This module defines the planned Debug Adapter Protocol (DAP) interface for
// the Axon debugger. All methods currently return "not implemented" errors.
// The full debugger will be implemented in a future phase using LLVM debug
// info (DWARF) and the DAP protocol for VS Code / editor integration.
//
// Architecture:
//   Debugger ←→ DAP server ←→ VS Code / editor
//              ↓
//         LLVM debug info (breakpoints, stepping, variable inspection)

use std::collections::HashMap;
use std::path::PathBuf;

/// Breakpoint representation.
#[derive(Debug, Clone)]
pub struct Breakpoint {
    pub id: u32,
    pub file: PathBuf,
    pub line: u32,
    pub enabled: bool,
    pub condition: Option<String>,
}

/// Debug session state.
#[derive(Debug, Clone, PartialEq)]
pub enum DebugState {
    /// Not yet launched.
    Idle,
    /// Running (no breakpoint hit).
    Running,
    /// Paused at a breakpoint or step.
    Paused,
    /// Target program has exited.
    Exited,
}

/// Axon debugger — manages debug sessions, breakpoints, and stepping.
#[allow(dead_code)]
pub struct Debugger {
    state: DebugState,
    breakpoints: HashMap<u32, Breakpoint>,
    next_bp_id: u32,
    /// Path to the source file being debugged.
    source_file: Option<PathBuf>,
    /// Current line (when paused).
    current_line: Option<u32>,
}

/// Error returned by debugger operations that are not yet implemented.
#[derive(Debug)]
pub struct DebugError {
    pub message: String,
}

impl std::fmt::Display for DebugError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "DebugError: {}", self.message)
    }
}

impl DebugError {
    fn not_implemented(op: &str) -> Self {
        DebugError {
            message: format!("{} is not yet implemented — planned for a future release", op),
        }
    }
}

impl Debugger {
    /// Create a new debugger instance.
    pub fn new() -> Self {
        Debugger {
            state: DebugState::Idle,
            breakpoints: HashMap::new(),
            next_bp_id: 1,
            source_file: None,
            current_line: None,
        }
    }

    /// Get the current debug state.
    pub fn state(&self) -> &DebugState {
        &self.state
    }

    /// Launch a debug session for the given source file.
    ///
    /// TODO: In the future, this will:
    ///   1. Compile the source with debug info (`-g`)
    ///   2. Spawn the target process under debugger control
    ///   3. Set up DAP communication
    pub fn launch(&mut self, file: &str) -> Result<(), DebugError> {
        self.source_file = Some(PathBuf::from(file));
        Err(DebugError::not_implemented("launch"))
    }

    /// Set a breakpoint at the given file and line.
    ///
    /// TODO: Will translate to LLVM debug info addresses.
    pub fn set_breakpoint(&mut self, file: &str, line: u32) -> Result<Breakpoint, DebugError> {
        let bp = Breakpoint {
            id: self.next_bp_id,
            file: PathBuf::from(file),
            line,
            enabled: true,
            condition: None,
        };
        self.next_bp_id += 1;
        self.breakpoints.insert(bp.id, bp);
        // Breakpoint is registered in the data structure but cannot be enforced yet
        Err(DebugError::not_implemented("set_breakpoint (breakpoint registered but runtime enforcement not available)"))
    }

    /// Continue execution until the next breakpoint or program exit.
    pub fn continue_(&mut self) -> Result<(), DebugError> {
        Err(DebugError::not_implemented("continue"))
    }

    /// Step over the current line (execute but don't step into function calls).
    pub fn step_over(&mut self) -> Result<(), DebugError> {
        Err(DebugError::not_implemented("step_over"))
    }

    /// Step into the current function call.
    pub fn step_into(&mut self) -> Result<(), DebugError> {
        Err(DebugError::not_implemented("step_into"))
    }

    /// Evaluate an expression in the current debug context.
    ///
    /// TODO: Will use the type checker + JIT to evaluate expressions
    /// in the paused scope, with access to local variables.
    pub fn evaluate(&self, _expr: &str) -> Result<String, DebugError> {
        Err(DebugError::not_implemented("evaluate"))
    }

    /// List all breakpoints.
    pub fn list_breakpoints(&self) -> Vec<&Breakpoint> {
        self.breakpoints.values().collect()
    }

    /// Remove a breakpoint by ID.
    pub fn remove_breakpoint(&mut self, id: u32) -> Result<(), DebugError> {
        if self.breakpoints.remove(&id).is_some() {
            Ok(())
        } else {
            Err(DebugError {
                message: format!("breakpoint {} not found", id),
            })
        }
    }
}

// ═══════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_debugger_initial_state() {
        let dbg = Debugger::new();
        assert_eq!(*dbg.state(), DebugState::Idle);
        assert!(dbg.list_breakpoints().is_empty());
    }

    #[test]
    fn test_launch_returns_not_implemented() {
        let mut dbg = Debugger::new();
        let result = dbg.launch("test.axon");
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("not yet implemented"));
    }

    #[test]
    fn test_set_breakpoint_returns_not_implemented() {
        let mut dbg = Debugger::new();
        let result = dbg.set_breakpoint("test.axon", 10);
        assert!(result.is_err());
        // Breakpoint should still be registered in the data structure
        assert_eq!(dbg.list_breakpoints().len(), 1);
    }

    #[test]
    fn test_continue_returns_not_implemented() {
        let mut dbg = Debugger::new();
        assert!(dbg.continue_().is_err());
    }

    #[test]
    fn test_step_over_returns_not_implemented() {
        let mut dbg = Debugger::new();
        assert!(dbg.step_over().is_err());
    }

    #[test]
    fn test_step_into_returns_not_implemented() {
        let mut dbg = Debugger::new();
        assert!(dbg.step_into().is_err());
    }

    #[test]
    fn test_evaluate_returns_not_implemented() {
        let dbg = Debugger::new();
        assert!(dbg.evaluate("1 + 2").is_err());
    }

    #[test]
    fn test_remove_breakpoint() {
        let mut dbg = Debugger::new();
        let _ = dbg.set_breakpoint("test.axon", 10);
        assert_eq!(dbg.list_breakpoints().len(), 1);
        assert!(dbg.remove_breakpoint(1).is_ok());
        assert!(dbg.list_breakpoints().is_empty());
    }

    #[test]
    fn test_remove_nonexistent_breakpoint() {
        let mut dbg = Debugger::new();
        let result = dbg.remove_breakpoint(999);
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("not found"));
    }
}
