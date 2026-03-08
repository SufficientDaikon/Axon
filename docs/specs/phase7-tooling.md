# Phase 7: Tooling — Implementation Specification

## 1. Overview

Phase 7 delivers the **developer experience layer** for Axon: tools that make writing, debugging, and managing Axon code productive and pleasant. This includes a Language Server Protocol (LSP) implementation, a package manager, a REPL, a debugger, and IDE extensions.

### Deliverables

| Tool                    | Artifact          | Purpose                                                              |
| ----------------------- | ----------------- | -------------------------------------------------------------------- |
| Language Server         | `axon-lsp/`       | LSP for IDE integration (completions, diagnostics, hover, go-to-def) |
| Package Manager         | `axon-pkg/`       | Dependency management, package registry, build orchestration         |
| REPL                    | `axonc repl`      | Interactive Axon evaluation shell                                    |
| Debugger                | `axon-dbg/`       | Source-level debugger (DAP protocol)                                 |
| Formatter               | `axonc fmt`       | Opinionated code formatter                                           |
| Linter                  | `axonc lint`      | Static analysis beyond type checking                                 |
| VS Code Extension       | `editors/vscode/` | Syntax highlighting, LSP client, snippets                            |
| Documentation Generator | `axonc doc`       | Generate HTML docs from doc comments                                 |

### Dependencies

- All previous phases (0–6) provide the compiler pipeline the tools wrap.

---

## 2. Language Server (`axon-lsp/`)

### 2.1 Architecture

```
Editor (VS Code / Neovim / etc.)
  ↕ JSON-RPC over stdio
axon-lsp (Rust binary)
  ├── Document Manager (open/close/change tracking)
  ├── Incremental Lexer + Parser
  ├── Incremental Type Checker
  ├── Completion Engine
  ├── Diagnostic Publisher
  └── Index (cross-file symbol table)
```

### 2.2 Supported LSP Features

| Feature               | LSP Method                        | Priority |
| --------------------- | --------------------------------- | -------- |
| **Diagnostics**       | `textDocument/publishDiagnostics` | P0       |
| **Go to Definition**  | `textDocument/definition`         | P0       |
| **Hover** (type info) | `textDocument/hover`              | P0       |
| **Completion**        | `textDocument/completion`         | P0       |
| **Signature Help**    | `textDocument/signatureHelp`      | P0       |
| **Find References**   | `textDocument/references`         | P1       |
| **Rename**            | `textDocument/rename`             | P1       |
| **Document Symbols**  | `textDocument/documentSymbol`     | P1       |
| **Workspace Symbols** | `workspace/symbol`                | P1       |
| **Code Actions**      | `textDocument/codeAction`         | P1       |
| **Formatting**        | `textDocument/formatting`         | P1       |
| **Inlay Hints**       | `textDocument/inlayHint`          | P2       |
| **Semantic Tokens**   | `textDocument/semanticTokens`     | P2       |
| **Call Hierarchy**    | `callHierarchy/incomingCalls`     | P2       |
| **Folding Ranges**    | `textDocument/foldingRange`       | P2       |

### 2.3 Incremental Analysis

The LSP must be fast. Strategy:

1. **Incremental parsing**: On edit, only re-parse the changed region + surrounding items.
2. **Incremental type checking**: Re-check only functions/items that depend on changed symbols.
3. **Background analysis**: Full re-check runs on a background thread; diagnostics published when ready.
4. **Caching**: Cache parsed ASTs and type info per file; invalidate on change.

### 2.4 Completion Engine

Context-aware completions:

| Context                    | Completions Offered                                          |
| -------------------------- | ------------------------------------------------------------ |
| After `.`                  | Fields, methods (inherent + trait) for the receiver type     |
| After `::`                 | Associated functions, enum variants, module items            |
| After `<` in type position | Generic parameters                                           |
| Start of statement         | Keywords (`let`, `if`, `while`, `for`, `fn`, `struct`, etc.) |
| Inside function body       | Local variables, parameters, imports                         |
| After `use`                | Module paths from dependency tree                            |
| Inside `@`                 | `@cpu`, `@gpu`, `@device(...)`                               |
| After `:` in `let x:`      | Type names (primitives, structs, enums, generics)            |

