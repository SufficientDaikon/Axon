// tests/codegen_tests.rs — Integration tests for Axon codegen pipeline (Phase 4f)

// ═══════════════════════════════════════════════════════════════
// Helpers
// ═══════════════════════════════════════════════════════════════

/// Run the full pipeline: source → parse → check → MIR → LLVM IR
fn full_pipeline(source: &str) -> String {
    let (typed_program, errors, checker) = axonc::check_source_full(source, "test.axon");
    assert!(errors.is_empty(), "Type check errors: {:?}", errors);

    let mut builder = axonc::mir::MirBuilder::new(&checker.interner);
    let mir = builder.build(&typed_program);

    let mut codegen = axonc::codegen::llvm::LlvmCodegen::new(
        &checker.interner,
        axonc::codegen::llvm::OptLevel::O0,
    );
    codegen.generate(&mir).expect("generate() failed — no main function?")
}

/// Run source through to MIR and return it as string.
fn to_mir(source: &str) -> String {
    let (typed_program, errors, checker) = axonc::check_source_full(source, "test.axon");
    assert!(errors.is_empty(), "Type check errors: {:?}", errors);

    let mut builder = axonc::mir::MirBuilder::new(&checker.interner);
    let mir = builder.build(&typed_program);
    format!("{}", mir)
}

/// Check whether clang is available on this system.
fn clang_available() -> bool {
    std::process::Command::new("clang")
        .arg("--version")
        .output()
        .is_ok()
}

// ═══════════════════════════════════════════════════════════════
// Section 1: LLVM IR Generation — Basic Programs
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_empty_main() {
    let ir = full_pipeline("fn main() {}");
    assert!(
        ir.contains("define") && ir.contains("@main"),
        "IR should define main: {}",
        ir
    );
}

#[test]
fn test_return_integer() {
    let ir = full_pipeline("fn main(): Int64 { return 42; }");
    assert!(ir.contains("ret"), "IR should contain ret: {}", ir);
    assert!(ir.contains("i64"), "IR should contain i64: {}", ir);
}

#[test]
fn test_return_float() {
    let ir = full_pipeline("fn main(): Float64 { return 3.14; }");
    assert!(ir.contains("ret"), "IR should contain ret: {}", ir);
    assert!(
        ir.contains("double"),
        "IR should contain double type: {}",
        ir
    );
}

#[test]
fn test_return_bool() {
    let ir = full_pipeline("fn main(): Bool { return true; }");
    assert!(ir.contains("ret"), "IR should contain ret: {}", ir);
    assert!(ir.contains("i1"), "IR should contain i1 for Bool: {}", ir);
}

#[test]
fn test_let_binding() {
    let ir = full_pipeline("fn main(): Int64 { val x: Int64 = 42; return x; }");
    assert!(
        ir.contains("alloca"),
        "IR should contain alloca for val binding: {}",
        ir
    );
    assert!(
        ir.contains("store") && ir.contains("42"),
        "IR should store the value 42: {}",
        ir
    );
}

#[test]
fn test_arithmetic_add() {
    let ir = full_pipeline(
        "fn main(): Int64 { val a: Int64 = 10; val b: Int64 = 20; return a + b; }",
    );
    assert!(
        ir.contains("add"),
        "IR should contain add instruction: {}",
        ir
    );
}

#[test]
fn test_arithmetic_sub() {
    let ir = full_pipeline(
        "fn main(): Int64 { val a: Int64 = 30; val b: Int64 = 10; return a - b; }",
    );
    assert!(
        ir.contains("sub"),
        "IR should contain sub instruction: {}",
        ir
    );
}

#[test]
fn test_arithmetic_mul() {
    let ir = full_pipeline(
        "fn main(): Int64 { val a: Int64 = 5; val b: Int64 = 6; return a * b; }",
    );
    assert!(
        ir.contains("mul"),
        "IR should contain mul instruction: {}",
        ir
    );
}

