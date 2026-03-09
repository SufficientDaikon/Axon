//! End-to-end compilation tests for the Axon compiler.
//!
//! Each test compiles an `.axon` source file to a native executable,
//! runs it, and checks the exit code (0 = success).

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Full pipeline: source → parse → typecheck → MIR → LLVM IR → compile+link → run.
fn compile_and_run(source: &str, test_name: &str) -> Result<std::process::Output, String> {
    // Step 1: Parse + type check
    let (typed_program, errors) = axonc::check_source(source, &format!("{}.axon", test_name));
    if !errors.is_empty() {
        let msgs: Vec<String> = errors.iter().map(|e| format!("{:?}", e)).collect();
        return Err(format!("Type check errors:\n{}", msgs.join("\n")));
    }

    // Step 2: Build MIR
    let (checker, _) = axonc::typeck::check(source, &format!("{}.axon", test_name));
    let mut mir_builder = axonc::mir::MirBuilder::new(&checker.interner);
    let mir_program = mir_builder.build(&typed_program);

    // Step 3: Generate LLVM IR
    let mut codegen =
        axonc::codegen::llvm::LlvmCodegen::new(&checker.interner, axonc::codegen::llvm::OptLevel::O0);
    let ir = codegen.generate(&mir_program);

    // Step 4: Compile + link
    let tmp_dir = std::env::temp_dir().join(format!("axon_e2e_{}", test_name));
    let _ = fs::remove_dir_all(&tmp_dir);
    fs::create_dir_all(&tmp_dir).map_err(|e| format!("mkdir: {}", e))?;

    let exe_ext = if cfg!(target_os = "windows") { ".exe" } else { "" };
    let output_path = tmp_dir.join(format!("{}{}", test_name, exe_ext));
    let output_str = output_path.to_string_lossy().to_string();

    axonc::codegen::llvm::compile_and_link(&ir, &output_str, axonc::codegen::llvm::OptLevel::O0, false)?;

    // Step 5: Run the executable
    let run_output = Command::new(&output_path)
        .output()
        .map_err(|e| format!("Failed to run {}: {}", output_str, e))?;

    // Cleanup
    let _ = fs::remove_dir_all(&tmp_dir);

    Ok(run_output)
}

/// Helper: compile, run, assert exit code 0.
fn assert_compiles_and_runs(source: &str, test_name: &str) {
    match compile_and_run(source, test_name) {
        Ok(output) => {
            assert!(
                output.status.success(),
                "{} exited with {:?}\nstdout: {}\nstderr: {}",
                test_name,
                output.status.code(),
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr),
            );
        }
        Err(e) => panic!("{} failed: {}", test_name, e),
    }
}

/// Helper: load an .axon file from tests/e2e/ directory.
fn load_e2e(name: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("e2e")
        .join(format!("{}.axon", name));
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("Could not read {}: {}", path.display(), e))
}

// ── Individual E2E tests ─────────────────────────────────────────────

#[test]
fn e2e_empty_main() {
    assert_compiles_and_runs(&load_e2e("empty_main"), "empty_main");
}

#[test]
fn e2e_function_call() {
    assert_compiles_and_runs(&load_e2e("function_call"), "function_call");
}

#[test]
fn e2e_if_else() {
    assert_compiles_and_runs(&load_e2e("if_else"), "if_else");
}

#[test]
fn e2e_while_loop() {
    assert_compiles_and_runs(&load_e2e("while_loop"), "while_loop");
}

#[test]
fn e2e_fibonacci() {
    assert_compiles_and_runs(&load_e2e("fibonacci"), "fibonacci");
}

#[test]
fn e2e_multi_function() {
    assert_compiles_and_runs(&load_e2e("multi_function"), "multi_function");
}

#[test]
fn e2e_arithmetic() {
    assert_compiles_and_runs(&load_e2e("arithmetic"), "arithmetic");
}

#[test]
fn e2e_nested_if() {
    assert_compiles_and_runs(&load_e2e("nested_if"), "nested_if");
}

#[test]
fn e2e_let_binding() {
    assert_compiles_and_runs(&load_e2e("let_binding"), "let_binding");
}

#[test]
fn e2e_composed_calls() {
    assert_compiles_and_runs(&load_e2e("composed_calls"), "composed_calls");
}

#[test]
fn e2e_mutual_recursion() {
    assert_compiles_and_runs(&load_e2e("mutual_recursion"), "mutual_recursion");
}

// ── Inline source tests ─────────────────────────────────────────────

#[test]
fn e2e_inline_simple_return() {
    let src = r#"
fn main() {
    let x: Int64 = 42;
}
"#;
    assert_compiles_and_runs(src, "inline_simple_return");
}

#[test]
fn e2e_inline_chain_arithmetic() {
    let src = r#"
fn main() {
    let a: Int64 = 1;
    let b: Int64 = 2;
    let c: Int64 = 3;
    let d: Int64 = a + b + c;
}
"#;
    assert_compiles_and_runs(src, "inline_chain_arithmetic");
}

#[test]
fn e2e_inline_bool_logic() {
    let src = r#"
fn main() {
    let a: Bool = true;
    let b: Bool = false;
}
"#;
    assert_compiles_and_runs(src, "inline_bool_logic");
}

#[test]
fn e2e_inline_nested_calls() {
    let src = r#"
fn inc(x: Int64) -> Int64 {
    return x + 1;
}

fn main() {
    let x: Int64 = inc(inc(inc(0)));
}
"#;
    assert_compiles_and_runs(src, "inline_nested_calls");
}
