# Source File Reference

> All paths relative to `h:/programing language/axon/`

## Core Compiler (src/)

| File | ~Lines | Role | Key Types/Functions |
|------|--------|------|-------------------|
| `lib.rs` | 82 | Module root + orchestration | `parse_source()`, `check_source()`, `check_source_full()` |
| `main.rs` | ~200 | CLI entry | Commands: parse, lex, check, build, fmt, lint, doc |
| `token.rs` | ~300 | Token definitions | `TokenKind` enum, `Token`, `lookup_keyword()` |
| `lexer.rs` | ~1034 | Tokenizer | `Lexer`, `tokenize()`, `next_token()` |
| `parser.rs` | ~2100 | Parser | `Parser`, `parse_program()`, `parse_fn_decl()`, `parse_expression()` |
| `ast.rs` | ~551 | AST node types | `Program`, `Expr`, `ExprKind`, `Stmt`, `TypeExpr`, `Pattern`, `FnDecl` |
| `typeck.rs` | ~1750 | Type checker | `TypeChecker`, `check()`, `check_fn_call()`, `resolve_type_expr()` |
| `types.rs` | ~400 | Type system | `Type` enum, `TypeId`, `TypeInterner`, `PrimKind` |
| `symbol.rs` | ~1200 | Symbol table | `SymbolTable`, `SymbolInfo`, `SymbolKind`, `lookup()`, `lookup_all()` |
| `tast.rs` | ~800 | Typed AST | `TypedProgram`, `TastBuilder`, `TypedExpr`, `build_expr()` |
| `mir.rs` | ~2000 | Mid-level IR | `MirBuilder`, `MirProgram`, `MirFunction`, `lower_expr()` |
| `borrow.rs` | ~850 | Borrow checker | `BorrowChecker::check_program()` |
| `shapes.rs` | ~420 | Tensor shapes | `check_shapes()` |
| `error.rs` | ~110 | Error types | `CompileError`, `Severity`, `ErrorReporter` |
| `span.rs` | ~80 | Source locations | `Span`, `Position` |
| `fmt.rs` | ~850 | Code formatter | `AxonFormatter::format()` — **hardcodes `"::"` and `" -> "` strings** |
| `doc.rs` | ~660 | Doc generator | `DocGenerator::generate()` — **hardcodes `"::"` and `" -> "` strings** |
| `lint.rs` | ~600 | Linter | `Linter::lint()` |
| `repl.rs` | ~350 | REPL | `Repl::run()` |

## Codegen (src/codegen/)

| File | Role | Key Functions |
|------|------|---------------|
| `mod.rs` | Module declarations | — |
| `llvm.rs` (~2100) | LLVM IR text generation | `LlvmCodegen::generate()`, `emit_function()`, `try_emit_tensor_call()` |
| `runtime.rs` (~1200) | Embedded C runtime source | `get_runtime_source()`, tensor ops, autograd tape |
| `abi.rs` (~120) | Name mangling | `NameMangler::mangle()` — **uses `"::"` join** |
| `mlir.rs` (~70) | GPU backend placeholder | Minimal, deferred to Phase 12 |

## Standard Library Registration (src/stdlib/)

| File | What It Registers |
|------|------------------|
| `mod.rs` (~220) | `register_stdlib()` calls 22 sub-modules; helpers: `def_fn()`, `def_struct()`, `def_method()`, `def_trait()`, `def_const()`, `def_enum()` |
| `prelude.rs` (~200) | Core: `println`, `print`, `assert`, `tensor_*`, `autograd_*`, `sgd_step` |
| `math.rs` (~125) | `sin`, `cos`, `sqrt`, `abs`, `pow`, `log`, `exp`, etc. |
| `string.rs` (~60) | String methods: `len`, `contains`, `to_uppercase` |
| `collections.rs` (~310) | Vec, HashMap, HashSet methods |
| `io.rs` (~165) | File I/O: `read_file`, `write_file` |
| `tensor.rs` (~230) | Tensor type methods |
| `nn.rs` (~260) | Neural layers: Linear, Conv2d, BatchNorm, etc. |
| `autograd.rs` (~290) | Autograd functions |
| `optim.rs` (~190) | Optimizers: SGD, Adam |
| `loss.rs` (~100) | Loss functions: mse, cross_entropy |
| `ops.rs` (~70) | Operator trait registrations |
| `convert.rs` (~70) | Type conversion functions |
| `sync.rs` (~100) | Mutex, RwLock, Channel |
| `thread.rs` (~50) | Thread spawning |
| `data.rs` (~65) | DataLoader, Dataset |
| `device.rs` (~40) | Device enum (CPU/GPU) |
| `random.rs` (~45) | Random number generation |
| `mem.rs` (~55) | Memory utilities |
| `time.rs` (~45) | Timer, benchmarking |
| `train.rs` (~100) | Training loop utilities |
| `metrics.rs` (~35) | Accuracy, precision metrics |
| `transforms.rs` (~90) | Data transforms |
| `export.rs` (~25) | Model export (ONNX placeholder) |

## LSP (src/lsp/)

| File | Role | Notes |
|------|------|-------|
| `mod.rs` | Module declarations | — |
| `server.rs` (~210) | LSP server loop | `LspServer::run()` |
| `handlers.rs` (~920) | Request handlers | completion, hover, signature — **uses `" -> "` strings** |
| `protocol.rs` (~415) | LSP protocol types | Message, Request, Response |

## Package Manager (src/pkg/)

| File | Role |
|------|------|
| `mod.rs` | Module declarations |
| `manifest.rs` (~590) | `Axon.toml` parsing |
| `lockfile.rs` (~120) | Lock file handling |
| `resolver.rs` (~75) | Dependency resolution |
| `scaffold.rs` (~100) | Project scaffolding |

## Test Files (tests/)

| File | Type | Count |
|------|------|-------|
| `e2e_tests.rs` | E2E harness | `compile_and_run()`, `assert_e2e_output()` |
| `codegen_tests.rs` (~700) | Codegen + MIR tests | ~44 `#[test]` |
| `integration_tests.rs` | Integration tests | Various |
| `type_tests.rs` | Type system tests | Various |
| `stdlib_tests.rs` | Stdlib tests | Various |
| `ai_framework_tests.rs` | AI/ML framework tests | Various |
| `fuzz_tests.rs` | Fuzz/robustness tests | Various |
| `e2e/*.axon` (37 files) | E2E programs | `// expect:` pattern |
| `examples/*.axon` (9 files) | Example programs | Used by integration_tests |

## Documentation Site (docs/)

| File | Role |
|------|------|
| `build_docs.py` | Assembles documentation.html from CSS+JS+markdown |
| `docs.css` | Documentation site styles |
| `docs.js` | Documentation site JS (nav, search, tooltips) |
| `documentation.html` (~409KB) | Built output (21 pages inlined) |
| `index.html` | Landing page |
| `guide/*.md` (7 files) | Language guide articles |
| `reference/*.md` (2 files) | CLI + error reference |
| `tutorials/*.md` (6 files) | Tutorial series |
| `migration/*.md` (3 files) | Migration guides (Python, PyTorch, Rust) |
