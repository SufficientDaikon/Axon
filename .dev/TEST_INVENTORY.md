# Test Inventory

> All paths relative to `h:/programing language/axon/`

## Running Tests

| Command | What It Runs |
|---------|-------------|
| `cargo test` | All tests (~1,034) |
| `cargo test --lib` | Unit tests in src/ only (478) |
| `cargo test --test e2e_tests` | E2E compile-and-run tests |
| `cargo test --test codegen_tests` | Codegen + MIR assertion tests |
| `cargo test --test integration_tests` | Integration tests |
| `cargo test --test type_tests` | Type system tests |
| `cargo test test_name` | Single test by name |

## E2E Test Structure

**Harness**: `tests/e2e_tests.rs`
**Pipeline**: `compile_and_run()` does: parse → typecheck → MIR → LLVM IR → clang → execute binary
**Pattern**: `.axon` files contain `// expect: <text>` comments; harness collects them and asserts stdout matches

### Helpers
- `assert_compiles_and_runs(src)` — checks exit code 0
- `assert_compiles_and_outputs(src, expected)` — checks stdout
- `assert_e2e_output(name)` — loads `tests/e2e/{name}.axon`, extracts `// expect:` lines, compiles+runs
- `assert_compile_fails(src, error_substr)` — asserts compilation fails
- `load_e2e(name)` — reads `tests/e2e/{name}.axon`

## E2E Files Using `->` (return type syntax)

**11 files** — must update if changing `->`:

| File | `->` Count | Example |
|------|-----------|---------|
| `composed_calls.axon` | 2 | `fn my_abs(x: Int64) -> Int64` |
| `enums.axon` | 1 | `fn classify(n: Int64) -> Int64` |
| `error_missing_main.axon` | 1 | `fn helper(x: Int64) -> Int64` |
| `fibonacci.axon` | 1 | `fn fib(n: Int64) -> Int64` |
| `fib_output.axon` | 1 | `fn fib(n: Int64) -> Int64` |
| `function_call.axon` | 1 | `fn add(a: Int64, b: Int64) -> Int64` |
| `multi_function.axon` | 2 | `fn square(x: Int64) -> Int64` |
| `mutual_recursion.axon` | 2 | `fn is_even(n: Int64) -> Bool` |
| `nested_calls_output.axon` | 2 | `fn my_abs(x: Int64) -> Int64` |
| `structs.axon` | 1 | `fn make_pair(...) -> Int64` |
| `tuples.axon` | 1 | `fn sum_and_diff(...) -> Int64` |

## E2E Files Using `::` (path separator)

**0 files** — no E2E tests use `::`

## Example Files (tests/examples/) Syntax Usage

| File | Has `::` | Has `->` |
|------|----------|----------|
| `example1_hello.axon` | no | no |
| `example2_tensors.axon` | no | no |
| `example3_ownership.axon` | no | yes |
| `example4_neural_net.axon` | **yes** (`NeuralNet::new()`) | yes |
| `example5_training.axon` | no | yes |
| `example6_device.axon` | no | no |
| `example7_errors.axon` | **yes** (`File::open()`) | yes |
| `example8_generics.axon` | no | yes |
| `broken.axon` | no | no |

## Inline Rust Tests with Axon Syntax Strings

These `src/` files have `#[cfg(test)]` modules containing Axon source as string literals.
**Must grep for old syntax when making changes:**

| File | Test Module Line | Contains `->` | Contains `::` |
|------|-----------------|---------------|---------------|
| `src/parser.rs` | ~2056 | yes (many) | yes (`"use std::io::File;"`) |
| `src/typeck.rs` | ~1724 | yes (many) | possible |
| `src/tast.rs` | ~755 | yes | possible |
| `src/mir.rs` | ~1769 | yes | yes (path joins) |
| `src/fmt.rs` | ~812 | yes (many) | yes (formatter output) |
| `src/doc.rs` | ~563 | yes | yes (doc output) |
| `src/lexer.rs` | ~717 | yes (token test) | yes (token test) |
| `src/lint.rs` | ~595 | yes | possible |
| `src/codegen/llvm.rs` | ~2234 | yes | possible |
| `src/codegen/abi.rs` | ~109 | no | yes (mangling) |

## Codegen Tests (tests/codegen_tests.rs)

~44 `#[test]` functions using two helpers:
- `full_pipeline(src)` → returns LLVM IR text string
- `to_mir(src)` → returns MIR text string

Tests assert on IR/MIR text (e.g., `ir.contains("add")`), NOT by executing.

## Unit Test Counts by Module

49 files with `#[cfg(test)]`, total ~478 `#[test]` functions. Largest:
- `src/codegen/llvm.rs`: 25 tests
- `src/codegen/abi.rs`: 13 tests
- `src/codegen/runtime.rs`: 10 tests
- `src/lsp/handlers.rs`: ~15 tests
- `src/stdlib/collections.rs`: ~20 tests

## Error Code Registry

| Range | Module | File | Examples |
|-------|--------|------|---------|
| E0001-E0007 | Lexer | `src/lexer.rs` | E0001: unexpected char, E0004: unterminated string |
| E0100-E0103 | Parser | `src/parser.rs` | E0100: expected token, E0101: expected identifier |
| E1002 | Symbols | `src/symbol.rs` | E1002: duplicate definition |
| E2001-E2015 | TypeChecker | `src/typeck.rs` | E2001: type mismatch, E2005: undefined function |
| E4001-E4007 | BorrowCheck | `src/borrow.rs` | E4001: use after move |
| W5001-W5008 | Lint | `src/lint.rs` | Warnings |
