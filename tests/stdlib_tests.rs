// tests/stdlib_tests.rs — Integration tests for the Axon standard library (type checker).
//
// Verifies that stdlib-registered free functions, constants, and type names
// are correctly resolved and type-checked through `typeck::check`.

use axonc::typeck;
use axonc::error::CompileError;

// ═══════════════════════════════════════════════════════════════
// Helpers
// ═══════════════════════════════════════════════════════════════

fn check_ok(source: &str) {
    let (_, errors) = typeck::check(source, "test.axon");
    assert!(
        errors.is_empty(),
        "Expected no errors, got: {:?}",
        errors.iter().map(|e| &e.message).collect::<Vec<_>>()
    );
}

fn check_err(source: &str) -> Vec<CompileError> {
    let (_, errors) = typeck::check(source, "test.axon");
    assert!(!errors.is_empty(), "Expected errors but got none");
    errors
}

fn check_has_error_code(source: &str, code: &str) -> Vec<CompileError> {
    let (_, errors) = typeck::check(source, "test.axon");
    assert!(
        errors.iter().any(|e| e.error_code == code),
        "Expected error code {}, got: {:?}",
        code,
        errors.iter().map(|e| format!("{}: {}", e.error_code, e.message)).collect::<Vec<_>>()
    );
    errors
}

