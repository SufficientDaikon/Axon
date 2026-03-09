//! Compiler benchmarks for lexer, parser, type checker, and full pipeline.
//! Run with: cargo test --test compiler_bench -- --nocapture

use std::time::Instant;

/// Generate a large Axon source file for benchmarking.
fn generate_source(num_functions: usize) -> String {
    let mut source = String::new();
    for i in 0..num_functions {
        source.push_str(&format!(
            "fn func_{}(x: Int32, y: Int32) -> Int32 {{\n    let result = x + y * {};\n    return result;\n}}\n\n",
            i, i
        ));
    }
    source.push_str("fn main() -> Int32 { return func_0(1, 2); }\n");
    source
}

// ═══════════════════════════════════════════════════════════════
// Lexer benchmarks
// ═══════════════════════════════════════════════════════════════

#[test]
fn bench_lexer_throughput() {
    let source = generate_source(1000);
    let start = Instant::now();
    let iterations = 10;
    for _ in 0..iterations {
        let mut lexer = axonc::lexer::Lexer::new(&source, "bench.axon");
        let _tokens = lexer.tokenize();
    }
    let elapsed = start.elapsed();
    // ~20 tokens per function * 1000 functions = ~20,000 tokens per iteration
    let approx_tokens_per_iter = 1000 * 20;
    let total_tokens = approx_tokens_per_iter * iterations;
    let tokens_per_sec = total_tokens as f64 / elapsed.as_secs_f64();
    println!(
        "Lexer: {:.0} tokens/sec ({:.2}ms per iteration)",
        tokens_per_sec,
        elapsed.as_millis() as f64 / iterations as f64
    );
    assert!(elapsed.as_millis() < 10000, "Lexer too slow: {}ms", elapsed.as_millis());
}

#[test]
fn bench_lexer_small_file() {
    let source = generate_source(10);
    let start = Instant::now();
    let iterations = 1000;
    for _ in 0..iterations {
        let mut lexer = axonc::lexer::Lexer::new(&source, "bench.axon");
        let _tokens = lexer.tokenize();
    }
    let elapsed = start.elapsed();
    println!(
        "Lexer (small): {:.2}ms total for {} iterations ({:.4}ms each)",
        elapsed.as_millis(),
        iterations,
        elapsed.as_secs_f64() * 1000.0 / iterations as f64
    );
    assert!(elapsed.as_millis() < 5000, "Lexer (small) too slow");
}

// ═══════════════════════════════════════════════════════════════
// Parser benchmarks
// ═══════════════════════════════════════════════════════════════

#[test]
fn bench_parser_throughput() {
    let source = generate_source(1000);
    let start = Instant::now();
    let iterations = 10;
    for _ in 0..iterations {
        let (_program, _errors) = axonc::parse_source(&source, "bench.axon");
    }
    let elapsed = start.elapsed();
    println!(
        "Parser (lex+parse): {:.2}ms per iteration ({} funcs)",
        elapsed.as_secs_f64() * 1000.0 / iterations as f64,
        1000
    );
    assert!(elapsed.as_millis() < 15000, "Parser too slow: {}ms", elapsed.as_millis());
}

#[test]
fn bench_parser_small_file() {
    let source = generate_source(10);
    let start = Instant::now();
    let iterations = 1000;
    for _ in 0..iterations {
        let (_program, _errors) = axonc::parse_source(&source, "bench.axon");
    }
    let elapsed = start.elapsed();
    println!(
        "Parser (small): {:.2}ms total for {} iterations ({:.4}ms each)",
        elapsed.as_millis(),
        iterations,
        elapsed.as_secs_f64() * 1000.0 / iterations as f64
    );
    assert!(elapsed.as_millis() < 5000, "Parser (small) too slow");
}

// ═══════════════════════════════════════════════════════════════
// Type checker benchmarks
// ═══════════════════════════════════════════════════════════════

#[test]
fn bench_type_checker_speed() {
    let source = generate_source(500);
    let start = Instant::now();
    let iterations = 5;
    for _ in 0..iterations {
        let (_checker, _errors) = axonc::typeck::check(&source, "bench.axon");
    }
    let elapsed = start.elapsed();
    println!(
        "TypeChecker: {:.2}ms per iteration ({} funcs)",
        elapsed.as_secs_f64() * 1000.0 / iterations as f64,
        500
    );
    assert!(elapsed.as_millis() < 30000, "TypeChecker too slow: {}ms", elapsed.as_millis());
}