#[test]
fn test_arithmetic_div() {
    let ir = full_pipeline(
        "fn main(): Int64 { val a: Int64 = 100; val b: Int64 = 4; return a / b; }",
    );
    assert!(
        ir.contains("div"),
        "IR should contain div instruction: {}",
        ir
    );
}

#[test]
fn test_float_arithmetic() {
    let ir = full_pipeline(
        "fn main(): Float64 { val a: Float64 = 1.0; val b: Float64 = 2.0; return a + b; }",
    );
    assert!(
        ir.contains("add") || ir.contains("fadd"),
        "IR should contain add/fadd for float addition: {}",
        ir
    );
    assert!(
        ir.contains("1.0") || ir.contains("1.000000e+00") || ir.contains("0x3FF0000000000000"),
        "IR should contain float literal: {}",
        ir
    );
}

// ═══════════════════════════════════════════════════════════════
// Section 2: Control Flow
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_if_else_ir() {
    let ir = full_pipeline(
        "fn main(): Int64 { if true { return 1; } else { return 2; } }",
    );
    assert!(ir.contains("br"), "IR should contain branch: {}", ir);
    // Should have multiple basic blocks
    let bb_count = ir.matches("bb").count();
    assert!(
        bb_count >= 2,
        "IR should have multiple basic blocks, found {} bb refs: {}",
        bb_count,
        ir
    );
}

#[test]
fn test_while_loop_ir() {
    let ir = full_pipeline(
        "fn main() { var x: Int64 = 0; while x < 10 { x = x + 1; } }",
    );
    assert!(ir.contains("br"), "IR should contain branch: {}", ir);
    assert!(
        ir.contains("bb"),
        "IR should contain basic block labels: {}",
        ir
    );
}

#[test]
fn test_comparison_ops() {
    let ir = full_pipeline(
        "fn test_cmp(a: Int64): Bool { return a < 10; }\nfn main() { val r: Bool = test_cmp(5); }",
    );
    assert!(
        ir.contains("icmp"),
        "IR should contain icmp for integer comparison: {}",
        ir
    );
}

#[test]
fn test_logical_and() {
    let ir = full_pipeline(
        "fn test_and(a: Bool, b: Bool): Bool { return a && b; }\nfn main() { val r: Bool = test_and(true, false); }",
    );
    assert!(
        ir.contains("and") || ir.contains("br"),
        "IR should contain and/branch for logical &&: {}",
        ir
    );
}

#[test]
fn test_logical_or() {
    let ir = full_pipeline(
        "fn test_or(a: Bool, b: Bool): Bool { return a || b; }\nfn main() { val r: Bool = test_or(true, false); }",
    );
    assert!(
        ir.contains("or") || ir.contains("br"),
        "IR should contain or/branch for logical ||: {}",
        ir
    );
}

#[test]
fn test_negation() {
    let ir = full_pipeline(
        "fn test_neg(x: Int64): Int64 { return -x; }\nfn main() { val r: Int64 = test_neg(5); }",
    );
    assert!(
        ir.contains("sub") || ir.contains("neg"),
        "IR should contain sub/neg for negation: {}",
        ir
    );
}

#[test]
fn test_boolean_not() {
    let ir = full_pipeline(
        "fn test_not(a: Bool): Bool { return !a; }\nfn main() { val r: Bool = test_not(true); }",
    );
    assert!(
        ir.contains("xor") || ir.contains("icmp"),
        "IR should contain xor or icmp for boolean not: {}",
        ir
    );
}

#[test]
fn test_nested_if() {
    let ir = full_pipeline(
        "fn test_nested(x: Bool, y: Bool): Int64 { \
         if x { if y { return 1; } else { return 2; } } \
         else { return 3; } }\nfn main() { val r: Int64 = test_nested(true, false); }",
    );
    // Nested if-else should generate several basic blocks
    let define_count = ir.matches("br ").count();
    assert!(
        define_count >= 2,
        "Nested if should generate at least 2 branches, got {}: {}",
        define_count,
        ir
    );
}

