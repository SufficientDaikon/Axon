# Phase 3: Type Checker + Borrow Checker — Implementation Specification

## 1. Overview

Phase 3 adds semantic analysis to the Axon compiler. The input is a parsed AST (from Phase 2); the output is a **Typed AST (TAST)** where every expression, statement, and pattern carries a resolved type, every name is resolved to its declaration, and ownership/borrowing invariants have been verified.

### Deliverables

| Artifact        | Crate Path           | Purpose                                                                         |
| --------------- | -------------------- | ------------------------------------------------------------------------------- |
| `src/types.rs`  | Type representation  | Internal type system: primitives, generics, tensors, references, function types |
| `src/symbol.rs` | Symbol table         | Scoped name → type/declaration mapping                                          |
| `src/typeck.rs` | Type checker         | Hindley-Milner–style inference, unification, trait resolution                   |
| `src/shapes.rs` | Tensor shape checker | Compile-time shape arithmetic and verification (FR-012, FR-013, FR-014)         |
| `src/borrow.rs` | Borrow checker       | Ownership tracking, lifetime analysis, move semantics (FR-015, FR-016)          |
| `src/tast.rs`   | Typed AST            | Mirror of `ast.rs` where every node carries a resolved `TypeId`                 |

### Dependencies

- Phase 2 outputs: `ast.rs`, `span.rs`, `error.rs`

---

## 2. Type Representation (`src/types.rs`)

### 2.1 Core Type Enum

```
Type
├── Primitive(PrimKind)          // Int8..Float64, Bool, Char, String
├── Tensor { dtype, shape }      // FR-011: Tensor<DType, [Shape]>
├── Tuple(Vec<TypeId>)
├── Array { elem, size }
├── Reference { mutable, inner } // FR-015: &T, &mut T
├── Function { params, ret }     // fn(A, B) -> C
├── Struct { name, fields, generics }
├── Enum { name, variants, generics }
├── Trait { name, methods, supertraits }
├── Generic(GenericVar)          // Unresolved generic parameter T
├── TypeVar(u32)                 // Inference variable (unification)
├── Named { path, args }        // User-defined with applied generics
├── Option(TypeId)               // Option<T>
├── Result(TypeId, TypeId)       // Result<T, E>
├── Never                        // ! (diverging)
├── Unit                         // ()
└── Error                        // Poison type for error recovery
```

### 2.2 PrimKind

```rust
enum PrimKind {
    Int8, Int16, Int32, Int64,
    UInt8, UInt16, UInt32, UInt64,
    Float16, Float32, Float64,
    Bool, Char, String,
}
```

### 2.3 TypeId & Interner

All types are stored in a `TypeInterner` arena. A `TypeId` is a lightweight handle (u32 index). This avoids deep cloning and allows O(1) equality checks on resolved types.

```rust
struct TypeInterner {
    types: Vec<Type>,
}

impl TypeInterner {
    fn intern(&mut self, ty: Type) -> TypeId;
    fn resolve(&self, id: TypeId) -> &Type;
}
```

### 2.4 Tensor Shape Representation

```rust
enum ShapeDim {
    Known(i64),        // e.g., 128
    Dynamic,           // ? (FR-014)
    Variable(String),  // Named dim: N, batch
    Inferred,          // To be resolved by unification
}

struct TensorType {
    dtype: TypeId,         // Must resolve to a numeric primitive
    shape: Vec<ShapeDim>,
}
```

---

## 3. Symbol Table (`src/symbol.rs`)

### 3.1 Design

Lexical scoping with a stack of `Scope` frames. Each scope maps `String → Symbol`.

```rust
struct SymbolTable {
    scopes: Vec<Scope>,
    symbols: Vec<SymbolInfo>,
}

struct Scope {
    parent: Option<ScopeId>,
    bindings: HashMap<String, SymbolId>,
    kind: ScopeKind,  // Module, Function, Block, Impl, Trait
}

struct SymbolInfo {
    name: String,
    ty: TypeId,
    kind: SymbolKind,
    mutable: bool,
    span: Span,
    scope: ScopeId,
}

enum SymbolKind {
    Variable,
    Function,
    Parameter,
    StructDef,
    EnumDef,
    TraitDef,
    TypeAlias,
    Module,
    Field,
    Method,
    GenericParam,
}
```

### 3.2 Scope Operations

| Operation                         | Description                   |
| --------------------------------- | ----------------------------- |
| `push_scope(kind)`                | Enter a new scope             |
| `pop_scope()`                     | Exit current scope            |
| `define(name, symbol)`            | Add binding to current scope  |
| `lookup(name) → Option<SymbolId>` | Walk up scope chain           |
| `lookup_in(scope, name)`          | Lookup in specific scope only |

