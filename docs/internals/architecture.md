# Axon Compiler Architecture

## Pipeline

```
Source → Lexer → Parser → AST → Name Resolution → Type Checking
      → Shape Checking → Borrow Checking → TAST → MIR → MIR Passes → LLVM IR → Native Binary
```

## Overview

The Axon compiler (`axonc`) is structured as a multi-phase pipeline. Each phase
transforms the program representation and may produce diagnostic errors. The
pipeline is designed to continue after errors where possible, providing multiple
diagnostics in a single compilation pass.

## Modules

### Core Pipeline

| Module              | File            | Description                                                                                                                                                                                                                 |
| ------------------- | --------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **Lexer**           | `src/lexer.rs`  | Tokenization of Axon source text. Handles keywords, types, operators, delimiters, literals (int, float, string, char), comments (`//` and `/* */`), attributes (`@cpu`, `@gpu`, `@device`), and source location tracking.   |
| **Parser**          | `src/parser.rs` | Recursive descent parser. Produces an AST from a token stream. Handles operator precedence with Pratt parsing, provides clear error messages with source locations, and implements error recovery for multiple diagnostics. |
| **AST**             | `src/ast.rs`    | Abstract Syntax Tree types. All nodes carry `Span` for source location. Serializable via serde for tooling integration.                                                                                                     |
| **Name Resolution** | `src/symbol.rs` | Symbol table with lexical scoping. Resolves names to definitions, detects undefined variables, and tracks variable mutability. Part of the type checking phase.                                                             |
| **Type Checker**    | `src/typeck.rs` | Hindley-Milner type inference with constraint-based unification. Registers stdlib types, resolves names, infers expression types, and checks type compatibility.                                                            |
| **Shape Checker**   | `src/shapes.rs` | Tensor dimension verification. Ensures tensor operations have compatible shapes at compile time. Axon's key differentiator for ML/AI safety.                                                                                |
| **Borrow Checker**  | `src/borrow.rs` | Ownership, move, and borrow analysis. Tracks value lifetimes, prevents use-after-move, and enforces mutable borrow exclusivity.                                                                                             |
| **TAST**            | `src/tast.rs`   | Typed Abstract Syntax Tree. Annotates each AST node with its resolved type. Serves as the bridge between type checking and code generation.                                                                                 |
| **MIR**             | `src/mir/`      | Mid-level Intermediate Representation. Flattened, SSA-like form suitable for optimization passes and lowering to LLVM IR.                                                                                                   |
| **MIR Passes**      | `src/mir/transform/` | MIR optimization passes: dead code elimination and constant folding. Managed by a `PassManager` that runs passes based on optimization level.                                                                         |
| **Name Interner**   | `src/interner.rs` | Global string interning for O(1) name comparisons. Deduplicates identifier strings via `NameInterner` and lightweight `Name` handles.                                                                                    |
| **Diagnostics**     | `src/error.rs`  | Accumulative diagnostic system with categories, severity configuration (--deny/--allow/--warn), error limits, and grouped display.                                                                                          |
| **Codegen**         | `src/codegen/`  | LLVM IR generation and native compilation.                                                                                                                                                                                  |

### Code Generation Submodules

| Module      | File                     | Description                                                                          |
| ----------- | ------------------------ | ------------------------------------------------------------------------------------ |
| **LLVM IR** | `src/codegen/llvm.rs`    | Generates textual LLVM IR (.ll files) from MIR. Compiles to native code via `clang`. |
| **ABI**     | `src/codegen/abi.rs`     | Application Binary Interface definitions for calling conventions.                    |
| **MLIR**    | `src/codegen/mlir.rs`    | MLIR integration for ML-specific optimizations (future).                             |
| **Runtime** | `src/codegen/runtime.rs` | Runtime support declarations (memory allocation, printing, etc.).                    |

### Standard Library

| Module     | File          | Description                                                                                                                                                        |
| ---------- | ------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| **Stdlib** | `src/stdlib/` | Built-in type and function registration. Registers primitive types (Int32, Float64, Bool, String), collection types, and AI framework types into the type checker. |

The stdlib includes AI/ML framework types:

- `src/stdlib/nn.rs` — Neural network layers (Linear, Conv2d, etc.)
- `src/stdlib/optim.rs` — Optimizers (SGD, Adam, etc.)
- `src/stdlib/data.rs` — Data loading types (Dataset, DataLoader)
- `src/stdlib/mem.rs` — Memory management primitives

### Tooling