### 2.5 Diagnostics

Real-time diagnostics from all phases:

- Lexer errors (E0xxx)
- Parser errors (E0xxx)
- Name resolution errors (E1xxx)
- Type errors (E2xxx)
- Shape errors (E3xxx)
- Borrow errors (E4xxx)
- Lint warnings (W5xxx)

---

## 3. Package Manager (`axon-pkg/`)

### 3.1 Project Structure

```
my-project/
├── Axon.toml          # Project manifest
├── Axon.lock          # Lockfile (auto-generated)
├── src/
│   ├── main.axon      # Binary entry point
│   └── lib.axon       # Library root
├── tests/
├── benches/
├── examples/
└── target/            # Build artifacts
```

### 3.2 Manifest (`Axon.toml`)

```toml
[package]
name = "my-project"
version = "0.1.0"
authors = ["Author Name <email@example.com>"]
edition = "2026"
description = "A sample Axon project"
license = "MIT"

[dependencies]
some-lib = "1.2.3"
gpu-utils = { git = "https://github.com/user/gpu-utils", branch = "main" }
local-dep = { path = "../local-dep" }

[dev-dependencies]
test-utils = "0.5.0"

[build]
target = "native"      # or "wasm", "cuda"
opt-level = 2

[features]
gpu = []
```

### 3.3 Commands

```
axon-pkg new <name>           # Create new project
axon-pkg init                 # Initialize in existing directory
axon-pkg build                # Compile the project
axon-pkg run                  # Build and run
axon-pkg test                 # Run tests
axon-pkg bench                # Run benchmarks
axon-pkg add <package>        # Add dependency
axon-pkg remove <package>     # Remove dependency
axon-pkg update               # Update dependencies
axon-pkg publish              # Publish to registry
axon-pkg search <query>       # Search registry
axon-pkg doc                  # Generate documentation
axon-pkg fmt                  # Format all source files
axon-pkg lint                 # Run linter
axon-pkg clean                # Remove build artifacts
```

### 3.4 Dependency Resolution

- Semver-based version resolution.
- SAT solver for dependency conflicts.
- Lock file for reproducible builds.
- Support for: registry packages, git dependencies, local path dependencies.

### 3.5 Package Registry

A central registry (like crates.io) for Axon packages:

- API: REST + JSON
- Authentication via API tokens
- Package search, download counts, version history
- Automated build verification on publish

---

## 4. REPL (`axonc repl`)

### 4.1 Features

```
$ axonc repl
Axon v0.1.0 REPL — Type :help for commands
>>> let x = 42
x: Int32 = 42
>>> let t = Tensor::zeros([3, 3])
t: Tensor<Float32, [3, 3]> = [[0.0, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0]]
>>> t.shape()
[3, 3]
>>> fn add(a: Int32, b: Int32) -> Int32 { a + b }
add: fn(Int32, Int32) -> Int32
>>> add(x, 10)
52
>>> :type add
fn(Int32, Int32) -> Int32
>>> :help
  :type <expr>    Show type of expression
  :ast <expr>     Show AST of expression
  :clear          Clear all bindings
  :load <file>    Load .axon file
  :save <file>    Save session to file
  :quit           Exit REPL
```

### 4.2 Implementation

- JIT compilation via LLVM's ORC JIT for fast expression evaluation.
- Persistent state across inputs (variables, functions survive between entries).
- Tab completion using the LSP completion engine.
- Syntax highlighting using the lexer.
- History with up/down arrows (rustyline or similar).

---

## 5. Debugger (`axon-dbg/`)

### 5.1 Architecture

Implement the **Debug Adapter Protocol (DAP)** so any DAP-compatible editor (VS Code, Neovim, etc.) can debug Axon programs.

### 5.2 Features

| Feature                  | Description                                                     |
| ------------------------ | --------------------------------------------------------------- |
| **Breakpoints**          | Line breakpoints, conditional breakpoints, function breakpoints |
| **Step execution**       | Step over, step into, step out, continue                        |
| **Variable inspection**  | View locals, parameters, fields, tensor contents                |
| **Watch expressions**    | Evaluate expressions at breakpoint                              |
| **Call stack**           | View full call stack with source locations                      |
| **Tensor visualization** | Pretty-print tensor data with shapes                            |
| **GPU debugging**        | Inspect GPU tensor data (transferred to CPU for display)        |

