//! Criterion benchmarks for the Axon compiler.
//!
//! Run with: cargo bench
//!
//! These benchmarks use criterion.rs for statistically rigorous measurements
//! with warmup, multiple iterations, and outlier detection.

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};

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

fn bench_lexer(c: &mut Criterion) {
    let source = generate_source(100);

    c.bench_function("lexer/100_functions", |b| {
        b.iter(|| {
            let mut lexer = axonc::lexer::Lexer::new(black_box(&source), "bench.axon");
            let _tokens = lexer.tokenize();
        })
    });

    let source_small = generate_source(10);
    c.bench_function("lexer/10_functions", |b| {
        b.iter(|| {
            let mut lexer = axonc::lexer::Lexer::new(black_box(&source_small), "bench.axon");
            let _tokens = lexer.tokenize();
        })
    });
}

fn bench_parser(c: &mut Criterion) {
    let source = generate_source(100);

    c.bench_function("parser/100_functions", |b| {
        b.iter(|| {
            let _ = axonc::parse_source(black_box(&source), "bench.axon");
        })
    });
}

fn bench_typechecker(c: &mut Criterion) {
    let source = generate_source(100);

    c.bench_function("typechecker/100_functions", |b| {
        b.iter(|| {
            let _ = axonc::typeck::check(black_box(&source), "bench.axon");
        })
    });
}

fn bench_full_pipeline(c: &mut Criterion) {
    let source = generate_source(100);

    c.bench_function("full_pipeline/100_functions", |b| {
        b.iter(|| {
            let _ = axonc::check_source(black_box(&source), "bench.axon");
        })
    });
}

fn bench_formatter(c: &mut Criterion) {
    let source = generate_source(100);

    c.bench_function("formatter/100_functions", |b| {
        b.iter(|| {
            let _ = axonc::fmt::Formatter::format(black_box(&source), "bench.axon");
        })
    });
}

fn bench_linter(c: &mut Criterion) {
    let source = generate_source(100);

    c.bench_function("linter/100_functions", |b| {
        b.iter(|| {
            let _ = axonc::lint::Linter::lint(black_box(&source), "bench.axon");
        })
    });
}

fn bench_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("scaling/lex");
    for size in [10, 50, 100, 500].iter() {
        let source = generate_source(*size);
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| {
                let mut lexer = axonc::lexer::Lexer::new(black_box(&source), "bench.axon");
                let _tokens = lexer.tokenize();
            })
        });
    }
    group.finish();

    let mut group = c.benchmark_group("scaling/parse");
    for size in [10, 50, 100, 500].iter() {
        let source = generate_source(*size);
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| {
                let _ = axonc::parse_source(black_box(&source), "bench.axon");
            })
        });
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_lexer,
    bench_parser,
    bench_typechecker,
    bench_full_pipeline,
    bench_formatter,
    bench_linter,
    bench_scaling,
);
criterion_main!(benches);
