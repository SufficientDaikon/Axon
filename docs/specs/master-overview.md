# Axon Compiler — Master Implementation Specification

## Executive Summary

Axon is a systems programming language designed for AI/ML with first-class tensors, compile-time shape checking, ownership-based memory safety, and native GPU execution. This document is the **master specification** that orchestrates all implementation phases from language design through public release.

**Current Status:** Phase 2 complete (Lexer + Parser + AST, 69 tasks, 102 tests passing).

---

## Architecture Overview

```
                        ┌───────────────────────────────────────────────┐
                        │              Axon Compiler Pipeline           │
                        └───────────────────────────────────────────────┘

  Source Code (.axon)
        │
        ▼
  ┌─────────────┐     Phase 2 (COMPLETE)
  │    Lexer     │     src/lexer.rs — Tokenizes source into TokenKind stream
  └──────┬──────┘
         ▼
  ┌─────────────┐     Phase 2 (COMPLETE)
  │   Parser     │     src/parser.rs — Recursive descent → AST
  └──────┬──────┘
         ▼
  ┌─────────────┐     Phase 3
  │Name Resolver │     src/symbol.rs — Scoped name → symbol resolution
  └──────┬──────┘
         ▼
  ┌─────────────┐     Phase 3
  │ Type Checker │     src/typeck.rs — Hindley-Milner inference + unification
  └──────┬──────┘
         ▼
  ┌─────────────┐     Phase 3
  │Shape Checker │     src/shapes.rs — Tensor dim verification (FR-012..014)
  └──────┬──────┘
         ▼
  ┌─────────────┐     Phase 3
  │Borrow Checker│     src/borrow.rs — Ownership + lifetime analysis (FR-015..016)
  └──────┬──────┘
         ▼
  ┌─────────────┐
  │  Typed AST   │     src/tast.rs — Every node carries TypeId
  └──────┬──────┘
         ▼
  ┌─────────────┐     Phase 4
  │  Axon MIR    │     src/mir.rs — Desugared, explicit drops, CFG
  └──────┬──────┘
         │
    ┌────┴────┐
    ▼         ▼
┌───────┐ ┌───────┐   Phase 4
│ LLVM  │ │ MLIR  │   src/codegen/llvm.rs — CPU native code
│Backend│ │Backend│   src/codegen/mlir.rs — GPU kernels (CUDA/ROCm/Vulkan)
└───┬───┘ └───┬───┘
    │         │
    ▼         ▼
  Native    GPU          Phase 4
  Binary    Kernels      Runtime: tensor alloc, device dispatch, panic handler
    │         │
    ▼         ▼
  ┌─────────────┐     Phase 5
  │   Stdlib     │     stdlib/ — Collections, I/O, math, tensor ops, sync
  └─────────────┘
         │
         ▼
  ┌─────────────┐     Phase 6
  │AI Framework  │     stdlib/nn/, autograd/, optim/, loss/ — Built-in ML
  └─────────────┘
         │
         ▼
  ┌─────────────┐     Phase 7
  │   Tooling    │     LSP, pkg manager, REPL, debugger, formatter, VS Code
  └─────────────┘
         │
         ▼
  ┌─────────────┐     Phase 8
  │  Release     │     Benchmarks, fuzzing, security, docs, binaries
  └─────────────┘
```

---

## Phase Summary

