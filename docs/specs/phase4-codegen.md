# Phase 4: LLVM/MLIR Code Generation — Implementation Specification

## 1. Overview

Phase 4 lowers the Typed AST (TAST) from Phase 3 into executable machine code via **LLVM IR** for CPU targets and **MLIR dialects** for GPU/accelerator targets. The output is native binaries, shared libraries, or object files.

### Deliverables

| Artifact                 | Path            | Purpose                                                  |
| ------------------------ | --------------- | -------------------------------------------------------- |
| `src/mir.rs`             | Mid-level IR    | Axon MIR: simplified, lowered representation before LLVM |
| `src/codegen/mod.rs`     | Code generator  | Main codegen orchestration                               |
| `src/codegen/llvm.rs`    | LLVM backend    | Lower MIR → LLVM IR for CPU                              |
| `src/codegen/mlir.rs`    | MLIR backend    | Lower MIR → MLIR for GPU/TPU                             |
| `src/codegen/runtime.rs` | Runtime support | Tensor allocation, device dispatch, refcounting          |
| `src/codegen/abi.rs`     | ABI / Linking   | Calling conventions, symbol mangling, FFI                |

### Dependencies

- Phase 3 outputs: `tast.rs`, `types.rs`, `symbol.rs`
- External: `llvm-sys` or `inkwell` crate for LLVM bindings
- External: `melior` or custom MLIR bindings for GPU codegen

---

## 2. Axon MIR (`src/mir.rs`)

### 2.1 Purpose

An intermediate representation between the TAST and LLVM IR. The MIR:

- Desugars high-level constructs (match → switch, for → loop+iterator, method calls → static dispatch).
- Makes control flow explicit (basic blocks + terminators).
- Makes drops/destructors explicit.
- Makes all type coercions explicit.

### 2.2 MIR Structure

```
MirProgram
├── functions: Vec<MirFunction>
├── statics: Vec<MirStatic>
└── type_layouts: TypeLayoutTable

MirFunction
├── name: String (mangled)
├── params: Vec<MirLocal>
├── return_ty: TypeId
├── locals: Vec<MirLocal>
├── basic_blocks: Vec<BasicBlock>
└── attributes: Vec<Attribute>  // @cpu, @gpu

BasicBlock
├── id: BlockId
├── stmts: Vec<MirStmt>
└── terminator: Terminator

MirStmt
├── Assign { place, rvalue }
├── Drop { place }
├── StorageLive { local }
├── StorageDead { local }
└── Nop

Terminator
├── Goto { target }
├── SwitchInt { value, targets, otherwise }
├── Return
├── Call { func, args, destination, cleanup }
├── Assert { cond, msg, target, cleanup }
└── Unreachable

Rvalue
├── Use(Operand)
├── BinaryOp { op, left, right }
├── UnaryOp { op, operand }
├── Ref { mutable, place }
├── AddressOf { place }
├── Aggregate { kind, fields }    // struct/tuple/enum construction
├── Cast { operand, target_ty }
├── Len { place }                 // array/vec length
├── TensorOp { kind, operands }   // matmul, elementwise, reshape, etc.
└── Discriminant { place }        // enum discriminant
```

### 2.3 MIR Lowering from TAST

| TAST Construct        | MIR Lowering                                                   |
| --------------------- | -------------------------------------------------------------- |
| `if/else`             | Conditional `SwitchInt` with two targets                       |
| `match`               | Decision tree → chain of `SwitchInt` on discriminants          |
| `for x in iter`       | `loop { let x = iter.next(); if None break; body }`            |
| `while cond { body }` | `loop { if !cond break; body }`                                |
| `x.method(args)`      | `ClassName::method(&x, args)` (static dispatch) or vtable call |
| `a @ b`               | `TensorOp::MatMul(a, b)`                                       |
| `expr?`               | `match expr { Ok(v) => v, Err(e) => return Err(e) }`           |
| Drop insertion        | Insert `Drop` stmts at scope exits per borrow checker info     |

---

## 3. LLVM Backend (`src/codegen/llvm.rs`)

### 3.1 Type Mapping

| Axon Type                 | LLVM Type                                                          |
| ------------------------- | ------------------------------------------------------------------ |
| `Int8`                    | `i8`                                                               |
| `Int16`                   | `i16`                                                              |
| `Int32`                   | `i32`                                                              |
| `Int64`                   | `i64`                                                              |
| `UInt8..UInt64`           | `i8..i64` (unsigned semantics via instructions)                    |
| `Float16`                 | `half`                                                             |
| `Float32`                 | `float`                                                            |
| `Float64`                 | `double`                                                           |
| `Bool`                    | `i1`                                                               |
| `Char`                    | `i32` (Unicode scalar)                                             |
| `String`                  | `{ i8*, i64, i64 }` (ptr, len, cap)                                |
| `&T`                      | `T*`                                                               |
| `&mut T`                  | `T*`                                                               |
| `Tensor<D, [S]>`          | `{ D*, i64*, i64, i8 }` (data_ptr, shape_ptr, ndim, device)        |
| `Vec<T>`                  | `{ T*, i64, i64 }` (ptr, len, cap)                                 |
| `(A, B, C)`               | `{ A, B, C }`                                                      |
| `struct S { a: A, b: B }` | `{ A, B }` with named field layout                                 |
| `enum E`                  | Tagged union: `{ i8_tag, union_of_variants }`                      |
| `Option<T>`               | Same as enum (tag + payload), with niche optimization for pointers |
| `fn(A) -> B`              | Function pointer `B (A)*`                                          |

