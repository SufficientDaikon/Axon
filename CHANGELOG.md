# Changelog

All notable changes to the Axon programming language are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/),
and this project adheres to [Semantic Versioning](https://semver.org/).

---

## [0.1.0-alpha.1] - 2026-03-09

### Added

#### Compiler Frontend

- **Lexer**: Full tokenization of Axon source — keywords, operators, literals
  (decimal, hex, binary, octal, float, scientific notation), string/char literals
  with escape sequences, single-line and nested block comments, attributes
  (`@cpu`, `@gpu`, `@device`), `&mut` compound token
- **Parser**: Complete parser with error recovery — functions (generics, params,
  return types), structs, enums (unit, tuple, struct variants), impl blocks,
  traits (supertraits, methods), type aliases, modules, use declarations,
  let/return/while/for/if-else/match statements, binary operators with
  precedence, `@` matmul operator, unary operators, postfix `?`/`.`/`()`/`[]`,
  type casts (`as`), path expressions, struct literals, tensor types with `?`
  and named dims, patterns, assignment operators, range expressions, visibility
  (`pub`), attributes, `unsafe fn`, multiple error reporting
- **AST**: Complete AST node types for all language constructs

#### Type System

- **Type Checker**: Constraint-based Hindley-Milner type inference with
  unification, expression/statement/item type checking, generic instantiation
  and bound checking, trait resolution (inherent + trait impls), pattern type
  checking, type coercion (`&mut` → `&`, auto-borrow)
- **Shape Checker**: Compile-time tensor shape verification — matmul inner dim
  matching, elementwise broadcast rules, reshape/transpose shape rules, runtime
  shape checks for dynamic dimensions
- **Borrow Checker**: Control flow graph construction, variable liveness
  analysis, borrow and move tracking, exclusivity enforcement (no `&mut` + `&`
  overlap), move semantics (use-after-move detection), lifetime inference,
  device-aware borrow rules (`@cpu`/`@gpu`)
- **Symbol Table**: Scoped symbol resolution, name resolution for top-level
  items, use/import path resolution, identifier and type resolution

#### Code Generation

- **MIR**: Mid-level intermediate representation with basic blocks,
  terminators, rvalues, expression/statement/control-flow lowering, drop
  insertion, tensor operation lowering
- **LLVM Backend**: LLVM IR text emitter, type mapping, function/expression/
  control-flow codegen, optimization pipeline (`-O0` through `-O3`), debug info
  (DWARF), native binary output via clang integration
- **MLIR Backend**: GPU target detection, architecture for CUDA/ROCm/Vulkan
  kernel compilation (stub)
- **Runtime & ABI**: Runtime library declarations, symbol mangling scheme, FFI
  calling conventions, C runtime source generation

#### Standard Library (200+ built-in functions, 30+ traits)

- **Core Traits**: Operator traits (Add, Sub, Mul, Div, MatMul, Index),
  conversion traits (From, Into, TryFrom, TryInto), Display, Debug, Clone,
  Copy, Default, Drop, Iterator
- **Collections**: `Vec<T>`, `HashMap<K, V>`, `HashSet<T>`, `Option<T>`,
  `Result<T, E>`, `String`
- **Tensor Operations**: Creation (zeros, ones, randn, rand, from_vec, arange,
  eye), shape ops (reshape, transpose, squeeze, unsqueeze, cat, stack),
  reductions (sum, mean, max, min, argmax, argmin, prod, norm), element-wise
  math (sin, cos, exp, log, sqrt, abs, clamp), linear algebra (matmul, dot,
  det, inv, qr, svd, eigenvalues, trace), device transfer (to_gpu, to_cpu,
  to_device)
- **Math**: Trigonometric, logarithmic, and statistical functions
- **I/O**: Read/Write traits, File, println/print/eprintln, string formatting
- **Concurrency**: Mutex, RwLock, Arc, Channel (bounded + unbounded),
  thread::spawn, JoinHandle
- **Data**: CSV/JSON loading, DataLoader for ML pipelines
- **Device**: Device abstraction, GPU query

#### AI Framework

- **Autograd**: Computation graph, GradTensor with gradient tracking, reverse-mode
  automatic differentiation, gradient rules for all operations, no_grad context,
  gradient checkpointing
- **Neural Network Layers**: Module trait, Linear, Conv2d, BatchNorm, LayerNorm,
  Dropout, MaxPool2d, AvgPool2d, LSTM, GRU, MultiHeadAttention,
  TransformerEncoder, Embedding, Sequential, activation modules (ReLU, GELU,
  SiLU, Softmax), weight initialization
- **Optimizers**: Optimizer trait, SGD, Adam, AdamW, learning rate schedulers
  (StepLR, CosineAnnealing, ReduceOnPlateau)
- **Loss Functions**: CrossEntropy, MSE, BCE, L1, Huber, KLDivergence,
  CosineEmbedding
- **Training**: Trainer with callbacks, checkpointing (save/load), mixed
  precision training
- **Export**: ONNX export, native model serialization
- **Metrics**: Accuracy, precision, recall, F1, confusion matrix, ROC-AUC
- **Transforms**: Image transforms (resize, crop, normalize), text transforms
  (tokenize, pad, truncate)

#### Tooling

- **Language Server (LSP)**: Document sync, real-time diagnostics,
  go-to-definition, hover (type info), completion engine, find-references,
  rename, signature help, inlay hints, semantic tokens
- **Package Manager**: Project scaffolding (new/init), Axon.toml manifest,
  dependency resolution, lock file generation, build orchestration,
  add/remove/update commands, registry client
- **REPL**: Interactive shell with history, expression evaluation, persistent
  state, commands (`:type`, `:ast`, `:load`, `:save`), tab completion
- **Formatter**: Idempotent code formatting (`axonc fmt`)
- **Linter**: Configurable lint rules with warnings (W5001–W5008)
- **Documentation Generator**: Doc comment parser, HTML output (`axonc doc`)
- **VS Code Extension**: TextMate grammar, LSP client, snippets, keybindings,
  launch configurations
- **CLI**: `axonc lex`, `parse`, `check`, `build`, `fmt`, `lint`, `repl`,
  `doc`, `lsp`, `pkg` commands with `--error-format=json` support

#### Testing & Quality

- 863+ tests across all compiler phases
- Fuzz testing infrastructure
- Security audit documentation
- Benchmark suite

### Performance

- Lexer handles 1000+ function files in <100ms
- Type checker handles complex programs with full inference
- Parser with error recovery produces multiple diagnostics per file
