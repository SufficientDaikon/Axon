# Axon Compiler — Claude Instructions

## Project
- **Root**: `h:/programing language/axon/`
- **Language**: Rust 2021 edition
- **Binary**: `axonc`
- **Build**: `cargo build`
- **Test**: `cargo test` (~1,034 tests)
- **Pipeline**: Lexer → Parser → TypeChecker → TAST → MIR → LLVM IR → clang → binary

## Knowledge Base (.dev/)

Before making compiler changes, **always read the relevant `.dev/` file first**:

| Task | Read This |
|------|-----------|
| Change/add/remove an operator or keyword | [`.dev/SYNTAX_CHANGE_GUIDE.md`](.dev/SYNTAX_CHANGE_GUIDE.md) |
| Find where a specific token is defined/used | [`.dev/TOKEN_OPERATOR_MAP.md`](.dev/TOKEN_OPERATOR_MAP.md) |
| Understand compiler pipeline & data flow | [`.dev/PIPELINE_ARCHITECTURE.md`](.dev/PIPELINE_ARCHITECTURE.md) |
| Find a specific file, function, or module | [`.dev/FILE_REFERENCE.md`](.dev/FILE_REFERENCE.md) |
| Find/update tests, know which syntax they use | [`.dev/TEST_INVENTORY.md`](.dev/TEST_INVENTORY.md) |

These files contain exact file:line references, grep patterns, and step-by-step checklists for common compiler modifications.

## Critical Rules

1. **No backward compatibility**: Axon is pre-release. Never keep old syntax around "for compatibility". When syntax changes, remove the old form entirely from lexer, parser, formatter, and tests.
2. **Shared interner**: `check_source_full()` in `src/lib.rs` returns the TypeChecker — MIR and codegen MUST use the same `TypeInterner` instance
3. **TAST lookups**: Uses `lookup_all()` (not `lookup()`) to search popped scopes
4. **Paths = `Vec<String>`** everywhere — no dedicated Path struct. Separator is reconstructed from string literals at display time
5. **String literal joins**: When changing syntax like `.` or `:`, the hardcoded string literals in fmt.rs, doc.rs, mir.rs, types.rs, typeck.rs, parser.rs, and codegen/abi.rs are the #1 thing missed. Always grep for the old string
6. **`model` and `val` are keywords**: Never use them as variable/field names in test code
7. **Run `cargo test` after every compiler change** — the test suite is comprehensive

## Current Syntax (Post-Overhaul)

| Old | New | Notes |
|-----|-----|-------|
| `::` | `.` | Path separator (uppercase-first = path, lowercase = field access) |
| `->` | `:` | Return type annotation |
| `let` / `let mut` | `val` / `var` | Immutable / mutable bindings |
| `impl` | `extend` | Implementation blocks |
| `struct` | `model` | Struct declaration (ML-first naming) |
| — | `\|>` | Pipe operator: `a \|> f(b)` → `f(a, b)` |
| — | `fn f(): T = expr;` | Expression shorthand for single-expression functions |
| — | implicit return | Last expression in fn body is returned (no `;` needed) |

## Build Environment
- Windows 11, MSVC toolchain
- Git's `link.exe` can shadow MSVC linker — set `LINK` env var if builds fail
- Tensor C runtime embedded in `src/codegen/runtime.rs`, compiled by clang at link time

## Documentation Site
- Source: `docs/docs.css` + `docs/docs.js` + markdown files
- Build: `cd docs && python build_docs.py` → `docs/documentation.html`
- Has interactive tooltips on syntax-highlighted code elements
