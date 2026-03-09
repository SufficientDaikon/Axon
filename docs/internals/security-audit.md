# Axon Compiler Security Audit

**Date:** 2024  
**Scope:** Axon compiler (`axonc`) crate — all source under `src/`

## 1. `unsafe` Block Inventory

**Finding: No `unsafe` blocks in compiler source code.**

A search of the entire `src/` directory confirms zero uses of `unsafe { }` blocks
in the Axon compiler implementation. The keyword `unsafe` appears only in:

- **Lexer/token** — as a keyword token that Axon can lex (`TokenKind::Unsafe`)
- **Parser** — to parse `unsafe fn` declarations in Axon source
- **LSP** — as a completion/hover entry for the `unsafe` keyword
- **Package manifest** — in test data for lint deny lists

This means the Axon compiler itself relies entirely on Rust's safe subset,
inheriting all of Rust's memory safety guarantees (no buffer overflows,
use-after-free, data races, etc.).

**Risk: None** — Rust's type system and borrow checker enforce safety at compile time.

## 2. FFI Boundaries

### 2.1 Clang Subprocess Invocation

The only external process invocation is in `src/codegen/llvm.rs`:

- `compile_ir_to_binary()` — invokes `clang` via `std::process::Command`
- `compile_ir_to_object()` — invokes `clang` via `std::process::Command`

**Risks:**

- **Command injection:** The output path is passed directly as a command argument.
  If an attacker controls the output path, they could potentially inject arguments.
- **Path traversal:** No validation is performed on the output path.

**Mitigations:**

- Arguments are passed as separate array elements to `Command::args()`, not
  concatenated into a shell string. This prevents shell injection.
- The IR content is written to a file first, not passed via stdin, limiting
  injection vectors.
- Only `clang` is invoked — no shell (`sh -c` / `cmd /c`) is used.

**Recommendations:**

1. Validate/sanitize output paths before passing to `clang`.
2. Use absolute paths for the `clang` binary or verify it on `$PATH`.
3. Consider sandboxing `clang` invocations (e.g., `seccomp`, containers).
4. Add a timeout to prevent hanging `clang` processes.

### 2.2 No Other FFI

The compiler does not use any `extern "C"` functions, does not link to C
libraries, and does not use `libc` or `std::ffi` directly.

## 3. Input Validation

### 3.1 Source Code Parsing

The lexer and parser handle arbitrary input gracefully:

- **Lexer** (`src/lexer.rs`): Processes input character-by-character.
  Unknown characters produce error tokens. Unterminated strings/comments
  produce error tokens with descriptive messages. No panics on any input.

- **Parser** (`src/parser.rs`): Uses error recovery to continue parsing
  after syntax errors. Returns a partial AST plus a list of errors.
  The `parse_source()` function in `lib.rs` is the safe entry point.

- **Type checker** (`src/typeck.rs`): Handles undefined types, recursive
  types, and type mismatches by producing error diagnostics. Falls through
  gracefully when earlier phases produce errors.

**Verification:** The fuzz test suite (`tests/fuzz_tests.rs`) exercises
the compiler with 40+ edge cases including empty input, all ASCII characters,
malformed syntax, deeply nested structures, and more.

### 3.2 Denial of Service via Input

**Potential risks:**

- **Extremely long identifiers:** The lexer allocates a `String` for each
  identifier. A 10GB identifier would consume 10GB of memory.
- **Deeply nested expressions:** The recursive descent parser uses the call
  stack. Extremely deep nesting (>1000 levels) may cause stack overflow.
- **Exponential type inference:** Pathological type constraints could cause
  the unification algorithm to run for a long time.

**Recommendations:**

1. Add configurable limits on identifier length (e.g., 1024 characters).
2. Add a nesting depth limit to the parser (e.g., 256 levels).
3. Add a timeout or iteration limit to the type inference engine.
4. Add a maximum source file size check (e.g., 10MB).

## 4. Package Registry Security Model

