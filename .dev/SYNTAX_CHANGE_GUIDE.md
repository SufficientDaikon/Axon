# Syntax Change Guide

> Action-oriented checklists for modifying the Axon compiler.
> All paths relative to `h:/programing language/axon/`
> Line numbers are approximate — grep patterns are authoritative.

---

## RECIPE: Change an Operator Symbol

**Example**: `::` → `.` for paths, `->` → `:` for return types

### Step 1: Token Definition (src/token.rs)

- [ ] Update `TokenKind` variant comment (grep: `Arrow` or `ColonColon` in enum)
- [ ] Update `name()` return string (grep: `TokenKind::Arrow =>` or `TokenKind::ColonColon =>`)
  - `Arrow => "->"` at line ~188
  - `ColonColon => "::"` at line ~190
- [ ] If REMOVING the token entirely: remove variant + all match arms in `name()`, `Display`, etc.
- [ ] If REUSING an existing token (e.g., `.` Dot already exists): no token.rs changes needed, just parser changes

### Step 2: Lexer (src/lexer.rs)

- [ ] Find the character match arm for the OLD operator's first char
- [ ] Update the character sequence logic

**For `::` (lines ~328-336)**: The `:` arm produces `ColonColon` when followed by `:`, else `Colon`.
**For `->` (lines ~275-286)**: The `-` arm produces `Arrow` when followed by `>`, else checks `-=` or `Minus`.

- [ ] **CONFLICT CHECK**: Verify the new char sequence doesn't collide with existing tokens
  - `.` already produces `Dot` and `DotDot` — if reusing Dot for paths, change PARSER not lexer
  - `:` already produces `Colon` and `ColonColon`
- [ ] Update lexer tests (grep: `#[test]` in lexer.rs, ~line 717+)

### Step 3: Parser (src/parser.rs)

Find all uses: grep `TokenKind::ColonColon` or `TokenKind::Arrow`

**`::` (ColonColon) — 4 parser sites:**
| Site | Function | ~Line | What It Does |
|------|----------|-------|-------------|
| 1 | `parse_use_decl()` | 717 | `while self.eat(&TokenKind::ColonColon)` builds use path |
| 2 | `parse_identifier_or_path()` | 1472 | `if self.at(&TokenKind::ColonColon)` detects path vs ident |
| 3 | `parse_named_type()` | 1736 | `while self.at(&TokenKind::ColonColon)` builds type path |
| 4 | `parse_pattern()` | 1939 | `while self.eat(&TokenKind::ColonColon)` builds enum variant path |

**`->` (Arrow) — 2 parser sites:**
| Site | Function | ~Line | What It Does |
|------|----------|-------|-------------|
| 1 | `parse_fn_decl()` | 323 | `if self.at(&TokenKind::Arrow)` checks for return type |
| 2 | `parse_type()` | 1670 | `if self.eat(&TokenKind::Arrow)` in fn type `fn(A) -> B` |

- [ ] Replace all `TokenKind::OldToken` with `TokenKind::NewToken`
- [ ] Update parser error messages that reference the old syntax
- [ ] Update parser tests (grep: `#[cfg(test)]` in parser.rs, ~line 2056+)

### Step 4: String Literals (THE #1 THING MISSED)

These files have the old syntax hardcoded as string literals, NOT as tokens:

**Files with `"::"` string (14 occurrences in 7 files):**

| File | Line | Code | Grep |
|------|------|------|------|
| `src/fmt.rs` | 328 | `u.path.join("::")` | use path formatting |
| `src/fmt.rs` | 401 | `parts.join("::")` | expr path formatting |
| `src/fmt.rs` | 546 | `name.join("::")` | struct literal name |
| `src/fmt.rs` | 584 | `parts.join("::")` | type path formatting |
| `src/fmt.rs` | 682 | `name.join("::")` | pattern struct name |
| `src/fmt.rs` | 697 | `path.join("::")` | pattern enum variant |
| `src/doc.rs` | 293 | `parts.join("::")` | doc path display |
| `src/mir.rs` | 966 | `segments.join("::")` | MIR function lookup |
| `src/mir.rs` | 1327 | `name.join("::")` | MIR struct name |
| `src/parser.rs` | 1778 | `path.join("::")` | error message path |
| `src/types.rs` | 276 | `path.join("::")` | Type Display impl |
| `src/typeck.rs` | 1306 | `name.join("::")` | struct literal lookup |
| `src/codegen/abi.rs` | 26 | `namespace.join("::")` + `"::"` | name mangling |
| `src/token.rs` | 190 | `"::"` | name() display |