### 3.3 Name Resolution Pass

Before type checking, perform a full name resolution pass:

1. **Collect all top-level items** (structs, enums, traits, functions, type aliases, modules) — register names without resolving bodies.
2. **Resolve use declarations** — map imported paths to their target symbols.
3. **Resolve type expressions** — map `TypeExpr::Named("Vec")` → `SymbolId` for Vec.
4. **Resolve expressions** — map `ExprKind::Identifier("x")` → `SymbolId`.

**Error codes:**

- `E1001`: Undefined name
- `E1002`: Duplicate definition in same scope
- `E1003`: Ambiguous import
- `E1004`: Private item accessed outside module

---

## 4. Type Checker (`src/typeck.rs`)

### 4.1 Inference Engine

Use a **constraint-based Hindley-Milner** approach:

1. **Generate fresh type variables** for every `let` binding without annotation, every function return type that is inferred, and every generic parameter instantiation.
2. **Collect constraints** by walking the TAST (equalities and subtype bounds).
3. **Unify** — apply Robinson's unification algorithm on type variables.
4. **Generalize** — after checking a function body, generalize remaining free type variables into generic parameters (let-polymorphism).

### 4.2 Typing Rules (Selected)

| Construct               | Rule                                                                              |
| ----------------------- | --------------------------------------------------------------------------------- |
| `let x = expr;`         | Infer type of `expr`, assign to `x`                                               |
| `let x: T = expr;`      | Check `expr` against `T`                                                          |
| `x + y`                 | Both operands must have same numeric type; result is same type                    |
| `x @ y` (FR-002)        | Both operands must be `Tensor`; apply matmul shape rules                          |
| `fn call f(args)`       | Instantiate generic params, check args against param types, result is return type |
| `obj.method(args)`      | Resolve method via impl/trait lookup, check args                                  |
| `if c { a } else { b }` | `c: Bool`, `a` and `b` must unify to same type                                    |
| `match expr { arms }`   | All arm patterns must cover `expr` type, all arm bodies must unify                |
| `expr?`                 | `expr` must be `Result<T, E>` or `Option<T>`; propagate `E` / `None`              |
| `&expr`                 | Result is `&T` where `expr: T`                                                    |
| `&mut expr`             | Result is `&mut T`; `expr` must be mutable lvalue                                 |
| `expr as T`             | Validate numeric cast rules                                                       |

### 4.3 Trait Resolution

```
Given: expr.method(args) where expr: SomeType

1. Look up inherent impls:  impl SomeType { fn method(...) }
2. Look up trait impls:     impl SomeTrait for SomeType { fn method(...) }
3. Check trait bounds on generic params
4. If ambiguous → error E2010
5. If not found → error E2011
```

**Trait coherence rules:**

- At most one impl of a trait for a given type in the current crate.
- Blanket impls are allowed with appropriate bounds.
- Orphan rule: either the trait or the type must be local.

### 4.4 Generic Instantiation

When calling a generic function `fn foo<T: Bound>(x: T)`:

1. Create fresh type variable `?T`.
2. Record constraint `?T: Bound`.
3. Unify argument types with `?T`.
4. After unification, check that resolved `?T` satisfies all bounds.

### 4.5 Type Coercions

| From             | To             | Condition                       |
| ---------------- | -------------- | ------------------------------- |
| `&mut T`         | `&T`           | Always                          |
| `T`              | `&T`           | Auto-borrow in method receivers |
| `&T`             | `&mut T`       | **Never** (error E2020)         |
| Numeric widening | `Int8 → Int32` | Only with explicit `as`         |

---

## 5. Tensor Shape Checker (`src/shapes.rs`)

### 5.1 Shape Arithmetic (FR-012, FR-013)

| Operation             | Input Shapes              | Output Shape    | Rule                       |
| --------------------- | ------------------------- | --------------- | -------------------------- |
| `a + b` (elementwise) | `[M, N]` + `[M, N]`       | `[M, N]`        | Exact match or broadcast   |
| `a @ b` (matmul)      | `[M, K]` @ `[K, N]`       | `[M, N]`        | Inner dims must match      |
| `a @ b` (batched)     | `[B, M, K]` @ `[B, K, N]` | `[B, M, N]`     | Batch dims must match      |
| `transpose(a)`        | `[M, N]`                  | `[N, M]`        | Reverse last two dims      |
| `reshape(a, shape)`   | `[*]` → `[*]`             | New shape       | Product of dims must match |
| `broadcast(a, b)`     | Per numpy rules           | Broadcast shape | Dims must be 1 or equal    |

### 5.2 Shape Unification

