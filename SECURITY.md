# Security Policy

## Overview

The Axon compiler (`axonc`) processes **untrusted source code input**. As a language compiler, it accepts arbitrary user-written programs and must handle malformed, adversarial, or pathological inputs safely without crashing, leaking memory, or producing undefined behavior in the compiler itself.

## Threat Model

| Threat | Mitigation |
|--------|------------|
| Malformed source input | Parser rejects invalid syntax with structured error messages |
| Pathological nesting / recursion | Parser limits nesting depth; stack overflow handled gracefully |
| Type-system abuse | Type checker prevents unsafe operations; unification has occurs-check |
| Memory safety violations in user code | Borrow checker enforces ownership rules at compile time |
| Compiler crash on fuzz input | Fuzz testing infrastructure validates robustness (see below) |
| Malicious package dependencies | Package manager validates manifests and checksums |

## Input Validation Strategy

1. **Lexer** — Rejects invalid tokens, unterminated strings, and malformed numeric literals with clear error messages.
2. **Parser** — Validates syntax against the Axon grammar. Malformed input produces structured `CompileError` diagnostics rather than panics.
3. **Type Checker** — Prevents type-unsafe operations through static analysis. Enforces type unification with occurs-check to prevent infinite types.
4. **Borrow Checker** — Enforces Rust-inspired ownership and borrowing rules to prevent use-after-move, double-free, and data race conditions in user programs.
5. **Shape Checker** — Validates tensor dimension compatibility at compile time, preventing shape mismatch errors that would occur at runtime.

## Fuzz Testing Infrastructure

The Axon compiler includes fuzzing targets in the `fuzz/` directory:

- **Parser fuzzer** (`fuzz/fuzz_targets/fuzz_parser.rs`) — Feeds arbitrary byte sequences to the parser to ensure it never panics on any input.
- **Type checker fuzzer** (`fuzz/fuzz_targets/fuzz_typeck.rs`) — Feeds arbitrary source to the full compilation pipeline (parse → type-check → borrow-check) to ensure robustness.

These fuzz targets are designed for use with `cargo-fuzz` (libFuzzer backend) and can be run with:

```bash
cargo fuzz run fuzz_parser
cargo fuzz run fuzz_typeck
```

## Secure Development Practices

- All compiler errors use structured types (`CompileError`) rather than string formatting, preventing information leakage.
- The compiler never executes user code during compilation (execution happens only in the REPL's type-evaluation mode, which does not evaluate at runtime).
- LLVM code generation produces well-formed IR; the LLVM backend provides additional validation.
- CI runs `cargo clippy` and `cargo test` on every pull request.

## Supported Versions

| Version | Supported |
|---------|-----------|
| 0.1.x   | ✅ Current development |

## Reporting Security Issues

If you discover a security vulnerability in the Axon compiler, please report it responsibly:

1. **Do not** open a public GitHub issue for security vulnerabilities.
2. **Email**: Send a detailed report to the project maintainers with:
   - Description of the vulnerability
   - Steps to reproduce (ideally a minimal `.axon` source file)
   - Expected vs. actual behavior
   - Impact assessment
3. **Response time**: We aim to acknowledge reports within 48 hours and provide a fix or mitigation within 7 days for critical issues.

## Security-Related Configuration

The Axon compiler does not require any special security configuration. It operates as a command-line tool with no network access, no file system writes (except for output files explicitly requested by the user), and no execution of user code during compilation.
