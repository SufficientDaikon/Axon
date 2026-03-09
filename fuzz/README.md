# Axon Fuzz Testing

## Overview

Fuzz testing helps ensure the Axon compiler never panics, crashes, or produces
undefined behavior on arbitrary (including invalid) input. The compiler should
always produce a well-formed error message rather than crashing.

## Lightweight Fuzz Tests

The primary fuzz tests live in `tests/fuzz_tests.rs` and run as part of
the normal `cargo test` suite. They exercise the compiler with:

- Empty and minimal inputs
- All ASCII single characters
- Keywords in isolation
- Malformed syntax (unclosed braces, parens, strings)
- Extremely long identifiers
- Deeply nested structures
- Invalid type references
- Randomized valid-looking programs

Run them with:

```bash
cargo test --test fuzz_tests -- --nocapture
```

## cargo-fuzz Targets (Future)

The `fuzz/targets/` directory contains descriptions of fuzz targets for use
with [cargo-fuzz](https://github.com/rust-fuzz/cargo-fuzz) when integrated:

| Target           | Description                                     |
| ---------------- | ----------------------------------------------- |
| `lexer_fuzz`     | Feed arbitrary bytes to the lexer               |
| `parser_fuzz`    | Feed arbitrary bytes through lex → parse        |
| `typeck_fuzz`    | Full lex → parse → typecheck on arbitrary input |
| `formatter_fuzz` | Feed arbitrary bytes to the formatter           |
| `linter_fuzz`    | Feed arbitrary bytes to the linter              |

### Setting up cargo-fuzz (future)

```bash
cargo install cargo-fuzz
cargo fuzz init
# Copy target templates from fuzz/targets/ into fuzz/fuzz_targets/
cargo fuzz run lexer_fuzz -- -max_len=10000 -max_total_time=300
```

## Property-Based Testing (Future)

For structured fuzz testing, we plan to integrate `proptest` or `quickcheck`
to generate syntactically plausible Axon programs and verify:

1. **No panics** — compiler never crashes
2. **Determinism** — same input always produces same output
3. **Idempotence** — formatting twice produces same result
4. **Round-trip** — parse → format → parse produces equivalent AST

## Reporting Issues

If fuzz testing discovers a crash, create a minimal reproducer and add it
as a regression test in `tests/fuzz_tests.rs`.