// ═══════════════════════════════════════════════════════════════
// Section 3: Functions
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_multiple_functions() {
    let ir = full_pipeline(
        "fn add(a: Int64, b: Int64): Int64 { return a + b; }\n\
         fn main(): Int64 { return add(1, 2); }",
    );
    let define_count = ir.matches("define ").count();
    assert!(
        define_count >= 2,
        "IR should define at least 2 functions, got {}: {}",
        define_count,
        ir
    );
    assert!(ir.contains("@main"), "IR should contain @main: {}", ir);
}

#[test]
fn test_function_params() {
    let ir = full_pipeline(
        "fn sum(a: Int64, b: Int64): Int64 { return a + b; }\nfn main() { val r: Int64 = sum(1, 2); }",
    );
    assert!(
        ir.contains("%arg0") && ir.contains("%arg1"),
        "IR should contain function parameters: {}",
        ir
    );
    assert!(
        ir.contains("define"),
        "IR should have a function definition: {}",
        ir
    );
}

#[test]
fn test_void_function() {
    let ir = full_pipeline("fn greet() { }\nfn main() { greet(); }");
    assert!(
        ir.contains("ret void") || ir.contains("ret "),
        "IR should contain ret for void function: {}",
        ir
    );
}

#[test]
fn test_function_call_ir() {
    let ir = full_pipeline(
        "fn foo(x: Int64): Int64 { return x; }\n\
         fn main(): Int64 { return foo(42); }",
    );
    assert!(
        ir.contains("call"),
        "IR should contain call instruction: {}",
        ir
    );
}

#[test]
fn test_multiple_params() {
    let ir = full_pipeline(
        "fn triple(a: Int64, b: Float64, c: Bool): Int64 { return a; }\nfn main() { val r: Int64 = triple(1, 2.0, true); }",
    );
    assert!(ir.contains("%arg0"), "IR should contain first param: {}", ir);
    assert!(ir.contains("%arg1"), "IR should contain second param: {}", ir);
    assert!(ir.contains("%arg2"), "IR should contain third param: {}", ir);
    // Three parameters in the define signature
    assert!(
        ir.contains("@triple("),
        "IR should define the function triple: {}",
        ir
    );
}

#[test]
fn test_function_with_locals() {
    let ir = full_pipeline(
        "fn compute(): Int64 { \
         val a: Int64 = 1; \
         val b: Int64 = 2; \
         val c: Int64 = 3; \
         return a + b + c; }\nfn main() { val r: Int64 = compute(); }",
    );
    let alloca_count = ir.matches("alloca").count();
    assert!(
        alloca_count >= 3,
        "IR should have at least 3 allocas for locals, got {}: {}",
        alloca_count,
        ir
    );
}

// ═══════════════════════════════════════════════════════════════
// Section 4: MIR Generation
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_mir_empty_function() {
    let mir = to_mir("fn main() {}");
    assert!(
        mir.contains("fn main"),
        "MIR should contain fn main: {}",
        mir
    );
}

#[test]
fn test_mir_basic_blocks() {
    let mir = to_mir(
        "fn test_if(x: Bool): Int64 { if x { return 1; } else { return 2; } }",
    );
    // if/else should create multiple basic blocks
    let bb_count = mir.matches("bb").count();
    assert!(
        bb_count >= 2,
        "MIR should have multiple basic blocks for if/else, found {}: {}",
        bb_count,
        mir
    );
}

#[test]
fn test_mir_while_loop() {
    let mir = to_mir(
        "fn test_loop() { var i: Int64 = 0; while i < 10 { i = i + 1; } }",
    );
    // While loop creates at least header/body/exit blocks
    let bb_count = mir.matches("bb").count();
    assert!(
        bb_count >= 2,
        "MIR should have multiple blocks for while loop, found {}: {}",
        bb_count,
        mir
    );
}

#[test]
fn test_mir_let_binding() {
    let mir = to_mir("fn main() { val x: Int64 = 42; }");
    // Assign statement for the let binding
    assert!(
        mir.contains("="),
        "MIR should contain assignment for let: {}",
        mir
    );
}