### 3.2 Function Codegen

For each `MirFunction`:

1. Create LLVM function with mangled name and correct calling convention.
2. Allocate `alloca` for each `MirLocal`.
3. For each `BasicBlock`:
   a. Create LLVM basic block.
   b. Emit LLVM instructions for each `MirStmt`.
   c. Emit terminator (branch, switch, ret, call).
4. Run LLVM optimization passes (configurable: `-O0` to `-O3`).

### 3.3 Optimization Passes

| Level | Passes                                                     |
| ----- | ---------------------------------------------------------- |
| `-O0` | None (debug builds)                                        |
| `-O1` | Mem2Reg, SROA, SimplifyCFG, EarlyCSE                       |
| `-O2` | + InstCombine, GVN, LoopVectorize, SLPVectorize            |
| `-O3` | + Aggressive inlining, LoopUnswitch, MergedLoadStoreMotion |

### 3.4 Debug Info

Emit DWARF debug info (via LLVM `DIBuilder`):

- Map MIR locals back to source-level names + spans.
- Emit line tables for step debugging.
- Emit type descriptors for struct/enum inspection.

---

## 4. MLIR Backend (`src/codegen/mlir.rs`)

### 4.1 Purpose

Functions annotated `@gpu` are lowered to MLIR dialects for GPU compilation.

### 4.2 Dialect Stack

```
Axon TAST
  ↓ Lower @gpu functions
MLIR Linalg dialect (tensor operations)
  ↓ Bufferize
MLIR MemRef dialect (memory references)
  ↓ Lower to target
MLIR GPU dialect → NVVM (CUDA) / ROCDL (ROCm) / SPIR-V (Vulkan/OpenCL)
  ↓ Translate
PTX / AMDGPU ISA / SPIR-V binary
```

### 4.3 Tensor Operation Mapping

| Axon Operation      | MLIR Op                           |
| ------------------- | --------------------------------- |
| `a + b` (tensor)    | `linalg.add`                      |
| `a @ b`             | `linalg.matmul`                   |
| `reshape(a, shape)` | `tensor.reshape`                  |
| `transpose(a)`      | `linalg.transpose`                |
| `a[i..j]` (slice)   | `tensor.extract_slice`            |
| Elementwise fn      | `linalg.generic` with custom body |

### 4.4 Device Dispatch

```
@device(expr) fn compute(x: Tensor<Float32, [N, M]>) -> Tensor<Float32, [N, M]> {
    // At compile time:
    // 1. Generate both CPU (LLVM) and GPU (MLIR→PTX) versions.
    // 2. Emit runtime dispatch: if expr == "gpu" → call GPU version, else CPU.
}
```

### 4.5 Memory Management on GPU

- Tensor data is allocated via `cudaMalloc` / `hipMalloc` or equivalent.
- Host↔Device transfers are explicit moves in the borrow checker.
- Kernel launches are emitted as `gpu.launch_func` ops.

---

## 5. Runtime Support (`src/codegen/runtime.rs`)

### 5.1 Runtime Library Functions

The compiler emits calls to a small Axon runtime library:

| Function                                              | Purpose                   |
| ----------------------------------------------------- | ------------------------- |
| `axon_alloc(size, align) → *u8`                       | Heap allocation           |
| `axon_dealloc(ptr, size, align)`                      | Heap deallocation         |
| `axon_tensor_alloc(dtype, shape, device) → TensorPtr` | Tensor allocation         |
| `axon_tensor_free(TensorPtr)`                         | Tensor deallocation       |
| `axon_tensor_shape_check(a, b, op) → bool`            | Runtime shape assertion   |
| `axon_device_transfer(src, dst_device) → TensorPtr`   | CPU↔GPU transfer          |
| `axon_panic(msg, file, line)`                         | Panic handler             |
| `axon_refcount_inc(ptr)`                              | For `Arc<T>`              |
| `axon_refcount_dec(ptr) → bool`                       | Returns true if count → 0 |

### 5.2 String Runtime

Strings use a SSO (Small String Optimization) layout:

- Strings ≤ 22 bytes are stored inline.
- Longer strings are heap-allocated with reference counting.

### 5.3 Vec/HashMap Runtime

Standard resizable collections with:

- Geometric growth (factor 2).
- Custom allocator support (future).

---

## 6. ABI & Linking (`src/codegen/abi.rs`)