**Files with `" -> "` string (3 occurrences in 2 files):**

| File | Line | Code | Grep |
|------|------|------|------|
| `src/fmt.rs` | 121 | `self.write(" -> ")` | fn return type formatting |
| `src/fmt.rs` | 572 | `self.write(" -> ")` | fn type formatting |
| `src/doc.rs` | 269 | `sig.push_str(" -> ")` | doc fn signature |

**Also check `"->"` (no spaces):**

| File | Line | Code |
|------|------|------|
| `src/token.rs` | 188 | `Arrow => "->"` | name() display |

### Step 5: Downstream IR (no changes usually needed)

- AST, TAST: Paths stored as `Vec<String>` segments — separator is reconstructed at display time, NOT stored
- MIR: `segments.join("::")` at line 966 — **this IS a string literal, caught in Step 4**
- LLVM Codegen: Uses MIR function name strings — if MIR join changed, codegen follows automatically
- **Exception**: `src/codegen/abi.rs` line 26 — name mangling uses `"::"`, update in Step 4

### Step 6: Test Files

**E2E `.axon` files (tests/e2e/):**
- `::` — 0 files use it
- `->` — 11 files use it (see TEST_INVENTORY.md for full list)

**Example `.axon` files (tests/examples/):**
- `::` — 2 files: `example4_neural_net.axon`, `example7_errors.axon`
- `->` — 5 files: examples 3, 4, 5, 7, 8

**Inline Rust test strings** (grep `"fn ` or `"use ` in test modules):
- `src/parser.rs` tests (~line 2056+)
- `src/typeck.rs` tests (~line 1724+)
- `src/tast.rs` tests (~line 755+)
- `src/mir.rs` tests (~line 1769+)
- `src/fmt.rs` tests (~line 812+)
- `src/doc.rs` tests (~line 563+)
- `src/lexer.rs` tests (~line 717+)
- `src/codegen/llvm.rs` tests (~line 2234+)
- `src/codegen/abi.rs` tests (~line 109+)

### Step 7: Documentation

- [ ] `docs/guide/language-tour.md` — syntax examples
- [ ] `docs/guide/modules-packages.md` — import/use syntax
- [ ] `docs/reference/compiler-errors.md` — error message examples
- [ ] `docs/tutorials/*.md` — code examples
- [ ] `docs/migration/*.md` — comparison syntax
- [ ] Rebuild: `cd docs && python build_docs.py`

### Step 8: Verify

- [ ] `cargo test` — should pass all ~1,034 tests
- [ ] `cargo test --test e2e_tests` — full pipeline
- [ ] Grep entire `src/` for old syntax string to catch stragglers:
  - `grep -rn '"::"' src/`
  - `grep -rn '" -> "' src/`
  - `grep -rn '"->"' src/`

---

## RECIPE: Add a New Keyword

### Step 1: Token (src/token.rs)
- [ ] Add variant to `TokenKind` enum (keywords section, lines ~9-32)
- [ ] Add `name()` match arm (lines ~122-145)
- [ ] Add `lookup_keyword()` match arm (lines ~249-271)

### Step 2: Lexer (src/lexer.rs)
- [ ] Nothing needed — `lex_identifier_or_keyword()` auto-calls `lookup_keyword()`
- [ ] Add lexer test verifying the keyword tokenizes correctly

### Step 3: Parser (src/parser.rs)
- [ ] Add parsing logic where the keyword appears:
  - Starts a top-level item → add to `parse_item()` (~line 228)
  - Starts a statement → add to `parse_statement()` (~line 736)
  - Is an expression prefix → add to `parse_primary()` / `parse_atom()`
- [ ] Create the corresponding AST node(s)

### Step 4: AST (src/ast.rs)
- [ ] Add new `ExprKind`, `StmtKind`, or `Item` variant as needed

### Step 5: Type Checker (src/typeck.rs)
- [ ] Add type-checking for new construct in the relevant `check_*` function

### Step 6: TAST (src/tast.rs)
- [ ] Add `TypedExprKind` / `TypedStmtKind` / `TypedItem` variant
- [ ] Add `build_*` function in `TastBuilder`

