//! End-to-end compilation tests for the Axon compiler.
//!
//! Each test compiles an `.axon` source file to a native executable,
//! runs it, and checks the exit code (0 = success).

use std::fs;
use std::path::PathBuf;
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
    let ir = codegen.generate(&mir_program).map_err(|e| format!("Codegen error: {}", e))?;

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

/// Helper: compile, run, assert exit code 0, and verify stdout contains expected text.
fn assert_compiles_and_outputs(source: &str, test_name: &str, expected_stdout: &str) {
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
            // Normalize line endings for cross-platform comparison
            let stdout = String::from_utf8_lossy(&output.stdout)
                .replace("\r\n", "\n");
            let expected = expected_stdout.replace("\r\n", "\n");
            assert_eq!(
                stdout.trim(),
                expected.trim(),
                "{}: stdout mismatch.\nExpected: {:?}\nActual:   {:?}",
                test_name,
                expected.trim(),
                stdout.trim(),
            );
        }
        Err(e) => panic!("{} failed: {}", test_name, e),
    }
}

/// Parse `// expect: <text>` comments from source and join them with newlines.
fn extract_expected_output(source: &str) -> String {
    source
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            if trimmed.starts_with("// expect: ") {
                Some(trimmed.strip_prefix("// expect: ").unwrap().to_string())
            } else {
                None
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Helper: attempt to compile source and assert it FAILS.
/// Optionally checks that the error message contains `expected_error_substring`.
fn assert_compile_fails(source: &str, test_name: &str, expected_error_substring: Option<&str>) {
    match compile_and_run(source, test_name) {
        Ok(output) => {
            // Some errors only manifest as non-zero exit codes (e.g. linker errors
            // that still produce a binary but it crashes on startup).  Accept that too.
            if output.status.success() {
                panic!(
                    "{} was expected to fail compilation, but it compiled and ran successfully \
                     (exit code: {:?})\nstdout: {}\nstderr: {}",
                    test_name,
                    output.status.code(),
                    String::from_utf8_lossy(&output.stdout),
                    String::from_utf8_lossy(&output.stderr),
                );
            }
        }
        Err(e) => {
            // Compilation or linking failed — this is the expected path.
            if let Some(substr) = expected_error_substring {
                assert!(
                    e.to_lowercase().contains(&substr.to_lowercase()),
                    "{}: expected error containing {:?}, but got:\n{}",
                    test_name,
                    substr,
                    e,
                );
            }
        }
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

/// Helper: load an .axon file, extract `// expect:` lines, compile, run, and
/// verify stdout matches the expected output.  Reduces per-test boilerplate.
fn assert_e2e_output(name: &str) {
    let src = load_e2e(name);
    let expected = extract_expected_output(&src);
    assert_compiles_and_outputs(&src, name, &expected);
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

// ── Print/Println E2E tests ─────────────────────────────────────────

#[test]
fn e2e_print_int() {
    let src = &load_e2e("print_int");
    let expected = extract_expected_output(src);
    assert_compiles_and_outputs(src, "print_int", &expected);
}

#[test]
fn e2e_println_bool() {
    let src = &load_e2e("println_bool");
    let expected = extract_expected_output(src);
    assert_compiles_and_outputs(src, "println_bool", &expected);
}

#[test]
fn e2e_print_concat() {
    let src = &load_e2e("print_concat");
    let expected = extract_expected_output(src);
    assert_compiles_and_outputs(src, "print_concat", &expected);
}

#[test]
fn e2e_print_float() {
    let src = &load_e2e("print_float");
    let expected = extract_expected_output(src);
    assert_compiles_and_outputs(src, "print_float", &expected);
}

#[test]
fn e2e_println_multiple() {
    let src = &load_e2e("println_multiple");
    let expected = extract_expected_output(src);
    assert_compiles_and_outputs(src, "println_multiple", &expected);
}

#[test]
fn e2e_inline_print_42() {
    let src = "fn main() { print(42); }";
    assert_compiles_and_outputs(src, "inline_print_42", "42");
}

#[test]
fn e2e_inline_println_true() {
    let src = "fn main() { println(true); }";
    assert_compiles_and_outputs(src, "inline_println_true", "true");
}

#[test]
fn e2e_inline_print_34() {
    let src = r#"
fn main() {
    print(3);
    print(4);
}
"#;
    assert_compiles_and_outputs(src, "inline_print_34", "34");
}

// ── New stdout-verification E2E tests ───────────────────────────────

#[test]
fn e2e_hello_world() {
    assert_e2e_output("hello_world");
}

#[test]
fn e2e_print_bool() {
    assert_e2e_output("print_bool");
}

#[test]
fn e2e_arithmetic_output() {
    assert_e2e_output("arithmetic_output");
}

#[test]
fn e2e_fib_output() {
    assert_e2e_output("fib_output");
}

#[test]
fn e2e_while_output() {
    assert_e2e_output("while_output");
}

#[test]
fn e2e_nested_calls_output() {
    assert_e2e_output("nested_calls_output");
}

// ── New Phase 8-9 E2E tests ─────────────────────────────────────────

#[test]
fn e2e_structs() {
    assert_e2e_output("structs");
}

#[test]
fn e2e_type_casts() {
    assert_e2e_output("type_casts");
}

#[test]
fn e2e_comparisons() {
    assert_e2e_output("comparisons");
}

#[test]
fn e2e_unary_ops() {
    assert_e2e_output("unary_ops");
}

#[test]
fn e2e_enums() {
    assert_e2e_output("enums");
}

#[test]
fn e2e_tuples() {
    assert_e2e_output("tuples");
}

// ── Compile-failure E2E tests ───────────────────────────────────────

#[test]
fn e2e_error_missing_main() {
    let src = load_e2e("error_missing_main");
    // A program without `fn main()` should fail with E5009 error
    // now caught at codegen stage before linking.
    assert_compile_fails(&src, "error_missing_main", Some("E5009"));
}