// ═══════════════════════════════════════════════════════════════
// Full pipeline benchmarks
// ═══════════════════════════════════════════════════════════════

#[test]
fn bench_full_pipeline() {
    let source = generate_source(500);
    let start = Instant::now();
    let iterations = 5;
    for _ in 0..iterations {
        let (_typed_program, _errors) = axonc::check_source(&source, "bench.axon");
    }
    let elapsed = start.elapsed();
    println!(
        "Full pipeline (lex+parse+typecheck+borrow): {:.2}ms per iteration ({} funcs)",
        elapsed.as_secs_f64() * 1000.0 / iterations as f64,
        500
    );
    assert!(elapsed.as_millis() < 60000, "Full pipeline too slow: {}ms", elapsed.as_millis());
}

// ═══════════════════════════════════════════════════════════════
// Formatter benchmarks
// ═══════════════════════════════════════════════════════════════

#[test]
fn bench_formatter_speed() {
    let source = generate_source(500);
    let start = Instant::now();
    let iterations = 5;
    for _ in 0..iterations {
        let _ = axonc::fmt::Formatter::format(&source, "bench.axon");
    }
    let elapsed = start.elapsed();
    println!(
        "Formatter: {:.2}ms per iteration ({} funcs)",
        elapsed.as_secs_f64() * 1000.0 / iterations as f64,
        500
    );
    assert!(elapsed.as_millis() < 30000, "Formatter too slow: {}ms", elapsed.as_millis());
}

// ═══════════════════════════════════════════════════════════════
// Linter benchmarks
// ═══════════════════════════════════════════════════════════════

#[test]
fn bench_linter_speed() {
    let source = generate_source(500);
    let start = Instant::now();
    let iterations = 5;
    for _ in 0..iterations {
        let _warnings = axonc::lint::Linter::lint(&source, "bench.axon");
    }
    let elapsed = start.elapsed();
    println!(
        "Linter: {:.2}ms per iteration ({} funcs)",
        elapsed.as_secs_f64() * 1000.0 / iterations as f64,
        500
    );
    assert!(elapsed.as_millis() < 30000, "Linter too slow: {}ms", elapsed.as_millis());
}

// ═══════════════════════════════════════════════════════════════
// Memory usage estimation
// ═══════════════════════════════════════════════════════════════

#[test]
fn bench_memory_estimate() {
    let source = generate_source(1000);
    let (checker, _errors) = axonc::typeck::check(&source, "bench.axon");
    let type_count = checker.interner.len();
    println!("Types interned: {}", type_count);
    println!("Estimated type interner memory: ~{} bytes", type_count * std::mem::size_of::<usize>() * 4);
    println!("Source size: {} bytes ({} chars)", source.len(), source.chars().count());
    // Sanity check: we should have at least the stdlib types + some per-function types
    assert!(type_count > 10, "Expected more types interned, got {}", type_count);
}

// ═══════════════════════════════════════════════════════════════
// Scaling benchmark
// ═══════════════════════════════════════════════════════════════

#[test]
fn bench_scaling() {
    let sizes = [10, 50, 100, 500];
    println!("\n{:<10} {:<15} {:<15} {:<15}", "Functions", "Lex (ms)", "Parse (ms)", "TypeCheck (ms)");
    println!("{}", "-".repeat(55));
    for &n in &sizes {
        let source = generate_source(n);

        let start = Instant::now();
        let mut lexer = axonc::lexer::Lexer::new(&source, "bench.axon");
        let _tokens = lexer.tokenize();
        let lex_ms = start.elapsed().as_secs_f64() * 1000.0;

        let start = Instant::now();
        let _ = axonc::parse_source(&source, "bench.axon");
        let parse_ms = start.elapsed().as_secs_f64() * 1000.0;

        let start = Instant::now();
        let _ = axonc::typeck::check(&source, "bench.axon");
        let typecheck_ms = start.elapsed().as_secs_f64() * 1000.0;

        println!("{:<10} {:<15.3} {:<15.3} {:<15.3}", n, lex_ms, parse_ms, typecheck_ms);
    }
}