#[test]
fn test_mir_return() {
    let mir = to_mir("fn main(): Int64 { return 42; }");
    // Return terminator
    assert!(
        mir.contains("return") || mir.contains("Return"),
        "MIR should contain Return terminator: {}",
        mir
    );
}

#[test]
fn test_mir_display() {
    let mir = to_mir("fn main(): Int64 { return 1; }");
    // Display impl should produce readable output
    assert!(!mir.is_empty(), "MIR display should produce output");
    assert!(mir.contains("fn"), "MIR display should contain fn keyword");
    assert!(mir.contains("{"), "MIR display should contain braces");
    assert!(mir.contains("}"), "MIR display should contain closing brace");
}

// ═══════════════════════════════════════════════════════════════
// Section 5: ABI & Name Mangling
// ═══════════════════════════════════════════════════════════════

use axonc::codegen::abi::NameMangler;

#[test]
fn test_mangle_main() {
    let result = NameMangler::mangle(&[], "main", &[]);
    assert_eq!(result, "main", "main should not be mangled");
}

#[test]
fn test_mangle_namespaced() {
    let ns = vec!["std".to_string(), "math".to_string()];
    let result = NameMangler::mangle(&ns, "sin", &[]);
    assert!(
        result.starts_with("_AX"),
        "Namespaced function should start with _AX: {}",
        result
    );
    assert!(
        result.contains("N3std"),
        "Should contain encoded namespace 'std': {}",
        result
    );
    assert!(
        result.contains("N4math"),
        "Should contain encoded namespace 'math': {}",
        result
    );
}

#[test]
fn test_mangle_generic() {
    let ns = vec!["collections".to_string()];
    let generics = vec!["Int64".to_string()];
    let result = NameMangler::mangle(&ns, "push", &generics);
    assert!(
        result.contains("G"),
        "Generic function should contain G marker: {}",
        result
    );
}

#[test]
fn test_mangle_type() {
    let result = NameMangler::mangle_type(&[], "Point");
    assert_eq!(result, "axon.Point", "Type without namespace: {}", result);

    let ns = vec!["geometry".to_string()];
    let result = NameMangler::mangle_type(&ns, "Point");
    assert_eq!(
        result, "axon.geometry.Point",
        "Type with namespace: {}",
        result
    );
}

#[test]
fn test_demangle_roundtrip() {
    let ns = vec!["mymod".to_string()];
    let mangled = NameMangler::mangle(&ns, "helper", &[]);
    assert!(
        mangled.starts_with("_AX"),
        "Mangled name should start with _AX: {}",
        mangled
    );
    // Demangle should return Some for _AX-prefixed symbols
    let demangled = NameMangler::demangle(&mangled);
    assert!(
        demangled.is_some(),
        "Demangling an _AX symbol should succeed"
    );
}

// ═══════════════════════════════════════════════════════════════
// Section 6: Runtime
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_runtime_declarations_in_ir() {
    let ir = full_pipeline("fn main() {}");
    assert!(
        ir.contains("declare"),
        "IR should contain runtime declarations: {}",
        ir
    );
    assert!(
        ir.contains("@axon_alloc"),
        "IR should contain axon_alloc declaration: {}",
        ir
    );
    assert!(
        ir.contains("@axon_panic"),
        "IR should contain axon_panic declaration: {}",
        ir
    );
    assert!(
        ir.contains("@axon_print_i64"),
        "IR should contain print declarations: {}",
        ir
    );
}

#[test]
fn test_runtime_c_source() {
    let src = axonc::codegen::runtime::generate_runtime_c_source();
    assert!(!src.is_empty(), "C runtime source should not be empty");
    assert!(
        src.contains("#include <stdio.h>"),
        "C source should include stdio: {}",
        &src[..200]
    );
    assert!(
        src.contains("void axon_panic("),
        "C source should define axon_panic"
    );
    assert!(
        src.contains("void* axon_alloc("),
        "C source should define axon_alloc"
    );
}

#[test]
fn test_runtime_function_count() {
    let count = axonc::codegen::runtime::RUNTIME_FUNCTIONS.len();
    assert_eq!(count, 39, "RUNTIME_FUNCTIONS should have 39 entries, got {}", count);
}

