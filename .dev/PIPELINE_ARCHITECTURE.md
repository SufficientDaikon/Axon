# Compiler Pipeline Architecture

> All paths relative to `h:/programing language/axon/`

## Pipeline Overview

```
Source text
  → Lexer (src/lexer.rs)          → Vec<Token>
  → Parser (src/parser.rs)        → ast::Program
  → TypeChecker (src/typeck.rs)   → TypeChecker (interner + symbols + errors)
  → TAST Builder (src/tast.rs)    → TypedProgram
  → MIR Builder (src/mir.rs)      → MirProgram (CFG)
  → LLVM Codegen (src/codegen/)   → String (LLVM IR text)
  → clang                         → native binary
```

## Stage 1: Lexer

| | |
|---|---|
| **File** | `src/lexer.rs` (~1034 lines) |
| **Input** | Source text (`String`) |
| **Output** | `Vec<Token>` |
| **Entry** | `Lexer::new(source, filename).tokenize()` |
| **Token type** | `Token { kind: TokenKind, span: Span }` (src/token.rs) |
| **Keyword recognition** | `token.rs::lookup_keyword()` called at lexer.rs:~368 |
| **Key methods** | `next_token()`, `advance()`, `peek()`, `lex_identifier_or_keyword()` |

Character-by-character scanning. Multi-char operators use 1-char lookahead (`peek()`).

## Stage 2: Parser

| | |
|---|---|
| **File** | `src/parser.rs` (~2100 lines) |
| **Input** | `Vec<Token>` |
| **Output** | `ast::Program` (defined in src/ast.rs) |
| **Entry** | `Parser::new(tokens, filename).parse_program()` |
| **Technique** | Recursive descent + hand-written precedence climbing |
| **Error recovery** | `synchronize()` skips to next `;`, `}`, or keyword |

### Key parse functions
| Function | ~Line | Parses |
|----------|-------|--------|
| `parse_program()` | 191 | Top-level: collects items |
| `parse_item()` | 228 | Dispatches fn/struct/enum/impl/trait/type/mod/use |
| `parse_fn_decl()` | 305 | Function decl (uses `->` for return type at ~323) |
| `parse_fn_params()` | 349 | Parameter list `(name: Type, ...)` |
| `parse_generic_params()` | 405 | `<T: Bound>` |
| `parse_use_decl()` | 710 | Use declarations (uses `::` at ~717) |
| `parse_statement()` | 736 | let/return/expr/assignment |
| `parse_expression()` | 934 | Entry to precedence chain |
| `parse_identifier_or_path()` | 1467 | Ident vs path (uses `::` at ~1472) |
| `parse_match_expr()` | 1579 | Match + arms (uses `=>` at ~1597) |
| `parse_type()` | 1627 | Type expressions dispatch |
| `parse_named_type()` | 1730 | Named/path/generic types (uses `::` at ~1736) |
| `parse_pattern()` | 1866 | Patterns incl. enum variants (uses `::` at ~1939) |

### AST representation (src/ast.rs)
- Paths = `Vec<String>` (no dedicated Path struct)
- `ExprKind::Path(Vec<String>)` — multi-segment paths
- `ExprKind::Identifier(String)` — single names
- `TypeExprKind::Path(Vec<String>)` — path types
- `FnDecl.return_type: Option<TypeExpr>` — return type after `->`
- `UseDecl.path: Vec<String>` — import paths

## Stage 3: Type Checker

| | |
|---|---|
| **File** | `src/typeck.rs` (~1750 lines) |
| **Input** | Source text (re-parses internally) |
| **Output** | `TypeChecker` (owns `interner: TypeInterner` + `symbols: SymbolTable`) |
| **Entry** | `typeck::check(source, filename)` |
| **Orchestrated by** | `lib.rs::check_source_full()` which also runs borrow checker |

Registers stdlib via `stdlib::register_stdlib()` before checking. Type unification is eager (no constraint solver).

### Key types
- `TypeInterner` (src/types.rs:~303): arena of `Type` values, deduplicates
- `TypeId(u32)`: lightweight handle into the interner
- Well-known: `TypeId::UNIT(0)`, `BOOL(3)`, `INT32(4)`, `INT64(5)`, `FLOAT32(6)`, `FLOAT64(7)`, `STRING(8)`
- `SymbolTable` (src/symbol.rs): scoped `HashMap<String, SymbolId>` with `lookup()` / `lookup_all()`

## Stage 3b: TAST Builder

| | |
|---|---|
| **File** | `src/tast.rs` (~800 lines) |
| **Input** | `ast::Program` + `&TypeChecker` |
| **Output** | `TypedProgram` |
| **Entry** | `TastBuilder::new(&checker).build(&program)` |

Every node carries a resolved `TypeId`. Uses `checker.symbols.lookup_all()` (not just `lookup()`) since typeck pops function scopes.

- `TypedExprKind::Path(Vec<String>)` — mirrors AST path
- `TypedFnDecl.return_type: TypeId` — resolved from `Option<TypeExpr>`

## Stage 4: MIR Builder

| | |
|---|---|
| **File** | `src/mir.rs` (~2000 lines) |
| **Input** | `TypedProgram` + `&TypeInterner` |
| **Output** | `MirProgram` (CFG: basic blocks + terminators) |
| **Entry** | `MirBuilder::new(&interner).build(&typed_program)` |

Paths joined: `segments.join("::")` at ~line 966 for function resolution.
`MirFunction.return_ty: TypeId`.

## Stage 5: LLVM Codegen

| | |
|---|---|
| **File** | `src/codegen/llvm.rs` (~2100 lines) |
| **Input** | `MirProgram` + `&TypeInterner` |
| **Output** | `String` (LLVM IR text) |
| **Entry** | `LlvmCodegen::new(&interner, opt_level).generate(&mir_program)` |

No direct syntax references — works entirely on MIR abstractions.
- Return types: `func.return_ty` → `llvm_type()` → IR type string
- Function names: MIR-provided string (already `::` joined)
- Tensor dispatch: `try_emit_tensor_call()`
- Name mangling: `src/codegen/abi.rs`

### Stage 6: Linking
`compile_and_link()` writes IR to temp file, invokes clang, links with embedded C runtime.

## Orchestration (src/lib.rs)

```rust
check_source_full(source, filename)
  -> (TypedProgram, Vec<CompileError>, TypeChecker)
```

**CRITICAL**: Returns the TypeChecker so MIR/codegen share the SAME TypeInterner instance.

## Tooling (independent of compilation)

| Tool | File | Entry |
|------|------|-------|
| Formatter | `src/fmt.rs` (~850 lines) | `AxonFormatter::format()` |
| Linter | `src/lint.rs` (~600 lines) | `Linter::new(config).lint()` |
| Doc Generator | `src/doc.rs` (~660 lines) | `DocGenerator::generate()` |
| LSP Server | `src/lsp/` (3 files) | `LspServer::run()` |
| REPL | `src/repl.rs` (~350 lines) | `Repl::new().run()` |
| Debugger | `src/debugger.rs` (~160 lines) | `Debugger::new().debug()` |
| Package Manager | `src/pkg/` (5 files) | Various |