### Step 7: MIR (src/mir.rs)
- [ ] Add `lower_*` function in `MirBuilder`
- [ ] May need new `MirStmt` or `Terminator` variant

### Step 8: Codegen (src/codegen/llvm.rs)
- [ ] Add LLVM IR emission for the new MIR construct

### Step 9: Tooling
- [ ] `src/fmt.rs` — formatting support
- [ ] `src/doc.rs` — doc generation
- [ ] `src/lint.rs` — lint rules
- [ ] `src/lsp/handlers.rs` — completion/hover

### Step 10: Tests
- [ ] Unit test in relevant module
- [ ] E2E test in `tests/e2e/`

---

## RECIPE: Add a New Built-in Function

### Step 1: Register (src/stdlib/)
- [ ] In `prelude.rs` or appropriate submodule
- [ ] Use `def_fn(symbols, interner, "name", vec![param_types], return_type)`
- [ ] For methods: `def_method(symbols, interner, "Type", "method", params, ret)`

### Step 2: Type Check Interception (if needed) (src/typeck.rs)
- [ ] In `check_fn_call()` (~line 827) — add special case if the function needs custom type logic
- [ ] Example: tensor builtins are intercepted here

### Step 3: MIR Lowering (src/mir.rs)
- [ ] In `lower_fn_call()` (~line 940) or `lower_print_call()` (~line 1002)
- [ ] Map Axon name → C runtime function name

### Step 4: Runtime Implementation (src/codegen/)
- [ ] `runtime.rs` — add C function implementation
- [ ] `llvm.rs` `emit_runtime_decls()` (~line 96) — add LLVM `declare` for the C function

### Step 5: Tests
- [ ] Unit test
- [ ] E2E test in `tests/e2e/` with `// expect:` pattern

---

## RECIPE: Add a New Type

### Step 1: Type System (src/types.rs)
- [ ] Add variant to `Type` enum
- [ ] Update `Display` impl
- [ ] Add `TypeId` constant if built-in (like `TypeId::STRING`)
- [ ] Pre-intern in `TypeInterner::new()` if needed (~line 308)

### Step 2: Token (if it has a keyword name) (src/token.rs)
- [ ] Add `TokenKind` variant (built-in types section, lines ~34-58)
- [ ] Add to `lookup_keyword()` match

### Step 3: Parser (src/parser.rs)
- [ ] In `parse_type()` / `parse_named_type()` — handle in type parsing

### Step 4: Type Checker (src/typeck.rs)
- [ ] In `resolve_type_expr()` (~line 1590+) — resolve the new type

### Step 5: MIR + Codegen
- [ ] `src/mir.rs` — handle in expression/statement lowering
- [ ] `src/codegen/llvm.rs` — map to LLVM type representation

---

## RECIPE: Change Operator Precedence

**Single location**: The parse function call chain in `src/parser.rs`

```
parse_expression (934) → parse_or_expr (948) → parse_and_expr (969)
  → parse_comparison (990) → parse_range_expr (1020) → parse_additive (1053)
  → parse_multiplicative (1079) → parse_cast (1107) → parse_unary (1127)
  → parse_postfix (1197) → parse_primary
```

- [ ] To raise precedence: move the operator to a function further down the chain
- [ ] To lower precedence: move it further up
- [ ] To add a new level: insert a new function in the chain
- [ ] Update the precedence comment at lines ~938-946

---

## RECIPE: Rename a Construct (e.g., `struct` → `model`)

### Step 1: Token (src/token.rs)
- [ ] Rename `TokenKind` variant (e.g., `Struct` → `Model`)
- [ ] Update `name()` return string
- [ ] Update `lookup_keyword()` entry (change `"struct"` → `"model"`)

### Step 2: Fix all references
- [ ] Grep `TokenKind::Struct` across all of `src/` — every match needs updating
- [ ] This includes: parser, AST, typeck, tast, mir, codegen, fmt, doc, lint, lsp

### Step 3: AST (src/ast.rs)
- [ ] Rename `StructDecl`, `StructLiteral`, etc. if desired
- [ ] Or just keep the AST names and only change the syntax keyword

### Step 4: String literals
- [ ] Grep for `"struct"` in error messages, formatting output, doc strings
- [ ] Update documentation files

### Step 5: Tests
- [ ] Grep all `.axon` test files for the old keyword
- [ ] Grep all inline Rust test strings

### Step 6: Verify
- [ ] `cargo test`