| Phase | Name                          | Tasks     | Key Output                                                                | Status      |
| ----- | ----------------------------- | --------- | ------------------------------------------------------------------------- | ----------- |
| 0     | Language Design Study         | —         | Research findings                                                         | ✅ Complete |
| 1     | Language Specification        | —         | 72 FR, 28 NFR, 984-line spec                                              | ✅ Complete |
| 2     | Lexer + Parser + AST          | T001–T069 | `lexer.rs`, `parser.rs`, `ast.rs`, 102 tests                              | ✅ Complete |
| 3     | Type Checker + Borrow Checker | T070–T108 | `types.rs`, `symbol.rs`, `typeck.rs`, `shapes.rs`, `borrow.rs`, `tast.rs` | 🔲 Next     |
| 4     | LLVM/MLIR Code Generation     | T109–T137 | `mir.rs`, `codegen/llvm.rs`, `codegen/mlir.rs`, native binaries           | 🔲 Planned  |
| 5     | Standard Library              | T138–T169 | `stdlib/` — collections, tensor ops, I/O, sync                            | 🔲 Planned  |
| 6     | AI Framework                  | T170–T204 | `stdlib/nn/`, `autograd/`, `optim/`, `loss/`, ONNX export                 | 🔲 Planned  |
| 7     | Tooling                       | T205–T245 | LSP, package manager, REPL, debugger, VS Code ext                         | 🔲 Planned  |
| 8     | Hardening & Release           | T246–T277 | Benchmarks, fuzzing, security, docs, release binaries                     | 🔲 Planned  |

**Total tasks across all phases: 277**

---

## Phase Dependency Graph

```
Phase 0 ──→ Phase 1 ──→ Phase 2 ──→ Phase 3 ──→ Phase 4 ──→ Phase 5 ──→ Phase 6
                                         │            │            │           │
                                         │            │            │           │
                                         └────────────┴────────────┴───────────┘
                                                           │
                                                           ▼
                                                       Phase 7 (Tooling)
                                                           │
                                                           ▼
                                                       Phase 8 (Release)
```

**Key dependencies:**

- Phase 3 requires Phase 2 (AST input)
- Phase 4 requires Phase 3 (Typed AST input)
- Phase 5 requires Phase 3 + 4 (stdlib must type-check and compile)
- Phase 6 requires Phase 5 (AI framework built on stdlib tensors)
- Phase 7 requires all of 2–6 (tools wrap the full pipeline)
- Phase 8 requires all of 0–7 (hardening and release)

**Parallelization opportunities:**

- Phase 7 LSP + formatter can start during Phase 3 (only needs lexer/parser initially)
- Phase 5 type signatures can be designed during Phase 3 (impl after Phase 4)
- Phase 8 CI/CD setup can start during Phase 5
- Documentation writing can happen continuously from Phase 3 onward

---

## Phase 3: Type Checker + Borrow Checker

**Spec:** [`docs/specs/phase3-type-checker.md`](phase3-type-checker.md)

### Summary

Adds semantic analysis: name resolution, type inference (Hindley-Milner), trait resolution, compile-time tensor shape verification, and Rust-style borrow checking with device-aware ownership for `@gpu`/`@cpu` tensors.

### Key Design Decisions

- **Type interning** via `TypeId` arena for O(1) equality and low memory.
- **Constraint-based inference** — generate type variables, collect constraints, then unify.
- **Tensor shapes** use a 3-tier system: `Known`, `Variable` (named generic), `Dynamic` (?).
- **Borrow checker** builds a CFG from the typed AST and tracks borrows/moves per program point.
- **Lifetimes are fully inferred** — no explicit lifetime annotations in user code.

### Files Created

| File            | Lines (est.) | Purpose                                      |
| --------------- | ------------ | -------------------------------------------- |
| `src/types.rs`  | ~400         | Type representation + interner               |
| `src/symbol.rs` | ~500         | Scoped symbol table + name resolution        |
| `src/typeck.rs` | ~1200        | Type inference + checking + trait resolution |
| `src/shapes.rs` | ~400         | Tensor shape arithmetic + verification       |
| `src/borrow.rs` | ~800         | CFG construction + borrow/move analysis      |
| `src/tast.rs`   | ~300         | Typed AST nodes                              |

### Error Codes Introduced

