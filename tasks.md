# Axon Compiler — Task Tracking

## Project: Axon Programming Language Compiler

## Current Phase: 3 — Type Checker + Borrow Checker

---

## Tasks

### Phase 1: Project Setup

- [x] T001 [P] [Setup] Initialize Cargo project at H:\programing language\axon\ — `Cargo.toml`
- [x] T002 [P] [Setup] Configure dependencies (serde, serde_json, clap) — `Cargo.toml`
- [x] T003 [P] [Setup] Create directory structure — `src/`, `tests/`, `tests/examples/`

### Phase 2: Foundation

- [x] T004 [P] [Foundation] Implement source location tracking — `src/span.rs`
- [x] T005 [P] [Foundation] Implement token types for all keywords, types, operators — `src/token.rs`
- [x] T006 [P] [Foundation] Implement error types with structured output — `src/error.rs`
- [x] T007 [P] [Foundation] Define AST node types — `src/ast.rs`
- [x] T008 [P] [Foundation] Create library root — `src/lib.rs`

### Phase 3: Lexer Implementation

- [x] T009 [Lexer] Implement lexer core (advance, peek, position tracking) — `src/lexer.rs`
- [x] T010 [Lexer] Lex keywords (fn, let, mut, struct, enum, ...) — `src/lexer.rs`
- [x] T011 [Lexer] Lex built-in type names (Int8..Float64, Tensor, Vec, ...) — `src/lexer.rs`
- [x] T012 [Lexer] Lex operators (+, -, \*, /, @, %, ==, !=, &&, ||, ...) — `src/lexer.rs`
- [x] T013 [Lexer] Lex delimiters ((, ), {, }, [, ], ;, :, ,) — `src/lexer.rs`
- [x] T014 [Lexer] Lex integer literals (decimal, hex, binary, octal) — `src/lexer.rs`
- [x] T015 [Lexer] Lex float literals (decimal, scientific notation) — `src/lexer.rs`
- [x] T016 [Lexer] Lex string literals with escape sequences — `src/lexer.rs`
- [x] T017 [Lexer] Lex char literals — `src/lexer.rs`
- [x] T018 [Lexer] Lex single-line comments (//) — `src/lexer.rs`
- [x] T019 [Lexer] Lex multi-line comments (/\* \*/ with nesting) — `src/lexer.rs`
- [x] T020 [Lexer] Lex attributes (@cpu, @gpu, @device) — `src/lexer.rs`
- [x] T021 [Lexer] Lex identifiers — `src/lexer.rs`
- [x] T022 [Lexer] Lex &mut as a compound token — `src/lexer.rs`

### Phase 4: Parser Implementation

- [x] T023 [Parser] Implement parser core (advance, peek, expect, error recovery) — `src/parser.rs`
- [x] T024 [Parser] Parse function declarations (generics, params, return type, body) — `src/parser.rs`
- [x] T025 [Parser] Parse struct declarations (generics, fields) — `src/parser.rs`
- [x] T026 [Parser] Parse enum declarations (unit, tuple, struct variants) — `src/parser.rs`
- [x] T027 [Parser] Parse impl blocks (with optional trait) — `src/parser.rs`
- [x] T028 [Parser] Parse trait declarations (supertraits, methods) — `src/parser.rs`
- [x] T029 [Parser] Parse type aliases — `src/parser.rs`
- [x] T030 [Parser] Parse module declarations — `src/parser.rs`
- [x] T031 [Parser] Parse use declarations — `src/parser.rs`
- [x] T032 [Parser] Parse let statements (mut, type annotation, initializer) — `src/parser.rs`
- [x] T033 [Parser] Parse return statements — `src/parser.rs`
- [x] T034 [Parser] Parse while loops — `src/parser.rs`
- [x] T035 [Parser] Parse for loops — `src/parser.rs`
- [x] T036 [Parser] Parse if/else expressions (including else-if chains) — `src/parser.rs`
- [x] T037 [Parser] Parse match expressions with patterns — `src/parser.rs`
- [x] T038 [Parser] Parse binary operators with correct precedence — `src/parser.rs`
- [x] T039 [Parser] Parse @ (matmul) at same precedence as \* — `src/parser.rs`
- [x] T040 [Parser] Parse unary operators (-, !, &, &mut) — `src/parser.rs`
- [x] T041 [Parser] Parse postfix operators (?, ., (), []) — `src/parser.rs`
- [x] T042 [Parser] Parse type cast (as) — `src/parser.rs`
- [x] T043 [Parser] Parse function calls and method calls — `src/parser.rs`
- [x] T044 [Parser] Parse field access — `src/parser.rs`
- [x] T045 [Parser] Parse index and slice expressions — `src/parser.rs`
- [x] T046 [Parser] Parse path expressions (A::B::C) — `src/parser.rs`
- [x] T047 [Parser] Parse struct literals — `src/parser.rs`
- [x] T048 [Parser] Parse type expressions (primitives, generics, tensors, references, tuples, arrays) — `src/parser.rs`
- [x] T049 [Parser] Parse tensor types Tensor<DType, [shape]> with ?, int, and named dims — `src/parser.rs`
- [x] T050 [Parser] Parse patterns (identifier, literal, wildcard, tuple, struct, enum variant) — `src/parser.rs`
- [x] T051 [Parser] Parse error propagation (?) — `src/parser.rs`
- [x] T052 [Parser] Parse assignment operators (=, +=, -=, \*=, /=) — `src/parser.rs`
- [x] T053 [Parser] Parse range expressions (..) — `src/parser.rs`
- [x] T054 [Parser] Parse visibility (pub) — `src/parser.rs`
- [x] T055 [Parser] Parse attributes (@cpu, @gpu, @device) on items — `src/parser.rs`
- [x] T056 [Parser] Parse unsafe fn — `src/parser.rs`
- [x] T057 [Parser] Error recovery (synchronize on error, continue parsing) — `src/parser.rs`
- [x] T058 [Parser] Multiple error reporting (FR-045) — `src/parser.rs`

### Phase 5: CLI Implementation

- [x] T059 [CLI] Implement `axonc parse <file.axon>` command — `src/main.rs`
- [x] T060 [CLI] Implement `axonc lex <file.axon>` command — `src/main.rs`
- [x] T061 [CLI] Implement `--error-format=json` flag (FR-044) — `src/main.rs`
- [x] T062 [CLI] Implement `--help` output — `src/main.rs`

### Phase 6: Testing

- [x] T063 [Test] Lexer unit tests for all token types — `src/lexer.rs`
- [x] T064 [Test] Parser unit tests for all syntax constructs — `src/parser.rs`
- [x] T065 [Test] Integration tests for spec examples 1-8 — `tests/integration_tests.rs`
- [x] T066 [Test] Error message tests (malformed input) — `tests/integration_tests.rs`
- [x] T067 [Test] FR coverage tests (FR-001..FR-045) — `tests/integration_tests.rs`
- [x] T068 [Test] Edge case tests (empty program, deeply nested, etc.) — `tests/integration_tests.rs`
- [x] T069 [Test] Example .axon files for all 8 spec examples — `tests/examples/*.axon`

### Phase 3a: Type Infrastructure

- [x] T070 Define `Type` enum and `TypeId` interner — `src/types.rs`
- [x] T071 Define `PrimKind` for all built-in primitives — `src/types.rs`
- [x] T072 Define `TensorType` with shape representation — `src/types.rs`
- [x] T073 Implement `TypeInterner` arena — `src/types.rs`

### Phase 3b: Symbol Table

- [x] T074 Implement `Scope`, `SymbolTable`, `SymbolInfo` — `src/symbol.rs`
- [x] T075 Implement scope push/pop/define/lookup — `src/symbol.rs`
- [x] T076 Name resolution pass: collect top-level items — `src/symbol.rs`
- [x] T077 Name resolution pass: resolve use/import paths — `src/symbol.rs`
- [x] T078 Name resolution pass: resolve all identifiers and types — `src/symbol.rs`

### Phase 3c: Type Checker

- [x] T079 Implement constraint-based inference engine (TypeVar, unification) — `src/typeck.rs`
- [x] T080 Type check expressions (literals, binary ops, unary ops, calls) — `src/typeck.rs`
- [x] T081 Type check statements (let, return, while, for, assignment) — `src/typeck.rs`
- [x] T082 Type check items (functions, structs, enums, impls, traits) — `src/typeck.rs`
- [x] T083 Implement generic instantiation and bound checking — `src/typeck.rs`
- [x] T084 Implement trait resolution (inherent + trait impls) — `src/typeck.rs`
- [x] T085 Implement pattern type checking (match exhaustiveness basic) — `src/typeck.rs`
- [x] T086 Type coercion rules (&mut→&, auto-borrow) — `src/typeck.rs`

### Phase 3d: Shape Checker

- [x] T087 Implement shape unification (Known, Dynamic, Variable) — `src/shapes.rs`
- [x] T088 Implement matmul shape rule (inner dims match) — `src/shapes.rs`
- [x] T089 Implement elementwise broadcast rules — `src/shapes.rs`
- [x] T090 Implement reshape/transpose shape rules — `src/shapes.rs`
- [x] T091 Insert runtime shape checks for dynamic dims — `src/shapes.rs`

### Phase 3e: Borrow Checker

- [x] T092 Build control flow graph from typed AST — `src/borrow.rs`
- [x] T093 Compute variable liveness — `src/borrow.rs`
- [x] T094 Track borrows and moves — `src/borrow.rs`
- [x] T095 Enforce exclusivity (no &mut + & overlap) — `src/borrow.rs`
- [x] T096 Enforce move semantics (use-after-move detection) — `src/borrow.rs`
- [x] T097 Lifetime inference and validation — `src/borrow.rs`
- [x] T098 Device-aware borrow rules (@cpu/@gpu) — `src/borrow.rs`

### Phase 3f: Typed AST & Integration

- [x] T099 Define TAST node types — `src/tast.rs`
- [x] T100 Build TAST from AST + type info — `src/tast.rs`
- [x] T101 Integrate into `lib.rs` pipeline — `src/lib.rs`
- [x] T102 CLI `axonc check` command — `src/main.rs`

### Phase 3g: Testing

- [x] T103 Unit tests for type unification — `src/typeck.rs`
- [x] T104 Unit tests for shape checking — `src/shapes.rs`
- [x] T105 Unit tests for borrow checking — `src/borrow.rs`
- [x] T106 Integration tests for full programs — `tests/type_tests.rs`
- [x] T107 Error message tests for all E-codes — `tests/type_tests.rs`
- [x] T108 Edge case tests (recursive types, complex generics) — `tests/type_tests.rs`

---

## Summary

- **Total Tasks**: 108
- **Completed**: 108
- **Remaining**: 0