### 6.1 Symbol Mangling

```
_AX{crate_hash}N{namespace_len}{namespace}F{function_len}{function}G{generic_args}
```

Example: `std::math::sin<Float32>` → `_AX7d3f4aN4mathF3sinGF32`

### 6.2 Calling Convention

- Default: C calling convention (for FFI compatibility).
- Internal Axon calls: Rust-like calling convention (pass aggregates by pointer when > 2 registers).

### 6.3 FFI

```axon
unsafe fn printf(fmt: &String, ...) -> Int32;  // C FFI declaration
```

- `unsafe fn` with no body = FFI import.
- Axon generates the appropriate extern declaration in LLVM IR.

---

## 7. CLI Integration

```
axonc build <file.axon>                   # Compile to native binary
axonc build --emit-llvm <file.axon>       # Emit LLVM IR (.ll)
axonc build --emit-mir <file.axon>        # Emit Axon MIR (debug)
axonc build --emit-obj <file.axon>        # Emit object file (.o)
axonc build -O0|-O1|-O2|-O3 <file.axon>  # Optimization level
axonc build --target <triple> <file.axon> # Cross-compilation
axonc build --gpu=cuda|rocm|vulkan        # GPU target
```

---

## 8. Task Breakdown

### Phase 4a: MIR

- [ ] T109 Define MIR data structures (BasicBlock, Terminator, Rvalue) — `src/mir.rs`
- [ ] T110 Lower TAST → MIR: expressions and statements — `src/mir.rs`
- [ ] T111 Lower TAST → MIR: control flow (if, match, loops) — `src/mir.rs`
- [ ] T112 Lower TAST → MIR: drop insertion — `src/mir.rs`
- [ ] T113 Lower TAST → MIR: tensor operations — `src/mir.rs`

### Phase 4b: LLVM Backend

- [ ] T114 Set up inkwell/llvm-sys integration — `src/codegen/llvm.rs`
- [ ] T115 Implement type mapping (Axon → LLVM types) — `src/codegen/llvm.rs`
- [ ] T116 Implement function codegen (params, locals, return) — `src/codegen/llvm.rs`
- [ ] T117 Implement expression codegen (arithmetic, calls, field access) — `src/codegen/llvm.rs`
- [ ] T118 Implement control flow codegen (branches, switches) — `src/codegen/llvm.rs`
- [ ] T119 Implement optimization pipeline (-O0 through -O3) — `src/codegen/llvm.rs`
- [ ] T120 Implement debug info emission (DWARF) — `src/codegen/llvm.rs`
- [ ] T121 Implement native binary output (linking) — `src/codegen/llvm.rs`

### Phase 4c: MLIR Backend

- [ ] T122 Set up MLIR bindings (melior or custom) — `src/codegen/mlir.rs`
- [ ] T123 Lower @gpu functions to Linalg dialect — `src/codegen/mlir.rs`
- [ ] T124 Bufferization pass (tensor → memref) — `src/codegen/mlir.rs`
- [ ] T125 Lower to GPU dialect / NVVM / SPIR-V — `src/codegen/mlir.rs`
- [ ] T126 Emit PTX/AMDGPU/SPIR-V binary — `src/codegen/mlir.rs`

### Phase 4d: Runtime & ABI

- [ ] T127 Implement runtime library (alloc, tensor ops, panic) — `src/codegen/runtime.rs`
- [ ] T128 Implement symbol mangling scheme — `src/codegen/abi.rs`
- [ ] T129 Implement FFI import/export — `src/codegen/abi.rs`
- [ ] T130 Implement device dispatch for @device — `src/codegen/runtime.rs`

### Phase 4e: CLI & Testing

- [ ] T131 CLI `axonc build` command with all flags — `src/main.rs`
- [ ] T132 Test: compile and run "hello world" — `tests/codegen_tests.rs`
- [ ] T133 Test: arithmetic operations produce correct results — `tests/codegen_tests.rs`
- [ ] T134 Test: struct/enum layout and access — `tests/codegen_tests.rs`
- [ ] T135 Test: tensor matmul on CPU — `tests/codegen_tests.rs`
- [ ] T136 Test: GPU kernel compilation (if CUDA available) — `tests/codegen_tests.rs`
- [ ] T137 Test: optimization levels produce valid output — `tests/codegen_tests.rs`

---

## 9. Acceptance Criteria

- [ ] `axonc build hello.axon` produces a working native binary
- [ ] All Axon primitive types compile to correct LLVM types
- [ ] Tensor operations produce correct results on CPU
- [ ] `@gpu` annotated functions compile to PTX (with CUDA toolkit)
- [ ] `-O2` produces optimized output (inlining, vectorization visible in IR)
- [ ] Debug builds produce DWARF info usable by GDB/LLDB
- [ ] FFI calls to C functions work correctly
- [ ] Runtime shape checks fire for dynamic tensor dimensions
- [ ] Cross-compilation for at least x86_64 and aarch64 works