- `E1xxx`: Name resolution (undefined, duplicate, ambiguous, private)
- `E2xxx`: Type checking (mismatch, unresolved, ambiguous method)
- `E3xxx`: Shape checking (mismatch, broadcast fail, reshape fail)
- `E4xxx`: Borrow checking (use-after-move, alias violation, lifetime escape)

### Tasks: T070–T108 (39 tasks)

---

## Phase 4: LLVM/MLIR Code Generation

**Spec:** [`docs/specs/phase4-codegen.md`](phase4-codegen.md)

### Summary

Lowers the Typed AST through an Axon MIR intermediate representation to LLVM IR (for CPU) and MLIR (for GPU). Produces native binaries with optional CUDA/ROCm/Vulkan GPU kernel compilation.

### Key Design Decisions

- **Axon MIR** sits between TAST and LLVM — desugars match/for/method calls, makes drops explicit.
- **LLVM backend** via `inkwell` crate — produces native binaries for x86_64 and aarch64.
- **MLIR backend** via `melior` crate — lowers `@gpu` functions through Linalg → MemRef → GPU dialect → PTX/SPIR-V.
- **Symbol mangling** uses a custom `_AX` scheme for Axon-to-Axon linking.
- **FFI** uses C calling convention for interop with existing libraries.

### Files Created

| File                     | Lines (est.) | Purpose                                   |
| ------------------------ | ------------ | ----------------------------------------- |
| `src/mir.rs`             | ~600         | MIR data structures + TAST → MIR lowering |
| `src/codegen/mod.rs`     | ~100         | Codegen orchestration                     |
| `src/codegen/llvm.rs`    | ~1500        | LLVM IR generation                        |
| `src/codegen/mlir.rs`    | ~800         | MLIR dialect lowering                     |
| `src/codegen/runtime.rs` | ~400         | Runtime support functions                 |
| `src/codegen/abi.rs`     | ~300         | Symbol mangling + FFI                     |

### Tasks: T109–T137 (29 tasks)

---

## Phase 5: Standard Library

**Spec:** [`docs/specs/phase5-stdlib.md`](phase5-stdlib.md)

### Summary

The Axon standard library: core traits (operator overloading, iterators, conversions), collections (Vec, HashMap, HashSet), full tensor operations (creation, shape ops, reductions, linalg), I/O, math, concurrency primitives, and an ML data loading pipeline.

### Key Design Decisions

- **Prelude** auto-imports essential types/traits without `use`.
- **Operator traits** (Add, Mul, MatMul, Index) enable operator overloading via `impl`.
- **Iterator** trait with rich default methods (map, filter, fold, collect, zip, etc.).
- **Tensor API** mirrors NumPy/PyTorch naming for familiarity.
- **Linear algebra** is a sub-module of tensor (`std::tensor::linalg`).
- **DataLoader** provides batching + shuffling for ML training loops.

### Module Count: 19 modules

### Tasks: T138–T169 (32 tasks)

---

## Phase 6: AI Framework

**Spec:** [`docs/specs/phase6-ai-framework.md`](phase6-ai-framework.md)

### Summary

A built-in AI/ML framework with define-by-run autograd, neural network layers (Linear, Conv2d, LSTM, Transformer), optimizers (SGD, Adam, AdamW), loss functions, training utilities, metrics, ONNX export, and data transforms. Leverages Axon's compile-time shape checking for safe model construction.

### Key Design Decisions

- **Autograd** uses a computation graph with reverse-mode automatic differentiation (like PyTorch).
- **GradTensor** wraps Tensor with gradient tracking — distinct type from Tensor.
- **Module trait** standardizes layer interface (forward, parameters, train/eval, save/load).
- **Sequential container** composes layers (like `nn.Sequential` in PyTorch).
- **Compile-time shape safety**: `Linear<784, 256>` connecting to `Linear<256, 10>` — dimension mismatch is a compile error.
- **ONNX export** traces the computation graph to produce industry-standard model files.

### Tasks: T170–T204 (35 tasks)

---