Named dimensions unify:

- `Known(128)` with `Known(128)` → `Known(128)` ✓
- `Known(128)` with `Known(256)` → **Error E3001**: shape mismatch
- `Known(128)` with `Variable("N")` → bind `N = 128`
- `Variable("N")` with `Variable("M")` → unify `N = M`
- `Dynamic` with anything → `Dynamic` (always succeeds)
- `Known(_)` with `Dynamic` → `Dynamic` (widening)

### 5.3 Compile-Time vs. Runtime

- **Known + Known** → compile-time verified ✓
- **Variable + Variable** → compile-time if same binding, else deferred
- **Dynamic involved** → insert runtime shape check at call site

### 5.4 Error Codes

| Code    | Message                                                              |
| ------- | -------------------------------------------------------------------- |
| `E3001` | Tensor shape mismatch: expected `[M, K]`, found `[M, N]` where K ≠ N |
| `E3002` | Matmul inner dimensions don't match                                  |
| `E3003` | Cannot broadcast shapes `[A]` and `[B]`                              |
| `E3004` | Reshape total element count mismatch                                 |
| `E3005` | Invalid tensor dtype: expected numeric, found `Bool`                 |
| `E3006` | Dynamic dimension where static required                              |

---

## 6. Borrow Checker (`src/borrow.rs`)

### 6.1 Ownership Model (FR-015, FR-016)

Axon follows Rust's ownership model:

1. **Each value has exactly one owner.**
2. **When the owner goes out of scope, the value is dropped.**
3. **At any time, you can have either:**
   - One mutable reference (`&mut T`), OR
   - Any number of immutable references (`&T`).
4. **References must always be valid** (no dangling references).

### 6.2 Move Semantics

- **Types that are `Copy`**: all primitives (`Int*`, `UInt*`, `Float*`, `Bool`, `Char`).
- **Types that move**: `String`, `Vec<T>`, `Tensor<D, S>`, all user-defined structs/enums (unless they implement `Copy`).
- After a move, the source variable is invalidated.

### 6.3 Borrow Analysis Algorithm

Use a simplified **MIR-like control flow graph** (CFG):

1. **Build CFG** from the typed AST: basic blocks with edges for branches, loops, match arms.
2. **Compute liveness** for each variable: at each program point, which variables are live.
3. **Track borrows**: for each reference creation, record the borrowed place, mutability, and lifetime span.
4. **Check invariants** at each program point:
   - No mutable borrow while immutable borrows exist for the same place.
   - No use of a moved value.
   - No mutable borrow of an immutable binding.
   - References do not outlive their referent.

### 6.4 Lifetime Inference

Axon uses **fully inferred lifetimes** (no explicit lifetime annotations in user code). The compiler:

1. Assigns a fresh lifetime variable to every reference.
2. Collects constraints from assignments, function calls, and returns.
3. Solves constraints to determine minimum lifetimes.
4. Reports errors when constraints are unsatisfiable.

### 6.5 Device-Aware Ownership (@cpu, @gpu, @device)

- Tensors annotated with `@gpu` cannot be borrowed as `&mut` from `@cpu` context.
- Transferring a tensor from CPU to GPU is a **move** (invalidates the CPU-side binding).
- `@device(expr)` dynamically selects the device; borrow checking is conservative.

### 6.6 Error Codes

| Code    | Message                                                               |
| ------- | --------------------------------------------------------------------- |
| `E4001` | Use of moved value `x`                                                |
| `E4002` | Cannot borrow `x` as mutable because it is also borrowed as immutable |
| `E4003` | Cannot borrow `x` as mutable more than once                           |
| `E4004` | Cannot assign to immutable variable `x`                               |
| `E4005` | Reference to `x` escapes its scope                                    |
| `E4006` | Cannot transfer `@gpu` tensor to `@cpu` context without explicit copy |
| `E4007` | Cannot borrow `@gpu` tensor as `&mut` from `@cpu` function            |

---

## 7. Typed AST (`src/tast.rs`)

Mirror the structure of `ast.rs` but annotate every node:

```rust
struct TypedExpr {
    kind: TypedExprKind,
    ty: TypeId,           // Resolved type
    span: Span,
}

struct TypedStmt {
    kind: TypedStmtKind,
    span: Span,
}

struct TypedItem {
    kind: TypedItemKind,
    span: Span,
    visibility: Visibility,
    attributes: Vec<Attribute>,
}
```

Every node that existed in `ast.rs` gets a typed counterpart. The TAST is the input to Phase 4 (code generation).

---

## 8. CLI Integration

Extend `src/main.rs`:

```
axonc check <file.axon>          # Run type checker + borrow checker
axonc check --emit-tast <file>   # Dump typed AST as JSON
```