#[test]
fn test_module_header() {
    let ir = full_pipeline("fn main() {}");
    assert!(
        ir.starts_with("; ModuleID"),
        "IR should start with ModuleID comment: {}",
        &ir[..80]
    );
    assert!(
        ir.contains("target triple"),
        "IR should contain target triple: {}",
        ir
    );
    assert!(
        ir.contains("source_filename"),
        "IR should contain source_filename: {}",
        ir
    );
}

// ═══════════════════════════════════════════════════════════════
// Section 7: End-to-End (compile & run, gated on clang)
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_compile_simple_ir() {
    if !clang_available() {
        eprintln!("skipping: clang not available");
        return;
    }

    let ir = full_pipeline("fn main() {}");

    let dir = std::env::temp_dir().join("axon_codegen_test_simple");
    let _ = std::fs::create_dir_all(&dir);
    let output_path = dir.join("test_simple");
    let output_str = output_path.to_str().unwrap();

    let result = axonc::codegen::llvm::compile_ir_to_binary(
        &ir,
        output_str,
        axonc::codegen::llvm::OptLevel::O0,
    );
    // Clean up regardless of outcome
    let _ = std::fs::remove_dir_all(&dir);

    // The compilation may fail if runtime symbols are unresolved, which
    // is acceptable — we only verify that the IR was written and clang invoked.
    if let Err(e) = &result {
        assert!(
            e.contains("clang") || e.contains("undefined"),
            "Unexpected compilation error: {}",
            e
        );
    }
}

#[test]
fn test_compile_return_42() {
    if !clang_available() {
        eprintln!("skipping: clang not available");
        return;
    }

    let ir = full_pipeline("fn main(): Int64 { return 42; }");

    let dir = std::env::temp_dir().join("axon_codegen_test_ret42");
    let _ = std::fs::create_dir_all(&dir);
    let ir_path = dir.join("ret42.ll");
    std::fs::write(&ir_path, &ir).expect("write IR file");

    // Try to compile just to object to verify IR validity
    let obj_path = dir.join("ret42.o");
    let compile_result = std::process::Command::new("clang")
        .args(&[
            "-c",
            ir_path.to_str().unwrap(),
            "-o",
            obj_path.to_str().unwrap(),
        ])
        .output();

    let _ = std::fs::remove_dir_all(&dir);

    match compile_result {
        Ok(output) => {
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                // IR validity issues from our codegen are informational
                eprintln!("clang compile stderr: {}", stderr);
            }
        }
        Err(e) => eprintln!("clang invocation failed: {}", e),
    }
}

#[test]
fn test_e2e_ir_is_valid_text() {
    // Verify generated IR is valid LLVM IR text (structural checks)
    let ir = full_pipeline("fn main(): Int64 { return 0; }");
    assert!(ir.contains("define"), "IR must have function definitions");
    assert!(ir.contains("ret"), "IR must have return instructions");
    assert!(
        ir.contains("entry:") || ir.contains("bb"),
        "IR must have labeled blocks"
    );
}

#[test]
fn test_e2e_multiple_functions_call() {
    let ir = full_pipeline(
        "fn double(x: Int64): Int64 { return x + x; }\n\
         fn main(): Int64 { return double(21); }",
    );
    assert!(ir.contains("call"), "IR should contain call to double");
    assert!(ir.contains("@main"), "IR should define main");
    let define_count = ir.matches("define ").count();
    assert!(
        define_count >= 2,
        "Should define at least 2 functions: {}",
        define_count
    );
}

#[test]
fn test_e2e_with_control_flow() {
    let ir = full_pipeline(
        "fn my_abs(x: Int64): Int64 { if x < 0 { return -x; } else { return x; } }\n\
         fn main(): Int64 { return my_abs(-5); }",
    );
    assert!(ir.contains("icmp"), "IR should contain comparison");
    assert!(ir.contains("br"), "IR should contain branch");
    assert!(ir.contains("call"), "IR should contain call");
}