| Module              | File          | Description                                                                                    |
| ------------------- | ------------- | ---------------------------------------------------------------------------------------------- |
| **Formatter**       | `src/fmt.rs`  | Code formatter. Parses source to AST and re-emits with consistent style.                       |
| **Linter**          | `src/lint.rs` | Static analysis linter. Checks for unused variables, naming conventions, complexity, and more. |
| **REPL**            | `src/repl.rs` | Interactive Read-Eval-Print Loop for Axon expressions.                                         |
| **Doc Generator**   | `src/doc.rs`  | Documentation generation from doc comments.                                                    |
| **LSP Server**      | `src/lsp/`    | Language Server Protocol implementation for IDE integration.                                   |
| **Package Manager** | `src/pkg/`    | Package management (manifests, registry, dependency resolution).                               |

## Error System

Errors are categorized by compiler phase using numeric codes:

| Range     | Category               | Examples                                                       |
| --------- | ---------------------- | -------------------------------------------------------------- |
| **E0xxx** | Lexer/Parser errors    | `E0001` unexpected character, `E0002` unterminated string      |
| **E1xxx** | Name resolution errors | `E1001` undefined variable, `E1002` duplicate definition       |
| **E2xxx** | Type errors            | `E2001` type mismatch, `E2002` cannot infer type               |
| **E3xxx** | Shape errors           | `E3001` dimension mismatch, `E3002` incompatible tensor shapes |
| **E4xxx** | Borrow errors          | `E4001` use after move, `E4002` mutable borrow conflict        |
| **E5xxx** | MIR/Codegen errors     | `E5009` no main function, `E5010` codegen failure               |
| **W5xxx** | Lint warnings          | `W5001` unused variable, `W5002` naming convention             |

All errors carry:

- A source `Span` (file, line, column, offset)
- A human-readable message
- A severity level (Error, Warning, Note)
- Optional suggestions for fixes
- An optional diagnostic category for filtering (parse-error, type-error, borrow-error, etc.)

Diagnostics support severity overrides via CLI flags (`--deny`, `--allow`, `--warn`)
and an error limit (`--error-limit N`) that stops compilation after N errors.

## Data Flow

```
                    ┌──────────┐
    Source text ───►│  Lexer   │───► Vec<Token>
                    └──────────┘
                         │
                    ┌──────────┐
                    │  Parser  │───► AST (Program)
                    └──────────┘
                         │
                    ┌──────────┐
                    │  Names   │───► SymbolTable
                    └──────────┘
                         │
                    ┌──────────┐
                    │  TypeCk  │───► TypeInterner + Constraints
                    └──────────┘
                         │
                ┌────────┴────────┐
                │                 │
           ┌─────────┐    ┌──────────┐
           │ ShapeCk  │    │ BorrowCk │
           └─────────┘    └──────────┘
                │                 │
                └────────┬────────┘
                         │
                    ┌──────────┐
                    │   TAST   │───► TypedProgram
                    └──────────┘
                         │
                    ┌──────────┐
                    │   MIR    │───► MirProgram
                    └──────────┘
                         │
                    ┌──────────┐
                    │ MIR Pass │───► Optimized MirProgram
                    └──────────┘
                         │
                    ┌──────────┐
                    │  LLVM IR │───► .ll file
                    └──────────┘
                         │
                    ┌──────────┐
                    │  clang   │───► Native binary
                    └──────────┘
```

## Key Design Decisions

1. **Safe Rust only** — No `unsafe` blocks anywhere in the compiler.
2. **Arena-style type interning** — Types are identified by `TypeId` (index),
   enabling O(1) lookups and avoiding lifetime complexity.
3. **Constraint-based type inference** — Generates constraints during traversal,
   then solves via unification. Enables HM-style inference.
4. **Error recovery** — Parser continues after errors to report multiple
   diagnostics in one pass.
5. **Textual LLVM IR** — Generates `.ll` files rather than using LLVM C API,
   keeping the compiler dependency-free and simplifying builds.
6. **External `clang`** — Uses `clang` as a subprocess for final compilation,
   avoiding LLVM library linking.
7. **Stack safety** — Recursive descent functions are wrapped with `stacker::maybe_grow`
   to dynamically grow the stack for deeply nested input, preventing stack overflows.
8. **MIR optimization passes** — Pluggable pass architecture (`MirPass` trait + `PassManager`)
   enables incremental addition of optimization passes. Dead code elimination and constant
   folding are built-in at `-O1` and above.