## Phase 7: Tooling

**Spec:** [`docs/specs/phase7-tooling.md`](phase7-tooling.md)

### Summary

Developer experience tools: LSP server (completions, diagnostics, go-to-def, hover with tensor shapes), package manager with Axon.toml manifests, JIT-powered REPL, DAP debugger with tensor visualization, opinionated formatter, configurable linter, VS Code extension, and documentation generator.

### Key Design Decisions

- **LSP** uses incremental parsing + type checking for real-time feedback (< 200ms on edit).
- **Package manager** uses `Axon.toml` (TOML manifest) with semver resolution and a central registry.
- **REPL** uses LLVM ORC JIT for fast expression evaluation with persistent state.
- **Debugger** implements DAP (Debug Adapter Protocol) wrapping LLDB.
- **Formatter** is opinionated and non-configurable (like `gofmt`): parse → AST → re-emit.
- **VS Code extension** is the primary IDE target, with TextMate grammar + LSP client.

### Tasks: T205–T245 (41 tasks)

---

## Phase 8: Hardening & Release

**Spec:** [`docs/specs/phase8-hardening.md`](phase8-hardening.md)

### Summary

Final quality phase: comprehensive benchmarks (compiler speed, runtime performance vs PyTorch/NumPy), 48-hour fuzz campaigns on all compiler passes, security audit of unsafe code, full documentation site with tutorials, CI/CD pipeline, prebuilt binaries for 5 platforms, and specification compliance verification.

### Key Design Decisions

- **Benchmarks** compare against PyTorch, NumPy, and C for performance validation.
- **Fuzzing** uses both random and grammar-based structured approaches.
- **Security** requires `// SAFETY:` comments on every `unsafe` block.
- **Documentation** includes a user guide, 6 ML tutorials, API reference, and migration guides.
- **Release** targets 5 platforms: Linux x86_64/aarch64, macOS x86_64/aarch64, Windows x86_64.
- **Compliance** verifies all 72 FR and 28 NFR from the original specification.

### Tasks: T246–T277 (32 tasks)

---

## Complete Task Index

| Range     | Phase | Count | Category                             |
| --------- | ----- | ----- | ------------------------------------ |
| T001–T003 | 2     | 3     | Project setup                        |
| T004–T008 | 2     | 5     | Foundation (span, token, error, AST) |
| T009–T022 | 2     | 14    | Lexer implementation                 |
| T023–T058 | 2     | 36    | Parser implementation                |
| T059–T062 | 2     | 4     | CLI (parse, lex commands)            |
| T063–T069 | 2     | 7     | Phase 2 testing                      |
| T070–T073 | 3     | 4     | Type infrastructure                  |
| T074–T078 | 3     | 5     | Symbol table + name resolution       |
| T079–T086 | 3     | 8     | Type checker                         |
| T087–T091 | 3     | 5     | Shape checker                        |
| T092–T098 | 3     | 7     | Borrow checker                       |
| T099–T102 | 3     | 4     | Typed AST + integration              |
| T103–T108 | 3     | 6     | Phase 3 testing                      |
| T109–T113 | 4     | 5     | MIR                                  |
| T114–T121 | 4     | 8     | LLVM backend                         |
| T122–T126 | 4     | 5     | MLIR backend                         |
| T127–T130 | 4     | 4     | Runtime + ABI                        |
| T131–T137 | 4     | 7     | Phase 4 CLI + testing                |
| T138–T142 | 5     | 5     | Core traits + prelude                |
| T143–T147 | 5     | 5     | Collections                          |
| T148–T153 | 5     | 6     | Tensor operations                    |
| T154–T157 | 5     | 4     | Math + I/O                           |
| T158–T164 | 5     | 7     | Concurrency + data loading           |
| T165–T169 | 5     | 5     | Phase 5 testing                      |
| T170–T174 | 6     | 5     | Autograd engine                      |
| T175–T183 | 6     | 9     | Neural network layers                |
| T184–T188 | 6     | 5     | Optimizers + loss functions          |
| T189–T193 | 6     | 5     | Training + export                    |
| T194–T197 | 6     | 4     | Metrics + transforms                 |
| T198–T204 | 6     | 7     | Phase 6 testing                      |
| T205–T214 | 7     | 10    | Language server                      |
| T215–T221 | 7     | 7     | Package manager                      |
| T222–T226 | 7     | 5     | REPL                                 |
| T227–T231 | 7     | 5     | Debugger                             |
| T232–T235 | 7     | 4     | Formatter + linter + docs            |
| T236–T240 | 7     | 5     | VS Code extension                    |
| T241–T245 | 7     | 5     | Phase 7 testing                      |
| T246–T250 | 8     | 5     | Benchmarks                           |
| T251–T255 | 8     | 5     | Fuzz testing                         |
| T256–T260 | 8     | 5     | Security                             |
| T261–T269 | 8     | 9     | Documentation                        |
| T270–T274 | 8     | 5     | Release engineering                  |
| T275–T277 | 8     | 3     | Compliance                           |

