// tests/snapshot_tests.rs — Snapshot tests for the Axon compiler (E5)
//
// Uses `insta` for deterministic output verification across all compiler phases.

use axonc::error::CompileError;
use axonc::lint::{LintWarning, Linter};
use axonc::mir::MirBuilder;

// ═══════════════════════════════════════════════════════════════
// Helpers
// ═══════════════════════════════════════════════════════════════

fn parse_snap(src: &str) -> String {
    let (prog, errors) = axonc::parse_source(src, "test.axon");
    if !errors.is_empty() {
        return format_errors(&errors);
    }
    axonc::ast_to_json(&prog)
}

fn check_snap(src: &str) -> String {
    let (typed, errors) = axonc::check_source(src, "test.axon");
    if !errors.is_empty() {
        return format_errors(&errors);
    }
    axonc::tast_to_json(&typed)
}

fn error_snap(src: &str) -> String {
    let (_typed, errors) = axonc::check_source(src, "test.axon");
    format_errors(&errors)
}

fn mir_snap(src: &str) -> String {
    let (typed, errors, checker) = axonc::check_source_full(src, "test.axon");
    if !errors.is_empty() {
        return format_errors(&errors);
    }
    let mut builder = MirBuilder::new(&checker.interner);
    let program = builder.build(&typed);
    format!("{}", program)
}

fn fmt_snap(src: &str) -> String {
    match axonc::fmt::Formatter::format(src, "test.axon") {
        Ok(formatted) => formatted,
        Err(errors) => format_errors(&errors),
    }
}

fn lint_snap(src: &str) -> String {
    let warnings = Linter::lint(src, "test.axon");
    format_lint_warnings(&warnings)
}

fn format_errors(errors: &[CompileError]) -> String {
    errors
        .iter()
        .map(|e| format!("[{}] {}", e.error_code, e.message))
        .collect::<Vec<_>>()
        .join("\n")
}

fn format_lint_warnings(warnings: &[LintWarning]) -> String {
    if warnings.is_empty() {
        return "No warnings".to_string();
    }
    warnings
        .iter()
        .map(|w| format!("[{}] {}", w.code, w.message))
        .collect::<Vec<_>>()
        .join("\n")
}

// ═══════════════════════════════════════════════════════════════
// Parse Snapshots
// ═══════════════════════════════════════════════════════════════

#[test]
fn snap_parse_hello_world() {
    let src = r#"
fn main() {
    println("Hello, world!");
}
"#;
    insta::assert_snapshot!(parse_snap(src));
}

#[test]
fn snap_parse_function_with_params() {
    let src = r#"
fn add(a: Int64, b: Int64): Int64 {
    a + b
}
"#;
    insta::assert_snapshot!(parse_snap(src));
}

#[test]
fn snap_parse_model_declaration() {
    let src = r#"
model Point {
    x: Float64,
    y: Float64,
}
"#;
    insta::assert_snapshot!(parse_snap(src));
}

#[test]
fn snap_parse_enum_declaration() {
    let src = r#"
enum Color {
    Red,
    Green,
    Blue,
    Custom(Int64, Int64, Int64),
}
"#;
    insta::assert_snapshot!(parse_snap(src));
}

#[test]
fn snap_parse_extend_block() {
    let src = r#"
model Rect { width: Float64, height: Float64 }

extend Rect {
    fn area(self): Float64 {
        self.width * self.height
    }
}
"#;
    insta::assert_snapshot!(parse_snap(src));
}

#[test]
fn snap_parse_match_expression() {
    let src = r#"
fn describe(x: Int64): String {
    match x {
        0 => "zero",
        1 => "one",
        _ => "other",
    }
}
"#;
    insta::assert_snapshot!(parse_snap(src));
}

#[test]
fn snap_parse_closure() {
    let src = r#"
fn apply(f: fn(Int64): Int64, x: Int64): Int64 {
    f(x)
}

fn main() {
    val result = apply(|x| x * 2, 5);
}
"#;
    insta::assert_snapshot!(parse_snap(src));
}

#[test]
fn snap_parse_pipe_operator() {
    let src = r#"
fn double(x: Int64): Int64 { x * 2 }
fn inc(x: Int64): Int64 { x + 1 }

fn main() {
    val result = 5 |> double() |> inc();
}
"#;
    insta::assert_snapshot!(parse_snap(src));
}

#[test]
fn snap_parse_val_var_bindings() {
    let src = r#"
fn main() {
    val x: Int64 = 10;
    var y: Int64 = 20;
    y = y + x;
}
"#;
    insta::assert_snapshot!(parse_snap(src));
}

#[test]
fn snap_parse_tensor_type() {
    let src = r#"
fn matmul(a: Tensor[Float32, 2, 3], b: Tensor[Float32, 3, 4]): Tensor[Float32, 2, 4] {
    a @ b
}
"#;
    insta::assert_snapshot!(parse_snap(src));
}

#[test]
fn snap_parse_generic_function() {
    let src = r#"
fn identity<T>(x: T): T {
    x
}
"#;
    insta::assert_snapshot!(parse_snap(src));
}

#[test]
fn snap_parse_if_else() {
    let src = r#"
fn max(a: Int64, b: Int64): Int64 {
    if a > b { a } else { b }
}
"#;
    insta::assert_snapshot!(parse_snap(src));
}

#[test]
fn snap_parse_while_loop() {
    let src = r#"
fn countdown(n: Int64) {
    var i = n;
    while i > 0 {
        i = i - 1;
    }
}
"#;
    insta::assert_snapshot!(parse_snap(src));
}