### Error Format

All errors use the existing `CompileError` structure with new error code ranges:

- `E1xxx`: Name resolution errors
- `E2xxx`: Type checking errors
- `E3xxx`: Shape checking errors
- `E4xxx`: Borrow checking errors

---

## 9. Task Breakdown

### Phase 3a: Type Infrastructure

- [ ] T070 Define `Type` enum and `TypeId` interner — `src/types.rs`
- [ ] T071 Define `PrimKind` for all built-in primitives — `src/types.rs`
- [ ] T072 Define `TensorType` with shape representation — `src/types.rs`
- [ ] T073 Implement `TypeInterner` arena — `src/types.rs`

### Phase 3b: Symbol Table

- [ ] T074 Implement `Scope`, `SymbolTable`, `SymbolInfo` — `src/symbol.rs`
- [ ] T075 Implement scope push/pop/define/lookup — `src/symbol.rs`
- [ ] T076 Name resolution pass: collect top-level items — `src/symbol.rs`
- [ ] T077 Name resolution pass: resolve use/import paths — `src/symbol.rs`
- [ ] T078 Name resolution pass: resolve all identifiers and types — `src/symbol.rs`

### Phase 3c: Type Checker

- [ ] T079 Implement constraint-based inference engine (TypeVar, unification) — `src/typeck.rs`
- [ ] T080 Type check expressions (literals, binary ops, unary ops, calls) — `src/typeck.rs`
- [ ] T081 Type check statements (let, return, while, for, assignment) — `src/typeck.rs`
- [ ] T082 Type check items (functions, structs, enums, impls, traits) — `src/typeck.rs`
- [ ] T083 Implement generic instantiation and bound checking — `src/typeck.rs`
- [ ] T084 Implement trait resolution (inherent + trait impls) — `src/typeck.rs`
- [ ] T085 Implement pattern type checking (match exhaustiveness basic) — `src/typeck.rs`
- [ ] T086 Type coercion rules (&mut→&, auto-borrow) — `src/typeck.rs`

### Phase 3d: Shape Checker

- [ ] T087 Implement shape unification (Known, Dynamic, Variable) — `src/shapes.rs`
- [ ] T088 Implement matmul shape rule (inner dims match) — `src/shapes.rs`
- [ ] T089 Implement elementwise broadcast rules — `src/shapes.rs`
- [ ] T090 Implement reshape/transpose shape rules — `src/shapes.rs`
- [ ] T091 Insert runtime shape checks for dynamic dims — `src/shapes.rs`

### Phase 3e: Borrow Checker

- [ ] T092 Build control flow graph from typed AST — `src/borrow.rs`
- [ ] T093 Compute variable liveness — `src/borrow.rs`
- [ ] T094 Track borrows and moves — `src/borrow.rs`
- [ ] T095 Enforce exclusivity (no &mut + & overlap) — `src/borrow.rs`
- [ ] T096 Enforce move semantics (use-after-move detection) — `src/borrow.rs`
- [ ] T097 Lifetime inference and validation — `src/borrow.rs`
- [ ] T098 Device-aware borrow rules (@cpu/@gpu) — `src/borrow.rs`

### Phase 3f: Typed AST & Integration

- [ ] T099 Define TAST node types — `src/tast.rs`
- [ ] T100 Build TAST from AST + type info — `src/tast.rs`
- [ ] T101 Integrate into `lib.rs` pipeline — `src/lib.rs`
- [ ] T102 CLI `axonc check` command — `src/main.rs`

### Phase 3g: Testing

- [ ] T103 Unit tests for type unification — `src/typeck.rs`
- [ ] T104 Unit tests for shape checking — `src/shapes.rs`
- [ ] T105 Unit tests for borrow checking — `src/borrow.rs`
- [ ] T106 Integration tests for full programs — `tests/type_tests.rs`
- [ ] T107 Error message tests for all E-codes — `tests/type_tests.rs`
- [ ] T108 Edge case tests (recursive types, complex generics) — `tests/type_tests.rs`

---

## 10. Acceptance Criteria

- [ ] All valid Phase 2 example programs type-check without errors
- [ ] Tensor shape mismatches produce clear compile-time errors
- [ ] Use-after-move is caught at compile time
- [ ] `&mut` aliasing is caught at compile time
- [ ] Generic functions instantiate correctly with inferred types
- [ ] Trait method calls resolve through impl blocks
- [ ] `@gpu`/`@cpu` device annotations are enforced
- [ ] All errors include span, error code, and suggestion where applicable
- [ ] Typed AST JSON output matches expected schema
- [ ] 100+ new tests pass