---

## Recommended Implementation Order

### Step 1: Phase 3 — Semantic Analysis (Foundation for everything)

**Why first:** Nothing can compile or run without types. This is the most critical phase.

```
3a: Type infrastructure (types.rs, TypeId interner)
    ↓
3b: Symbol table + name resolution (symbol.rs)
    ↓
3c: Type checker (typeck.rs) — inference, unification, trait resolution
    ↓
3d: Shape checker (shapes.rs) — tensor dim verification
    ↓
3e: Borrow checker (borrow.rs) — ownership + lifetime analysis
    ↓
3f: Typed AST + CLI integration
    ↓
3g: Testing (100+ tests)
```

### Step 2: Phase 4 — Code Generation (Make things run)

**Why second:** After type checking, you need to produce executables.

```
4a: MIR lowering (TAST → MIR)
    ↓
4b: LLVM backend — CPU code generation
    ↓
4c: Runtime library (alloc, panic, tensor ops)
    ↓
4d: MLIR backend — GPU code generation
    ↓
4e: ABI + FFI
    ↓
4f: CLI build command + testing
```

### Step 3: Phase 5 — Standard Library (Make things useful)

**Why third:** Users need collections, I/O, and tensor ops to write real programs.

```
5a: Core traits + prelude (operator overloading, iterators)
    ↓
5b: Collections (Vec, HashMap, HashSet)
    ↓
5c: Tensor operations (the heart of Axon)
    ↓
5d: Math + I/O + strings
    ↓
5e: Concurrency + data loading
    ↓
5f: Testing + benchmarks
```

### Step 4: Phase 6 — AI Framework (Axon's killer feature)

**Why fourth:** The AI framework is what differentiates Axon from Rust/C++.

```
6a: Autograd engine (computation graph + backward pass)
    ↓
6b: Core layers (Linear, Conv2d, BatchNorm)
    ↓
6c: Advanced layers (LSTM, Transformer, Attention)
    ↓
6d: Optimizers (SGD, Adam, AdamW) + loss functions
    ↓
6e: Training loop + checkpointing + mixed precision
    ↓
6f: ONNX export + metrics
    ↓
6g: Testing (train MLP, CNN, verify against PyTorch)
```

### Step 5: Phase 7 — Tooling (Make things productive)

**Why fifth:** Developer experience multiplies productivity.

```
7a: Formatter (simplest tool, immediate value)
    ↓
7b: LSP server (biggest impact on daily dev)
    ↓
7c: VS Code extension (connect LSP to most popular editor)
    ↓
7d: Package manager (enable ecosystem growth)
    ↓
7e: REPL (interactive exploration)
    ↓
7f: Debugger (last resort debugging)
    ↓
7g: Documentation generator
```