### 5.3 Implementation

- Compile with debug info (Phase 4 DWARF emission).
- Use LLDB as the backend debugger engine.
- `axon-dbg` wraps LLDB with Axon-aware type pretty-printers.
- DAP adapter translates between editor and LLDB.

---

## 6. Formatter (`axonc fmt`)

### 6.1 Style Rules

| Rule            | Description                                          |
| --------------- | ---------------------------------------------------- |
| Indentation     | 4 spaces (no tabs)                                   |
| Line width      | 100 characters max                                   |
| Braces          | Opening brace on same line                           |
| Trailing commas | Always in multi-line lists                           |
| Imports         | Sorted alphabetically, grouped by std/external/local |
| Blank lines     | 1 between items, 0 between consecutive fields        |
| Alignment       | Align `=>` in match arms when ≤ 5 arms               |

### 6.2 Implementation

- Parse source → AST → re-emit formatted source.
- Preserve comments (attach to nearest AST node).
- Idempotent: `axonc fmt` applied twice produces the same output.

---

## 7. Linter (`axonc lint`)

### 7.1 Lint Categories

| Category        | Examples                                                                |
| --------------- | ----------------------------------------------------------------------- |
| **Correctness** | Unused variables, unreachable code, missing match arms                  |
| **Performance** | Unnecessary clones, tensor on wrong device, unneeded allocations        |
| **Style**       | Naming conventions (snake_case for vars, PascalCase for types)          |
| **ML-specific** | Forgetting `model.train()`, not calling `zero_grad()`, unused gradients |
| **Safety**      | Unwrap on Option/Result without checks, raw pointer usage               |

### 7.2 Configuration

```toml
# Axon.toml [lint] section
[lint]
warn = ["unused-variables", "missing-docs"]
deny = ["unsafe-unwrap"]
allow = ["snake-case-warning"]
```

---

## 8. VS Code Extension (`editors/vscode/`)

### 8.1 Features

- Syntax highlighting (TextMate grammar for `.axon` files)
- LSP client (connects to `axon-lsp`)
- Snippets (fn, struct, enum, impl, match, for, etc.)
- Tensor shape display in hover
- Run/Debug integration (launch.json configurations)
- Problem matcher for `axonc build` output
- Code lens: "Run" / "Debug" above main functions

### 8.2 TextMate Grammar Scopes

```
source.axon
  keyword.control.axon           — fn, let, if, else, while, for, match, return
  keyword.other.axon             — pub, use, mod, impl, trait, struct, enum, type, unsafe
  storage.type.axon              — Int32, Float64, Tensor, Vec, etc.
  entity.name.function.axon      — function names
  entity.name.type.axon          — struct/enum/trait names
  variable.parameter.axon        — function parameters
  string.quoted.double.axon      — string literals
  constant.numeric.axon          — number literals
  comment.line.axon              — // comments
  comment.block.axon             — /* */ comments
  keyword.operator.axon          — +, -, *, /, @, ==, etc.
  meta.attribute.axon            — @cpu, @gpu, @device
```

---

## 9. Documentation Generator (`axonc doc`)

### 9.1 Doc Comment Syntax

````axon
/// This is a doc comment for the item below.
///
/// # Examples
///
/// ```
/// let x = add(2, 3);
/// assert(x == 5);
/// ```
///
/// # Arguments
/// - `a`: First operand
/// - `b`: Second operand
fn add(a: Int32, b: Int32) -> Int32 {
    a + b
}
````

### 9.2 Output

- Generates HTML documentation (similar to rustdoc).
- Includes: module hierarchy, type signatures, doc comments, cross-references.
- Renders code examples with syntax highlighting.
- Supports `axonc doc --open` to build and open in browser.

---

## 10. Task Breakdown

### Phase 7a: Language Server