// ═══════════════════════════════════════════════════════════════
// 1. Prelude functions (T165)
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_println_accepted() {
    check_ok(r#"fn main() { println("hello"); }"#);
}

#[test]
fn test_print_accepted() {
    check_ok(r#"fn main() { print("world"); }"#);
}

#[test]
fn test_eprintln_accepted() {
    check_ok(r#"fn main() { eprintln("error"); }"#);
}

#[test]
fn test_assert_accepted() {
    check_ok("fn main() { assert(true); }");
}

#[test]
fn test_panic_accepted() {
    check_ok(r#"fn main() { panic("oh no"); }"#);
}

#[test]
fn test_unreachable_accepted() {
    check_ok("fn main() { unreachable(); }");
}

#[test]
fn test_todo_accepted() {
    check_ok("fn main() { todo(); }");
}

#[test]
fn test_unimplemented_accepted() {
    check_ok("fn main() { unimplemented(); }");
}

#[test]
fn test_dbg_accepted() {
    check_ok(r#"fn main() { dbg("value"); }"#);
}

// ═══════════════════════════════════════════════════════════════
// 2. Math functions (T167)
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_sin() {
    check_ok("fn main() -> Float64 { return sin(1.0); }");
}

#[test]
fn test_cos() {
    check_ok("fn main() -> Float64 { return cos(1.0); }");
}

#[test]
fn test_tan() {
    check_ok("fn main() -> Float64 { return tan(1.0); }");
}

#[test]
fn test_asin() {
    check_ok("fn main() -> Float64 { return asin(0.5); }");
}

#[test]
fn test_acos() {
    check_ok("fn main() -> Float64 { return acos(0.5); }");
}

#[test]
fn test_atan() {
    check_ok("fn main() -> Float64 { return atan(1.0); }");
}

#[test]
fn test_sqrt() {
    check_ok("fn main() -> Float64 { return sqrt(4.0); }");
}

#[test]
fn test_exp() {
    check_ok("fn main() -> Float64 { return exp(1.0); }");
}

#[test]
fn test_log() {
    check_ok("fn main() -> Float64 { return log(2.7); }");
}

#[test]
fn test_log2() {
    check_ok("fn main() -> Float64 { return log2(8.0); }");
}

#[test]
fn test_log10() {
    check_ok("fn main() -> Float64 { return log10(100.0); }");
}

#[test]
fn test_abs() {
    check_ok("fn main() -> Float64 { return abs(1.0); }");
}

#[test]
fn test_pow() {
    check_ok("fn main() -> Float64 { return pow(2.0, 3.0); }");
}

#[test]
fn test_cbrt() {
    check_ok("fn main() -> Float64 { return cbrt(27.0); }");
}

#[test]
fn test_floor() {
    check_ok("fn main() -> Float64 { return floor(3.7); }");
}

#[test]
fn test_ceil() {
    check_ok("fn main() -> Float64 { return ceil(3.2); }");
}

#[test]
fn test_round() {
    check_ok("fn main() -> Float64 { return round(3.5); }");
}

#[test]
fn test_trunc() {
    check_ok("fn main() -> Float64 { return trunc(3.9); }");
}

#[test]
fn test_min() {
    check_ok("fn main() -> Float64 { return min(1.0, 2.0); }");
}

#[test]
fn test_max() {
    check_ok("fn main() -> Float64 { return max(1.0, 2.0); }");
}

#[test]
fn test_clamp() {
    check_ok("fn main() -> Float64 { return clamp(5.0, 0.0, 3.0); }");
}

#[test]
fn test_atan2() {
    check_ok("fn main() -> Float64 { return atan2(1.0, 1.0); }");
}

#[test]
fn test_signum() {
    check_ok("fn main() -> Float64 { return signum(1.0); }");
}

#[test]
fn test_hypot() {
    check_ok("fn main() -> Float64 { return hypot(3.0, 4.0); }");
}

#[test]
fn test_fract() {
    check_ok("fn main() -> Float64 { return fract(3.7); }");
}

#[test]
fn test_is_nan() {
    check_ok("fn main() -> Bool { return is_nan(0.0); }");
}

#[test]
fn test_is_inf() {
    check_ok("fn main() -> Bool { return is_inf(1.0); }");
}

#[test]
fn test_is_finite() {
    check_ok("fn main() -> Bool { return is_finite(1.0); }");
}

// ═══════════════════════════════════════════════════════════════
// 3. Math constants
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_pi_constant() {
    check_ok("fn main() -> Float64 { return PI; }");
}

#[test]
fn test_e_constant() {
    check_ok("fn main() -> Float64 { return E; }");
}

#[test]
fn test_tau_constant() {
    check_ok("fn main() -> Float64 { return TAU; }");
}

#[test]
fn test_max_int_constant() {
    check_ok("fn main() -> Int64 { return MAX_INT; }");
}

// ═══════════════════════════════════════════════════════════════
// 4. Memory & utility functions (T166)
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_size_of() {
    check_ok("fn main() -> Int64 { return size_of(); }");
}

#[test]
fn test_align_of() {
    check_ok("fn main() -> Int64 { return align_of(); }");
}

// ═══════════════════════════════════════════════════════════════
// 5. Random functions
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_random() {
    check_ok("fn main() -> Float64 { return random(); }");
}

#[test]
fn test_random_range() {
    check_ok("fn main() -> Float64 { return random_range(0.0, 1.0); }");
}

#[test]
fn test_random_int() {
    check_ok("fn main() -> Int64 { return random_int(0, 100); }");
}

#[test]
fn test_seed() {
    check_ok("fn main() { seed(42); }");
}

#[test]
fn test_random_bool() {
    check_ok("fn main() -> Bool { return random_bool(0.5); }");
}

// ═══════════════════════════════════════════════════════════════
// 6. I/O functions
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_format_fn() {
    check_ok(r#"fn main() -> String { return format("hello {}"); }"#);
}

// ═══════════════════════════════════════════════════════════════
// 7. Thread functions
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_sleep() {
    check_ok("fn main() { sleep(1000); }");
}

#[test]
fn test_yield_now() {
    check_ok("fn main() { yield_now(); }");
}

#[test]
fn test_current_thread_id() {
    check_ok("fn main() -> Int64 { return current_thread_id(); }");
}

#[test]
fn test_available_parallelism() {
    check_ok("fn main() -> Int64 { return available_parallelism(); }");
}

// ═══════════════════════════════════════════════════════════════
// 8. Data functions
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_read_csv() {
    check_ok(r#"fn main() { read_csv("data.csv"); }"#);
}

#[test]
fn test_read_json() {
    check_ok(r#"fn main() { read_json("data.json"); }"#);
}

#[test]
fn test_parse_json() {
    check_ok(r#"fn main() { parse_json("{\"key\": 1}"); }"#);
}

// ═══════════════════════════════════════════════════════════════
// 9. Negative tests — wrong argument types
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_println_wrong_arg_type() {
    // println now accepts any printable type (Int64, Float64, Bool, String)
    // Previously it only accepted String, but the new print/println builtins
    // dispatch to type-specific runtime functions.
    check_ok("fn main() { println(42); }");
}

#[test]
fn test_print_wrong_arg_type() {
    // print now accepts any printable type (Int64, Float64, Bool, String)
    check_ok("fn main() { print(true); }");
}

#[test]
fn test_assert_wrong_arg_type() {
    // assert expects Bool, "hello" is String
    check_err(r#"fn main() { assert("hello"); }"#);
}

#[test]
fn test_panic_wrong_arg_type() {
    // panic expects String, 42 is Int64
    check_err("fn main() { panic(42); }");
}

#[test]
fn test_sin_wrong_type() {
    // sin expects Float64, true is Bool
    check_err("fn main() { sin(true); }");
}

// ═══════════════════════════════════════════════════════════════
// 10. Negative tests — wrong arity
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_sin_wrong_arity() {
    // sin expects 1 arg, got 2
    check_has_error_code("fn main() { sin(1.0, 2.0); }", "E2003");
}

#[test]
fn test_min_wrong_arity() {
    // min expects 2 args, got 1
    check_has_error_code("fn main() { min(1.0); }", "E2003");
}

#[test]
fn test_todo_wrong_arity() {
    // todo expects 0 args, got 1
    check_has_error_code(r#"fn main() { todo("not yet"); }"#, "E2003");
}

#[test]
fn test_clamp_wrong_arity() {
    // clamp expects 3 args, got 2
    check_has_error_code("fn main() { clamp(1.0, 2.0); }", "E2003");
}

#[test]
fn test_random_wrong_arity() {
    // random expects 0 args, got 1
    check_has_error_code("fn main() { random(42); }", "E2003");
}

#[test]
fn test_yield_now_wrong_arity() {
    // yield_now expects 0 args, got 1
    check_has_error_code("fn main() { yield_now(1); }", "E2003");
}

// ═══════════════════════════════════════════════════════════════
// 11. Return type verification
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_sin_returns_float64() {
    // sin returns Float64, but function declares Int64 return
    check_err("fn main() -> Int64 { return sin(1.0); }");
}

#[test]
fn test_random_returns_float64() {
    // random returns Float64, but function declares Bool return
    check_err("fn main() -> Bool { return random(); }");
}

#[test]
fn test_current_thread_id_returns_int64() {
    // current_thread_id returns Int64, but function declares String return
    check_err(r#"fn main() -> String { return current_thread_id(); }"#);
}