### Step 6: Phase 8 — Hardening & Release (Make things solid)

**Why last:** Polish and release after everything works.

```
8a: Benchmarks (quantify performance)
    ↓
8b: Fuzz testing (find edge cases)
    ↓
8c: Security audit (verify safety claims)
    ↓
8d: Documentation (user guide + tutorials)
    ↓
8e: CI/CD + release binaries
    ↓
8f: Specification compliance verification
    ↓
    🎉 PUBLIC RELEASE: Axon v0.1.0
```

---

## Milestones

| Milestone          | Gate           | Definition of Done                                     |
| ------------------ | -------------- | ------------------------------------------------------ |
| **M1: Type-Safe**  | End of Phase 3 | All example programs type-check; borrow errors caught  |
| **M2: Compiles**   | End of Phase 4 | `axonc build hello.axon` produces working binary       |
| **M3: Useful**     | End of Phase 5 | Can write real programs with collections, I/O, tensors |
| **M4: ML-Ready**   | End of Phase 6 | Can train an MLP on MNIST end-to-end                   |
| **M5: Productive** | End of Phase 7 | LSP works in VS Code with completions + diagnostics    |
| **M6: Release**    | End of Phase 8 | All FRs verified, docs complete, binaries published    |

---

## Risk Register

| Risk                           | Impact                         | Mitigation                                                         |
| ------------------------------ | ------------------------------ | ------------------------------------------------------------------ |
| LLVM API instability           | Codegen breaks on LLVM updates | Pin LLVM version, use inkwell's stable API                         |
| MLIR complexity                | GPU backend delays             | Start with CPU-only, add GPU incrementally                         |
| Type inference edge cases      | Confusing error messages       | Extensive fuzz testing of type checker                             |
| Borrow checker false positives | User frustration               | Conservative but correct; escape hatch via `unsafe`                |
| Performance gap vs PyTorch     | Credibility                    | Focus on compile-time safety as differentiator, optimize hot paths |
| Ecosystem bootstrapping        | No packages → no users         | Ship comprehensive stdlib; support FFI to existing C/Rust libs     |

---

## File Structure (Final)