The package system (`src/pkg/`) is in early development. Future security
considerations:

### 4.1 Package Integrity

- **Requirement:** All packages must have cryptographic signatures (ed25519).
- **Requirement:** Package contents must be verified against a hash (SHA-256).
- **Requirement:** Lock files must pin exact versions with hashes.

### 4.2 Dependency Resolution

- **Risk:** Dependency confusion attacks (public package overriding private).
- **Mitigation:** Support private registry priorities, namespace scoping.
- **Risk:** Typosquatting.
- **Mitigation:** Name similarity checks during `axon pkg add`.

### 4.3 Build Scripts

- **Risk:** Arbitrary code execution during package installation.
- **Recommendation:** Axon should NOT support arbitrary build scripts.
  Instead, provide a declarative build configuration.

### 4.4 Supply Chain

- **Recommendation:** Support `axon audit` command to check for known
  vulnerabilities in dependencies.
- **Recommendation:** Support reproducible builds.

## 5. REPL Security Considerations

The REPL (`src/repl.rs`) reads from stdin and evaluates Axon expressions.

**Current state:** The REPL only performs parsing and type checking — it does
not execute code. This limits the attack surface.

**Future risks (when execution is added):**

- **File system access:** Axon code in the REPL should be sandboxed.
- **Network access:** Should be disabled by default in REPL mode.
- **Resource limits:** CPU time and memory should be bounded.
- **History file:** REPL history should be stored with restricted permissions.

**Recommendations:**

1. Implement a capability-based security model for REPL execution.
2. Default to a restricted sandbox with explicit opt-in for I/O.
3. Add `:sandbox on/off` REPL command for security-conscious users.

## 6. Memory Safety Guarantees

### 6.1 Rust Safety Model

The Axon compiler is written entirely in safe Rust. This provides:

- **No buffer overflows:** Bounds checking on all array/vector accesses.
- **No use-after-free:** Ownership system prevents dangling references.
- **No null pointer dereference:** `Option<T>` forces explicit handling.
- **No data races:** Borrow checker prevents shared mutable state.
- **No uninitialized memory:** All values must be initialized.

### 6.2 Allocation Patterns

- **Type interner** (`src/types.rs`): Uses arena-style allocation via `Vec<Type>`.
  Types are identified by index (`TypeId`), preventing dangling references.
- **Symbol table** (`src/symbol.rs`): Uses scoped `HashMap` with stack-based
  scope management. Scopes are pushed/popped correctly.
- **AST nodes** (`src/ast.rs`): Heap-allocated via `Box<Expr>` for recursive
  types. Ownership is clear and single-owner.
- **Error collection:** Errors are collected in `Vec<CompileError>` and
  returned to the caller. No global mutable state.

### 6.3 Dependencies

| Crate        | Version | Purpose              | Risk                 |
| ------------ | ------- | -------------------- | -------------------- |
| `serde`      | 1.x     | Serialization        | Low — widely audited |
| `serde_json` | 1.x     | JSON output          | Low — widely audited |
| `clap`       | 4.x     | CLI argument parsing | Low — widely audited |

All dependencies are well-established, widely audited crates with no
known vulnerabilities.

## Summary

| Area             | Status                      | Risk Level          |
| ---------------- | --------------------------- | ------------------- |
| `unsafe` code    | None found                  | ✅ None             |
| FFI boundaries   | Clang subprocess only       | ⚠️ Low              |
| Input validation | Good (with fuzz tests)      | ⚠️ Low              |
| Package registry | Not yet implemented         | 📋 Future           |
| REPL security    | Parse-only (no execution)   | ✅ None (currently) |
| Memory safety    | Full Rust safety guarantees | ✅ None             |
| Dependencies     | 3 well-audited crates       | ✅ None             |

**Overall assessment:** The Axon compiler has a strong security posture
thanks to being written in safe Rust with minimal dependencies. The primary
areas for future hardening are input size limits and the package registry
security model.