#[test]
fn snap_parse_for_loop() {
    let src = r#"
fn sum_range() {
    var total: Int64 = 0;
    for i in 0..10 {
        total = total + i;
    }
}
"#;
    insta::assert_snapshot!(parse_snap(src));
}

// ═══════════════════════════════════════════════════════════════
// TypeCheck Snapshots
// ═══════════════════════════════════════════════════════════════

#[test]
fn snap_check_simple_function() {
    let src = r#"
fn add(a: Int64, b: Int64): Int64 {
    a + b
}
"#;
    insta::assert_snapshot!(check_snap(src));
}

#[test]
fn snap_check_variable_binding() {
    let src = r#"
fn main() {
    val x: Int64 = 42;
    val y: Float64 = 3.14;
}
"#;
    insta::assert_snapshot!(check_snap(src));
}

#[test]
fn snap_check_bool_expression() {
    let src = r#"
fn is_positive(x: Int64): Bool {
    x > 0
}
"#;
    insta::assert_snapshot!(check_snap(src));
}

// ═══════════════════════════════════════════════════════════════
// Error Snapshots
// ═══════════════════════════════════════════════════════════════

#[test]
fn snap_error_undefined_variable() {
    let src = r#"
fn main() {
    val x: Int64 = y;
}
"#;
    insta::assert_snapshot!(error_snap(src));
}

#[test]
fn snap_error_type_mismatch() {
    let src = r#"
fn main() {
    val x: Int64 = "hello";
}
"#;
    insta::assert_snapshot!(error_snap(src));
}

#[test]
fn snap_error_wrong_arg_count() {
    let src = r#"
fn add(a: Int64, b: Int64): Int64 { a + b }

fn main() {
    val x = add(1);
}
"#;
    insta::assert_snapshot!(error_snap(src));
}

#[test]
fn snap_error_duplicate_definition() {
    let src = r#"
fn foo() {}
fn foo() {}
"#;
    insta::assert_snapshot!(error_snap(src));
}

#[test]
fn snap_error_parse_unexpected_token() {
    let src = r#"
fn main() {
    val x = ;
}
"#;
    insta::assert_snapshot!(parse_snap(src));
}

#[test]
fn snap_error_missing_return_type() {
    let src = r#"
fn add(a: Int64, b: Int64) {
    a + b
}

fn main() {
    val x: Int64 = add(1, 2);
}
"#;
    insta::assert_snapshot!(error_snap(src));
}

#[test]
fn snap_error_undefined_type() {
    let src = r#"
fn main() {
    val x: FooBar = 42;
}
"#;
    insta::assert_snapshot!(error_snap(src));
}

#[test]
fn snap_error_mismatched_if_arms() {
    let src = r#"
fn main() {
    val x = if true { 1 } else { "hello" };
}
"#;
    insta::assert_snapshot!(error_snap(src));
}

// ═══════════════════════════════════════════════════════════════
// MIR Snapshots
// ═══════════════════════════════════════════════════════════════

#[test]
fn snap_mir_empty_main() {
    let src = r#"
fn main() {
}
"#;
    insta::assert_snapshot!(mir_snap(src));
}

#[test]
fn snap_mir_return_value() {
    let src = r#"
fn square(x: Int64): Int64 {
    x * x
}

fn main() {
    val y = square(5);
}
"#;
    insta::assert_snapshot!(mir_snap(src));
}

#[test]
fn snap_mir_if_else() {
    let src = r#"
fn abs(x: Int64): Int64 {
    if x > 0 { x } else { 0 - x }
}

fn main() {
    val y = abs(-5);
}
"#;
    insta::assert_snapshot!(mir_snap(src));
}

#[test]
fn snap_mir_while_loop() {
    let src = r#"
fn count() {
    var i: Int64 = 0;
    while i < 10 {
        i = i + 1;
    }
}

fn main() {
    count();
}
"#;
    insta::assert_snapshot!(mir_snap(src));
}

#[test]
fn snap_mir_binary_ops() {
    let src = r#"
fn math(a: Int64, b: Int64): Int64 {
    val sum = a + b;
    val prod = a * b;
    sum + prod
}

fn main() {
    val r = math(3, 4);
}
"#;
    insta::assert_snapshot!(mir_snap(src));
}

// ═══════════════════════════════════════════════════════════════
// Format Snapshots
// ═══════════════════════════════════════════════════════════════

#[test]
fn snap_fmt_function() {
    let src = "fn   add( a :  Int64 , b:Int64 ) : Int64 { a+b }";
    insta::assert_snapshot!(fmt_snap(src));
}

#[test]
fn snap_fmt_model() {
    let src = "model Point{x:Float64,y:Float64}";
    insta::assert_snapshot!(fmt_snap(src));
}

#[test]
fn snap_fmt_val_var() {
    let src = r#"
fn main(){val x:Int64=10;var y:Int64=20;y=y+x;}
"#;
    insta::assert_snapshot!(fmt_snap(src));
}

// ═══════════════════════════════════════════════════════════════
// Lint Snapshots
// ═══════════════════════════════════════════════════════════════

#[test]
fn snap_lint_unused_variable() {
    let src = r#"
fn main() {
    val unused_var: Int64 = 42;
}
"#;
    insta::assert_snapshot!(lint_snap(src));
}

#[test]
fn snap_lint_naming_convention() {
    let src = r#"
fn BadName() {
}
"#;
    insta::assert_snapshot!(lint_snap(src));
}

#[test]
fn snap_lint_clean_code() {
    let src = r#"
fn main() {
    println("hello");
}
"#;
    insta::assert_snapshot!(lint_snap(src));
}
