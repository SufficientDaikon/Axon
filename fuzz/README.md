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

## cargo-fuzz / libFuzzer Targets

The `fuzz/fuzz_targets/` directory contains real cargo-fuzz targets powered by
libFuzzer. These provide coverage-guided fuzzing for much deeper exploration.

### Available Targets

| Target         | Description                                     |
| -------------- | ----------------------------------------------- |
| `fuzz_parser`  | Feed arbitrary bytes to the lexer + parser      |
| `fuzz_typeck`  | Full lex → parse → typecheck on arbitrary input |

### Setup & Running

```bash
# Install cargo-fuzz (requires nightly Rust)
rustup install nightly
cargo +nightly install cargo-fuzz

# List available targets
cargo +nightly fuzz list

# Run a target (from the repo root)
cargo +nightly fuzz run fuzz_parser -- -max_len=10000 -max_total_time=300
cargo +nightly fuzz run fuzz_typeck -- -max_len=10000 -max_total_time=300
```

### Directory Structure

```
fuzz/
├── Cargo.toml              # cargo-fuzz package manifest
├── README.md               # This file
├── fuzz_targets/
│   ├── fuzz_parser.rs      # Parser fuzz target (libFuzzer)
│   └── fuzz_typeck.rs      # Type checker fuzz target (libFuzzer)
└── targets/                # Legacy target descriptions
    ├── lexer_fuzz.txt
    ├── parser_fuzz.txt
    ├── typeck_fuzz.txt
    ├── formatter_fuzz.txt
    └── linter_fuzz.txt
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
