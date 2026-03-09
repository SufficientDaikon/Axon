# Testing Strategy

This document describes the Axon compiler's testing and CI strategy.

## CI Pipeline (`.github/workflows/ci.yml`)

### Current Jobs

| Job | What it does | Runs on |
|-----|-------------|---------|
| **lint** | `cargo fmt --check` + `cargo clippy -D warnings` | ubuntu |
| **test** | `cargo test --all` (debug + release) | ubuntu, macos, windows |
| **benchmark** | Criterion benchmarks + legacy test benchmarks | ubuntu |
| **audit** | `cargo audit` — checks for known vulnerabilities | ubuntu |
| **docs** | `cargo doc --no-deps` — ensures docs build | ubuntu |
| **release** | Cross-platform release builds | all platforms |

### Test Tiers

1. **Unit Tests** (`cargo test --lib`)
   - Lexer, parser, type checker, MIR builder, codegen, formatter, linter
   - Fast (~5 seconds), run on every commit

2. **Integration Tests** (`cargo test --tests`)
   - `tests/e2e_tests.rs` — Full pipeline: source → binary → execute
   - `tests/codegen_tests.rs` — Source → LLVM IR verification
   - `tests/type_tests.rs` — Type system edge cases
   - `tests/stdlib_tests.rs` — Standard library functions
   - `tests/tooling_tests.rs` — Formatter, linter, doc generator
   - `tests/fuzz_tests.rs` — Lightweight randomized fuzzing

3. **Benchmarks** (`cargo bench`)
   - Criterion benchmarks in `benches/criterion_bench.rs`
   - Legacy throughput benchmarks in `benches/compiler_bench.rs`
   - Measures lexer, parser, typechecker, formatter, linter performance

4. **Fuzz Testing** (`cargo +nightly fuzz run <target>`)
   - `fuzz/fuzz_targets/fuzz_parser.rs` — Parser robustness
   - `fuzz/fuzz_targets/fuzz_typeck.rs` — Full frontend robustness
   - Run manually or in dedicated fuzzing CI (not in main pipeline)

## Planned Additions (Phase 15)

### Miri — Undefined Behavior Detection

[Miri](https://github.com/rust-lang/miri) interprets MIR and detects:
- Use-after-free
- Out-of-bounds memory access
- Use of uninitialized memory
- Invalid pointer arithmetic
- Data races

```bash
rustup +nightly component add miri
cargo +nightly miri test
```

**Status**: Planned. Requires nightly Rust. Will be added as a separate CI job.

### Address Sanitizer (ASan)

Detects memory errors at runtime:
- Buffer overflows
- Use-after-free
- Memory leaks

```bash
RUSTFLAGS="-Zsanitizer=address" cargo +nightly test --target x86_64-unknown-linux-gnu
```

**Status**: Planned. Requires nightly Rust and Linux.

### Thread Sanitizer (TSan)

Detects data races in concurrent code:

```bash
RUSTFLAGS="-Zsanitizer=thread" cargo +nightly test --target x86_64-unknown-linux-gnu
```

**Status**: Planned. Will be relevant when async/parallel compilation is added.

### cargo-audit

Checks dependencies for known security vulnerabilities.

```bash
cargo install cargo-audit
cargo audit
```

**Status**: ✅ Active — runs in CI as advisory (non-blocking) job.

## Running Tests Locally

```bash
# All tests
cargo test --all

# Specific test suite
cargo test --test e2e_tests
cargo test --test codegen_tests

# Unit tests only
cargo test --lib

# With output
cargo test -- --nocapture

# Benchmarks
cargo bench

# Legacy benchmarks (run as tests with output)
cargo test --test compiler_bench -- --nocapture
```
