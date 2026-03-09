# Contributing to Axon

Thank you for your interest in contributing to the Axon programming language!
This guide covers how to build from source, run tests, and submit changes.

---

## Getting Started

### Prerequisites

- **Rust** (stable, 1.75+) — [rustup.rs](https://rustup.rs/)
- **Git**
- **Clang** (for the native binary backend) — optional for most development

### Clone and Build

```bash
git clone https://github.com/axon-lang/axon.git
cd axon
cargo build
```

Verify it works:

```bash
cargo run -- --help
cargo run -- lex tests/examples/example1_hello.axon
```

### Project Structure

```
axon/
├── Cargo.toml              # Rust project manifest
├── src/
│   ├── main.rs             # CLI entry point (axonc)
│   ├── lib.rs              # Library root — compiler pipeline
│   ├── token.rs            # Token types
│   ├── lexer.rs            # Lexer (source → tokens)
│   ├── ast.rs              # AST node definitions
│   ├── parser.rs           # Parser (tokens → AST)
│   ├── span.rs             # Source location tracking
│   ├── error.rs            # Error types and reporting
│   ├── types.rs            # Type system (Type, TypeInterner)
│   ├── symbol.rs           # Symbol table and name resolution
│   ├── typeck.rs           # Type checker (HM inference)
│   ├── shapes.rs           # Shape checker (tensor dims)
│   ├── borrow.rs           # Borrow checker (ownership)
│   ├── tast.rs             # Typed AST
│   ├── mir.rs              # Mid-level IR
│   ├── codegen/
│   │   ├── llvm.rs         # LLVM IR generation
│   │   ├── mlir.rs         # MLIR / GPU backend
│   │   ├── runtime.rs      # Runtime library
│   │   └── abi.rs          # ABI and symbol mangling
│   ├── stdlib/             # Standard library definitions
│   │   ├── prelude.rs      # Auto-imported items
│   │   ├── ops.rs          # Operator traits
│   │   ├── collections.rs  # Vec, HashMap, Option, Result
│   │   ├── tensor.rs       # Tensor operations
│   │   ├── nn.rs           # Neural network layers
│   │   ├── autograd.rs     # Automatic differentiation
│   │   ├── optim.rs        # Optimizers
│   │   ├── loss.rs         # Loss functions
│   │   └── ...             # More stdlib modules
│   ├── fmt.rs              # Code formatter
│   ├── lint.rs             # Linter
│   ├── doc.rs              # Documentation generator
│   ├── repl.rs             # REPL
│   ├── lsp/                # Language server
│   │   └── handlers.rs     # LSP request handlers
│   └── pkg/                # Package manager
│       ├── manifest.rs     # Axon.toml parsing
│       ├── resolver.rs     # Dependency resolution
│       └── commands.rs     # CLI commands
├── stdlib/                 # Axon source stubs (.axon files)
├── tests/
│   ├── integration_tests.rs
│   ├── type_tests.rs
│   ├── codegen_tests.rs
│   ├── stdlib_tests.rs
│   ├── ai_framework_tests.rs
│   ├── tooling_tests.rs
│   └── examples/*.axon     # Example programs
├── editors/
│   └── vscode/             # VS Code extension
├── benches/                # Benchmarks
├── fuzz/                   # Fuzz testing
└── docs/                   # Documentation
```

---

## Running Tests

### Full Test Suite

```bash
cargo test
```

This runs 863+ tests across all compiler phases.

### Specific Test Files

```bash
# Lexer and parser tests
cargo test --lib lexer
cargo test --lib parser

# Type checker tests
cargo test --test type_tests

# Code generation tests
cargo test --test codegen_tests

# Standard library tests
cargo test --test stdlib_tests

# AI framework tests
cargo test --test ai_framework_tests

# Tooling tests (LSP, formatter, linter, REPL)
cargo test --test tooling_tests
```

### Running a Single Test

```bash
cargo test test_name_here -- --exact
```

### Running Benchmarks

```bash
cargo test --test compiler_bench -- --ignored
```

---

## Development Workflow

### 1. Create a Branch

```bash
git checkout -b feature/my-feature
```

### 2. Make Changes

Edit the relevant source files. The compiler pipeline flows:

```
Source → Lexer → Parser → AST
                           ↓
                    Name Resolution
                           ↓
                    Type Checker → Shape Checker → Borrow Checker
                           ↓
                        Typed AST
                           ↓
                          MIR
                           ↓
                    LLVM IR / MLIR
                           ↓
                      Native Binary
```

### 3. Add Tests

Every change should include tests. Add them to the appropriate test file:

- **Lexer/Parser changes**: `src/lexer.rs` or `src/parser.rs` (unit tests)
- **Type system changes**: `tests/type_tests.rs`
- **Codegen changes**: `tests/codegen_tests.rs`
- **Stdlib additions**: `tests/stdlib_tests.rs`
- **Tooling changes**: `tests/tooling_tests.rs`

### 4. Run Tests

```bash
cargo test
```

Ensure all tests pass before submitting.

### 5. Format and Lint

```bash
cargo fmt
cargo clippy
```

### 6. Submit a Pull Request

Push your branch and open a PR. Include:

- **Description** of what the change does
- **Related issue** number (if any)
- **Test output** confirming tests pass

---

## Coding Guidelines

### Style

- Follow Rust standard style (`cargo fmt`)
- Use descriptive variable names
- Add doc comments (`///`) for public items
- Keep functions focused and under 50 lines when possible

### Error Handling

- Use proper error codes (see [Compiler Errors](../reference/compiler-errors.md))
- Include source locations in all errors
- Add suggestions where helpful
- Test both success and error cases

### Testing

- Each feature should have positive and negative tests
- Test edge cases (empty input, deeply nested structures, etc.)
- Integration tests should use `.axon` example files
- Aim for test names that describe what they verify

---

## Adding a New Feature

### Adding a New Keyword

1. Add the keyword to `Token` enum in `src/token.rs`
2. Add it to the keyword map in `src/lexer.rs`
3. Add parser support in `src/parser.rs`
4. Add AST node in `src/ast.rs`
5. Add type checking in `src/typeck.rs`
6. Add tests at each level
7. Update documentation

### Adding a Stdlib Function

1. Add the function signature in `src/stdlib/<module>.rs`
2. Register it in the type checker (`src/typeck.rs`)
3. Add an Axon stub in `stdlib/<module>.axon`
4. Add tests in `tests/stdlib_tests.rs`
5. Update documentation

### Adding a New Lint Rule

1. Add the warning code to `src/lint.rs`
2. Implement detection logic
3. Add tests in `tests/tooling_tests.rs`
4. Document in `docs/reference/compiler-errors.md`

---

## Architecture Overview

For detailed architecture documentation, see
[`docs/internals/architecture.md`](architecture.md).

### Key Design Principles

1. **Correctness first** — the compiler should never accept invalid programs
2. **Helpful errors** — every error should explain what went wrong and suggest a fix
3. **Performance** — the compiler should be fast (targeting <100ms for typical files)
4. **Testability** — every component should be independently testable

---

## Communication

- **Issues**: Report bugs and request features via GitHub Issues
- **Discussions**: Design discussions in GitHub Discussions
- **Code Review**: All changes require at least one review

---

## License

Axon is open source. By contributing, you agree that your contributions
will be licensed under the same license as the project.

---

## See Also

- [Architecture](architecture.md) — compiler internals
- [CLI Reference](../reference/cli-reference.md) — command reference
- [Task Tracking](../../tasks.md) — project task list