```
axon/
├── Cargo.toml
├── Cargo.lock
├── src/
│   ├── lib.rs              # Library root
│   ├── main.rs             # CLI entry point
│   ├── span.rs             # Source locations           (Phase 2) ✅
│   ├── token.rs            # Token types                (Phase 2) ✅
│   ├── error.rs            # Error types                (Phase 2) ✅
│   ├── ast.rs              # AST nodes                  (Phase 2) ✅
│   ├── lexer.rs            # Lexer                      (Phase 2) ✅
│   ├── parser.rs           # Parser                     (Phase 2) ✅
│   ├── types.rs            # Type representation        (Phase 3)
│   ├── symbol.rs           # Symbol table               (Phase 3)
│   ├── typeck.rs           # Type checker               (Phase 3)
│   ├── shapes.rs           # Shape checker              (Phase 3)
│   ├── borrow.rs           # Borrow checker             (Phase 3)
│   ├── tast.rs             # Typed AST                  (Phase 3)
│   ├── mir.rs              # Mid-level IR               (Phase 4)
│   ├── codegen/
│   │   ├── mod.rs          # Codegen orchestration      (Phase 4)
│   │   ├── llvm.rs         # LLVM backend               (Phase 4)
│   │   ├── mlir.rs         # MLIR backend               (Phase 4)
│   │   ├── runtime.rs      # Runtime support            (Phase 4)
│   │   └── abi.rs          # ABI + FFI                  (Phase 4)
│   ├── fmt.rs              # Formatter                  (Phase 7)
│   ├── lint.rs             # Linter                     (Phase 7)
│   ├── doc.rs              # Doc generator              (Phase 7)
│   └── repl.rs             # REPL                       (Phase 7)
├── stdlib/
│   ├── prelude.axon        # Auto-imported types/traits (Phase 5)
│   ├── ops.axon            # Operator traits            (Phase 5)
│   ├── convert.axon        # Conversion traits          (Phase 5)
│   ├── iter.axon           # Iterator trait             (Phase 5)
│   ├── option.axon         # Option<T>                  (Phase 5)
│   ├── result.axon         # Result<T, E>               (Phase 5)
│   ├── string.axon         # String utilities           (Phase 5)
│   ├── math.axon           # Math functions             (Phase 5)
│   ├── io.axon             # File I/O                   (Phase 5)
│   ├── fmt.axon            # String formatting          (Phase 5)
│   ├── mem.axon            # Memory utilities           (Phase 5)
│   ├── time.axon           # Time/duration              (Phase 5)
│   ├── random.axon         # RNG                        (Phase 5)
│   ├── device.axon         # Device abstraction         (Phase 5)
│   ├── collections/
│   │   ├── vec.axon        # Vec<T>                     (Phase 5)
│   │   ├── hashmap.axon    # HashMap<K, V>              (Phase 5)
│   │   └── hashset.axon    # HashSet<T>                 (Phase 5)
│   ├── tensor/
│   │   ├── create.axon     # Tensor creation            (Phase 5)
│   │   ├── shape.axon      # Shape operations           (Phase 5)
│   │   ├── reduce.axon     # Reduction ops              (Phase 5)
│   │   ├── math.axon       # Element-wise math          (Phase 5)
│   │   ├── linalg.axon     # Linear algebra             (Phase 5)
│   │   └── device.axon     # Device transfer            (Phase 5)
│   ├── sync/
│   │   ├── mutex.axon      # Mutex                      (Phase 5)
│   │   ├── rwlock.axon     # RwLock                     (Phase 5)
│   │   ├── channel.axon    # Channel                    (Phase 5)
│   │   └── arc.axon        # Arc                        (Phase 5)
│   ├── thread.axon         # Thread spawning            (Phase 5)
│   ├── data/
│   │   ├── csv.axon        # CSV loading                (Phase 5)
│   │   ├── json.axon       # JSON loading               (Phase 5)
│   │   ├── loader.axon     # DataLoader                 (Phase 5)
│   │   └── transforms.axon # Data transforms            (Phase 6)
│   ├── autograd/
│   │   ├── graph.axon      # Computation graph          (Phase 6)
│   │   ├── grad_tensor.axon# GradTensor                 (Phase 6)
│   │   ├── backward.axon   # Backward pass              (Phase 6)
│   │   ├── ops.axon        # Gradient rules             (Phase 6)
│   │   └── context.axon    # no_grad, checkpointing     (Phase 6)
│   ├── nn/
│   │   ├── module.axon     # Module trait               (Phase 6)
│   │   ├── layers.axon     # Linear, Conv2d, BatchNorm  (Phase 6)
│   │   ├── recurrent.axon  # LSTM, GRU                  (Phase 6)
│   │   ├── transformer.axon# Attention, Transformer     (Phase 6)
│   │   ├── embedding.axon  # Embedding                  (Phase 6)
│   │   ├── sequential.axon # Sequential container       (Phase 6)
│   │   ├── activation.axon # ReLU, GELU, Softmax        (Phase 6)
│   │   └── init.axon       # Weight initialization      (Phase 6)
│   ├── optim/
│   │   ├── mod.axon        # Optimizer trait             (Phase 6)
│   │   ├── sgd.axon        # SGD                        (Phase 6)
│   │   ├── adam.axon       # Adam, AdamW                (Phase 6)
│   │   └── scheduler.axon  # LR schedulers              (Phase 6)
│   ├── loss/
│   │   └── mod.axon        # All loss functions         (Phase 6)
│   ├── train/
│   │   ├── trainer.axon    # Training loop              (Phase 6)
│   │   ├── checkpoint.axon # Save/load                  (Phase 6)
│   │   └── amp.axon        # Mixed precision            (Phase 6)
│   ├── metrics/
│   │   └── mod.axon        # Accuracy, F1, etc.         (Phase 6)
│   └── export/
│       ├── onnx.axon       # ONNX export                (Phase 6)
│       └── native.axon     # Native serialization       (Phase 6)
├── axon-lsp/               # Language server             (Phase 7)
├── axon-pkg/               # Package manager             (Phase 7)
├── axon-dbg/               # Debugger                    (Phase 7)
├── editors/
│   └── vscode/             # VS Code extension           (Phase 7)
├── tests/
│   ├── examples/*.axon     # Example programs            (Phase 2) ✅
│   ├── integration_tests.rs # Parser integration         (Phase 2) ✅
│   ├── type_tests.rs       # Type checker tests          (Phase 3)
│   ├── codegen_tests.rs    # Codegen tests               (Phase 4)
│   ├── stdlib_tests.rs     # Stdlib tests                (Phase 5)
│   ├── tensor_tests.rs     # Tensor op tests             (Phase 5)
│   ├── autograd_tests.rs   # Autograd tests              (Phase 6)
│   ├── train_tests.rs      # Training tests              (Phase 6)
│   └── export_tests.rs     # Export tests                (Phase 6)
├── benches/                # Benchmarks                   (Phase 8)
├── fuzz/                   # Fuzz testing                  (Phase 8)
├── docs/
│   ├── index.html          # Landing page                 (Phase 2) ✅
│   ├── specs/              # Phase specifications
│   │   ├── phase3-type-checker.md
│   │   ├── phase4-codegen.md
│   │   ├── phase5-stdlib.md
│   │   ├── phase6-ai-framework.md
│   │   ├── phase7-tooling.md
│   │   ├── phase8-hardening.md
│   │   └── master-overview.md    ← This file
│   ├── guide/              # User guide                   (Phase 8)
│   ├── tutorial/           # ML tutorials                 (Phase 8)
│   ├── reference/          # Language + API reference     (Phase 8)
│   └── migration/          # Migration guides             (Phase 8)
└── tasks.md                # Task tracking                (Phase 2) ✅
```