- [ ] T205 Set up axon-lsp crate with LSP protocol handling — `axon-lsp/`
- [ ] T206 Implement document synchronization (open/close/change) — `axon-lsp/`
- [ ] T207 Implement real-time diagnostics from lexer/parser/typeck — `axon-lsp/`
- [ ] T208 Implement go-to-definition — `axon-lsp/`
- [ ] T209 Implement hover (type info, doc comments) — `axon-lsp/`
- [ ] T210 Implement completion engine — `axon-lsp/`
- [ ] T211 Implement find-references and rename — `axon-lsp/`
- [ ] T212 Implement signature help — `axon-lsp/`
- [ ] T213 Implement inlay hints (inferred types) — `axon-lsp/`
- [ ] T214 Implement semantic tokens — `axon-lsp/`

### Phase 7b: Package Manager

- [ ] T215 Implement project scaffolding (new/init) — `axon-pkg/`
- [ ] T216 Define and parse Axon.toml manifest — `axon-pkg/`
- [ ] T217 Implement dependency resolution (semver + SAT) — `axon-pkg/`
- [ ] T218 Implement lock file generation — `axon-pkg/`
- [ ] T219 Implement build orchestration (multi-package) — `axon-pkg/`
- [ ] T220 Implement `add`/`remove`/`update` commands — `axon-pkg/`
- [ ] T221 Implement registry client (publish/search/download) — `axon-pkg/`

### Phase 7c: REPL

- [ ] T222 Implement REPL shell with history and line editing — `src/repl.rs`
- [ ] T223 Implement JIT compilation for expressions — `src/repl.rs`
- [ ] T224 Implement persistent state (variables, functions) — `src/repl.rs`
- [ ] T225 Implement REPL commands (:type, :ast, :load, :save) — `src/repl.rs`
- [ ] T226 Implement tab completion in REPL — `src/repl.rs`

### Phase 7d: Debugger

- [ ] T227 Implement DAP server — `axon-dbg/`
- [ ] T228 Implement breakpoint management — `axon-dbg/`
- [ ] T229 Implement step execution (over/into/out) — `axon-dbg/`
- [ ] T230 Implement variable inspection with type-aware pretty printing — `axon-dbg/`
- [ ] T231 Implement tensor visualization in debugger — `axon-dbg/`

### Phase 7e: Formatter, Linter, Docs

- [ ] T232 Implement code formatter (AST → formatted source) — `src/fmt.rs`
- [ ] T233 Implement linter with configurable rules — `src/lint.rs`
- [ ] T234 Implement doc comment parser — `src/doc.rs`
- [ ] T235 Implement HTML documentation generator — `src/doc.rs`

### Phase 7f: VS Code Extension

- [ ] T236 Create TextMate grammar for .axon files — `editors/vscode/`
- [ ] T237 Implement LSP client extension — `editors/vscode/`
- [ ] T238 Add snippets and keybindings — `editors/vscode/`
- [ ] T239 Add launch.json configurations for run/debug — `editors/vscode/`
- [ ] T240 Publish extension to VS Code Marketplace — `editors/vscode/`

### Phase 7g: Testing

- [ ] T241 Test LSP: diagnostics, completion, go-to-def — `tests/lsp_tests.rs`
- [ ] T242 Test package manager: resolve, build, publish — `tests/pkg_tests.rs`
- [ ] T243 Test REPL: expression evaluation, state persistence — `tests/repl_tests.rs`
- [ ] T244 Test formatter: idempotency, comment preservation — `tests/fmt_tests.rs`
- [ ] T245 Test linter: all lint rules fire correctly — `tests/lint_tests.rs`

---

## 11. Acceptance Criteria

- [ ] LSP provides real-time diagnostics with < 200ms latency on edit
- [ ] Go-to-definition works across files and into stdlib
- [ ] Completions show correct methods/fields for any typed expression
- [ ] `axon-pkg build` compiles multi-file projects with dependencies
- [ ] REPL evaluates expressions and persists state across entries
- [ ] Debugger sets breakpoints, steps, and inspects variables
- [ ] Formatter produces consistent output and is idempotent
- [ ] VS Code extension installs and provides syntax highlighting + LSP
- [ ] `axonc doc` generates navigable HTML documentation
- [ ] Tensor shapes shown in hover tooltips