---

## Success Criteria (v0.1.0 Release)

1. **A complete Axon program** can be written, type-checked, compiled, and executed:

   ```axon
   use std::nn::{Linear, ReLU, Sequential};
   use std::optim::Adam;
   use std::loss::CrossEntropyLoss;
   use std::data::DataLoader;

   fn main() {
       let model = Sequential::new(vec![
           Linear::new(784, 256),
           ReLU::new(),
           Linear::new(256, 10),
       ]);
       let optimizer = Adam::new(model.parameters(), lr: 0.001);
       let loss_fn = CrossEntropyLoss::new();
       // ... training loop
   }
   ```

2. **Shape errors caught at compile time:**

   ```
   error[E3002]: matmul inner dimensions don't match
     --> model.axon:5:12
      |
   5  |     let y = x @ w;    // x: [batch, 784], w: [256, 10]
      |             ^^^^^
      |  left inner dim: 784, right outer dim: 256
      |  help: did you mean w: Tensor<Float32, [784, 10]>?
   ```

3. **Borrow errors caught at compile time:**

   ```
   error[E4001]: use of moved value `tensor`
     --> train.axon:12:20
      |
   10 |     let gpu_tensor = tensor.to_gpu();  // tensor moved here
      |                      ------
   12 |     println(tensor.shape());            // error: tensor was moved
      |             ^^^^^^
      |  help: use tensor.clone().to_gpu() to keep the original
   ```

4. **Performance** within 2× of PyTorch for standard ML workloads.

5. **Developer experience**: full LSP support in VS Code with < 200ms latency.

6. **All 72 functional requirements** and **28 non-functional requirements** verified and passing.
