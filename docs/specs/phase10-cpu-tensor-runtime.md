# Phase 10: CPU Tensor Runtime

## Summary

Phase 10 replaces every tensor stub with a real, numerically-correct CPU
implementation.  After this phase a user can write:

```axon
let a = Tensor::rand([128, 784]);
let w = Tensor::rand([784, 256]);
let b = a @ w;                       // 128×256
let c = b.relu();
let loss = c.sum();
println(loss);
```

…compile it, run it, and get a correct floating-point result.

## Prerequisites

| Requirement | Provided by |
|-------------|-------------|
| E2E scalar compilation works | Phase 9 |
| `axon_runtime.c` compiles and links | Phase 9 |
| Clang in PATH | Phase 9 |
| Type checker understands Tensor types | Phase 3 / Phase 5 |
| MIR has TensorOp rvalues | Phase 4 |

## Architecture

### Tensor C struct

```c
typedef enum {
    AXON_F32 = 0,
    AXON_F64 = 1,
    AXON_I32 = 2,
    AXON_I64 = 3,
    AXON_BOOL = 4,
} AxonDtype;

typedef enum {
    AXON_CPU  = 0,
    AXON_CUDA = 1,
    AXON_ROCM = 2,
} AxonDevice;

typedef struct {
    void    *data;          // raw element buffer
    int64_t *shape;         // heap-allocated shape array
    int64_t *strides;       // heap-allocated strides (in elements)
    int32_t  ndim;          // number of dimensions
    int64_t  numel;         // total element count (cached)
    AxonDtype dtype;        // element data type
    AxonDevice device;      // CPU / CUDA / ROCm
    int32_t  refcount;      // atomic reference count
    uint8_t  owns_data;     // 0 for views, 1 for owning tensors
    void    *base;          // if view: pointer to owning AxonTensor
} AxonTensor;
```

The LLVM IR representation of a tensor is a **pointer to AxonTensor**
(`%AxonTensor*`), not an inline struct.  This lets us pass tensors cheaply
(single pointer) and manage lifetimes through refcounting.

### Design Decisions

1. **C runtime library** — All tensor ops are implemented in a C file
   (`runtime/axon_tensor.c`) compiled to `libaxon_rt.a` / `axon_rt.lib`.
   This avoids bloating LLVM IR with tensor logic and lets us link
   optimised BLAS directly.

2. **OpenBLAS for matmul** — We link OpenBLAS for `sgemm`/`dgemm`.  On
   systems without OpenBLAS we fall back to a naive triple-loop.  On
   macOS we can use the Accelerate framework.

3. **NumPy broadcasting** — Element-wise ops follow NumPy broadcasting
   rules: trailing dimensions are matched, size-1 dims are broadcast.

4. **Reference counting** — `axon_tensor_retain(t)` increments refcount,
   `axon_tensor_release(t)` decrements and frees at zero.  Views hold
   a retain on their base tensor.

5. **Row-major (C-contiguous) default** — Strides are computed
   row-major.  Transpose creates a view with swapped strides.

6. **dtype dispatch** — Every op has a `switch (dtype)` that dispatches
   to the correctly-typed implementation.  Macros generate the per-dtype
   bodies.

### File Layout

```
runtime/
├── axon_tensor.h        # public header (AxonTensor, all prototypes)
├── axon_tensor.c        # tensor core: alloc, free, retain, release, print
├── axon_tensor_create.c # zeros, ones, rand, from_data, arange, linspace, eye, full
├── axon_tensor_index.c  # indexing, slicing, gather, scatter
├── axon_tensor_view.c   # reshape, transpose, permute, squeeze, unsqueeze, contiguous
├── axon_tensor_elemwise.c  # add, sub, mul, div, neg, exp, log, sqrt, pow, abs + broadcast
├── axon_tensor_reduce.c    # sum, mean, max, min, argmax, argmin
├── axon_tensor_matmul.c    # matmul, dot, outer, batch_matmul (BLAS dispatch)
├── axon_tensor_compare.c   # eq, ne, lt, gt, le, ge, where
├── axon_tensor_activate.c  # relu, sigmoid, tanh, softmax, gelu
└── Makefile / build.rs     # compile runtime into static library
```

---

## Task List

### Sub-phase 10a: Tensor Core (T306–T311)

#### T306 — Define AxonTensor struct and header
- **File**: `runtime/axon_tensor.h`
- **Description**: Define `AxonTensor`, `AxonDtype`, `AxonDevice` enums and
  the struct layout shown above.  Declare all public API prototypes.
- **Acceptance**: Header compiles with `clang -c -fsyntax-only`.

#### T307 — Implement tensor allocation and lifecycle
- **File**: `runtime/axon_tensor.c`
- **Description**: Implement `axon_tensor_new(ndim, shape, dtype)`,
  `axon_tensor_retain(t)`, `axon_tensor_release(t)`, and helper
  `axon_tensor_clone(t)` (deep copy).  Strides computed row-major.
  `numel` cached.  `axon_tensor_release` frees data, shape, strides when
  refcount hits zero.  Views (`owns_data == 0`) release their base instead.
- **Acceptance**: Alloc/free cycle passes valgrind (Linux) or ASAN.

#### T308 — Tensor print and debug
- **File**: `runtime/axon_tensor.c`
- **Description**: `axon_tensor_print(t)` — prints tensor in a readable
  format like `Tensor([1.0, 2.0, 3.0], shape=[3], dtype=f32)`.  For 2D+,
  prints nested brackets.  `axon_tensor_debug(t)` prints metadata
  (shape, strides, dtype, refcount, device).
- **Acceptance**: Print output matches expected format for 1D, 2D, 3D tensors.

#### T309 — Update LLVM IR tensor representation
- **File**: `src/codegen/llvm.rs`
- **Description**: Change tensor LLVM type from inline `{ ptr, ptr, i32 }`
  to `ptr` (opaque pointer to AxonTensor).  All tensor operations become
  calls to `axon_tensor_*` runtime functions.  Update `emit_type()`,
  `emit_rvalue()` for tensor cases.
- **Acceptance**: Existing codegen tests still pass with updated representation.

#### T310 — Update MIR tensor lowering
- **File**: `src/mir.rs`
- **Description**: Ensure MIR TensorOps lower to `Call` terminators
  targeting `axon_tensor_*` functions instead of inline rvalues.  This
  means `MirRvalue::TensorOp(MatMul, a, b)` becomes
  `Call { func: "axon_tensor_matmul", args: [a, b], .. }`.
- **Acceptance**: MIR dump for a tensor program shows Call nodes.

#### T311 — Build system for runtime library
- **File**: `build.rs` or `scripts/build_runtime.py`
- **Description**: Compile all `runtime/*.c` files into a static library
  (`libaxon_rt.a` on Unix, `axon_rt.lib` on Windows).  The `axonc build`
  command links this library automatically.  Detect OpenBLAS presence
  and link if available.
- **Acceptance**: `axonc build tensor_hello.axon` links runtime successfully.

### Sub-phase 10b: Tensor Creation (T312–T316)

#### T312 — zeros, ones, full
- **File**: `runtime/axon_tensor_create.c`
- **Description**: `axon_tensor_zeros(ndim, shape, dtype)`,
  `axon_tensor_ones(...)`, `axon_tensor_full(ndim, shape, dtype, value)`.
  Allocate tensor, memset or fill with value.
- **Acceptance**: `Tensor::zeros([2,3])` creates a 2×3 tensor of all 0.0.

#### T313 — rand, randn
- **File**: `runtime/axon_tensor_create.c`
- **Description**: `axon_tensor_rand(...)` — uniform [0,1),
  `axon_tensor_randn(...)` — standard normal (Box-Muller).
  Use a seedable PRNG (xoshiro256++).
- **Acceptance**: 10000-element rand tensor has mean ≈ 0.5 ± 0.02.

#### T314 — from_data
- **File**: `runtime/axon_tensor_create.c`
- **Description**: `axon_tensor_from_data(data, ndim, shape, dtype)` —
  copies `data` into a new tensor.  Used for literal tensor construction.
- **Acceptance**: Round-trip: create from data, read back, values match.

#### T315 — arange, linspace
- **File**: `runtime/axon_tensor_create.c`
- **Description**: `axon_tensor_arange(start, stop, step, dtype)` — like
  NumPy arange.  `axon_tensor_linspace(start, stop, n, dtype)`.
- **Acceptance**: `arange(0, 5, 1)` → `[0, 1, 2, 3, 4]`.

#### T316 — eye (identity matrix)
- **File**: `runtime/axon_tensor_create.c`
- **Description**: `axon_tensor_eye(n, dtype)` — n×n identity matrix.
- **Acceptance**: `eye(3)` → 3×3 with 1s on diagonal, 0s elsewhere.

### Sub-phase 10c: Indexing & Views (T317–T322)

#### T317 — Single-element indexing
- **File**: `runtime/axon_tensor_index.c`
- **Description**: `axon_tensor_get_f32(t, indices)`,
  `axon_tensor_set_f32(t, indices, val)` — read/write single elements.
  Support negative indexing (`-1` = last).  Bounds check with panic.
- **Acceptance**: `t.get([1, 2])` returns correct element.

#### T318 — Slice views
- **File**: `runtime/axon_tensor_index.c`
- **Description**: `axon_tensor_slice(t, ranges)` — returns a view
  (owns_data=0) with adjusted data pointer, shape, and strides.
  Range is `{start, stop, step}` per dimension.
- **Acceptance**: Slicing doesn't copy data; modifying view modifies original.

#### T319 — reshape
- **File**: `runtime/axon_tensor_view.c`
- **Description**: `axon_tensor_reshape(t, new_shape, new_ndim)`.
  If contiguous, returns a view with new shape/strides.
  If not contiguous, copies to contiguous first.  Validate `numel` matches.
- **Acceptance**: reshape `[2,6]` → `[3,4]` works; mismatched numel panics.

#### T320 — transpose & permute
- **File**: `runtime/axon_tensor_view.c`
- **Description**: `axon_tensor_transpose(t)` — swap last two dims (view).
  `axon_tensor_permute(t, axes, naxes)` — arbitrary axis permutation (view).
- **Acceptance**: transpose of `[2,3]` tensor has shape `[3,2]`.

#### T321 — squeeze & unsqueeze
- **File**: `runtime/axon_tensor_view.c`
- **Description**: `axon_tensor_squeeze(t, dim)` — remove size-1 dim.
  `axon_tensor_unsqueeze(t, dim)` — add size-1 dim.  Both return views.
- **Acceptance**: unsqueeze `[3]` at dim 0 → `[1,3]`.

#### T322 — contiguous
- **File**: `runtime/axon_tensor_view.c`
- **Description**: `axon_tensor_contiguous(t)` — if already contiguous,
  retain and return.  Otherwise allocate new tensor and copy data in
  row-major order.
- **Acceptance**: After transpose + contiguous, strides are row-major.

### Sub-phase 10d: Element-wise Ops & Broadcasting (T323–T328)

#### T323 — Broadcasting engine
- **File**: `runtime/axon_tensor_elemwise.c`
- **Description**: Implement `broadcast_shapes(a_shape, a_ndim, b_shape,
  b_ndim, out_shape, out_ndim)` following NumPy rules:
  1. Right-align shapes.
  2. Each dim: must be equal, or one must be 1.
  3. Output dim = max of the two.
  Implement `broadcast_index(out_idx, shape, ndim)` to map output index
  to input index (clamp dims with size 1 to 0).
- **Acceptance**: `[3,1] + [1,4]` broadcasts to `[3,4]`.

#### T324 — Binary element-wise ops
- **File**: `runtime/axon_tensor_elemwise.c`
- **Description**: `axon_tensor_add(a, b)`, `_sub`, `_mul`, `_div`,
  `_pow`.  All support broadcasting.  Allocate output tensor, iterate
  with broadcast indexing, apply op element-by-element.
  Macro-generate for each dtype.
- **Acceptance**: `ones([2,3]) + ones([2,3])` → all 2.0.

#### T325 — Unary element-wise ops
- **File**: `runtime/axon_tensor_elemwise.c`
- **Description**: `axon_tensor_neg(t)`, `_exp`, `_log`, `_sqrt`,
  `_abs`, `_sign`, `_ceil`, `_floor`.  Allocate output, iterate, apply.
- **Acceptance**: `exp(zeros([3]))` → all 1.0.

#### T326 — Scalar-tensor ops
- **File**: `runtime/axon_tensor_elemwise.c`
- **Description**: `axon_tensor_add_scalar(t, val)`, etc.  Avoids
  creating a scalar tensor for `t + 5.0`.
- **Acceptance**: `ones([3]) * 3.0` → `[3.0, 3.0, 3.0]`.

#### T327 — In-place element-wise ops
- **File**: `runtime/axon_tensor_elemwise.c`
- **Description**: `axon_tensor_add_(a, b)` (in-place on `a`).
  Only valid if `a` owns data and shapes are compatible.
  Used by optimizers for weight updates.
- **Acceptance**: In-place add modifies tensor, no allocation.

#### T328 — Clamp, min/max element-wise
- **File**: `runtime/axon_tensor_elemwise.c`
- **Description**: `axon_tensor_clamp(t, min, max)`,
  `axon_tensor_maximum(a, b)`, `axon_tensor_minimum(a, b)`.
- **Acceptance**: `clamp(tensor, 0.0, 1.0)` clips values.

### Sub-phase 10e: Reductions (T329–T331)

#### T329 — Sum and mean
- **File**: `runtime/axon_tensor_reduce.c`
- **Description**: `axon_tensor_sum(t, dim, keepdim)` — sum along axis
  (or all if dim == -1).  `axon_tensor_mean(t, dim, keepdim)`.
  Handle keepdim by preserving the reduced dimension as size 1.
- **Acceptance**: `sum(ones([2,3]), dim=1)` → `[3.0, 3.0]`.

#### T330 — Max, min, argmax, argmin
- **File**: `runtime/axon_tensor_reduce.c`
- **Description**: `axon_tensor_max(t, dim, keepdim)`,
  `axon_tensor_argmax(t, dim)` (returns i64 tensor of indices).
- **Acceptance**: `argmax([3.0, 1.0, 2.0])` → `0`.

#### T331 — Variance and standard deviation
- **File**: `runtime/axon_tensor_reduce.c`
- **Description**: `axon_tensor_var(t, dim, keepdim)`,
  `axon_tensor_std(t, dim, keepdim)`.  Use two-pass for numerical stability.
- **Acceptance**: `std(tensor([1,2,3,4,5]))` ≈ 1.414.

### Sub-phase 10f: Matrix Operations & BLAS (T332–T336)

#### T332 — Naive matmul fallback
- **File**: `runtime/axon_tensor_matmul.c`
- **Description**: `axon_tensor_matmul(a, b)` — triple-loop
  implementation for when BLAS is not available.
  Handle 1D (dot), 2D (matrix), and batched (3D+) cases.
- **Acceptance**: `matmul([2,3], [3,4])` → `[2,4]` with correct values.

#### T333 — OpenBLAS integration
- **File**: `runtime/axon_tensor_matmul.c`, build system
- **Description**: If OpenBLAS is available, call `cblas_sgemm`/`cblas_dgemm`
  for 2D f32/f64 matmul.  Detect at build time via pkg-config or explicit
  path.  On macOS, try Accelerate framework's `cblas_sgemm`.
- **Acceptance**: BLAS matmul within 2× of NumPy for 512×512 f32.

#### T334 — Dot product
- **File**: `runtime/axon_tensor_matmul.c`
- **Description**: `axon_tensor_dot(a, b)` — 1D dot product.  Use
  `cblas_sdot`/`cblas_ddot` if available, else loop.  Returns scalar tensor.
- **Acceptance**: `dot([1,2,3], [4,5,6])` → `32`.

#### T335 — Outer product
- **File**: `runtime/axon_tensor_matmul.c`
- **Description**: `axon_tensor_outer(a, b)` — 1D×1D → 2D.
- **Acceptance**: `outer([1,2], [3,4])` → `[[3,4],[6,8]]`.

#### T336 — Batch matmul
- **File**: `runtime/axon_tensor_matmul.c`
- **Description**: `axon_tensor_bmm(a, b)` — batch matrix multiply.
  Leading dims are batch dims.  Loop over batches, call matmul per batch.
- **Acceptance**: `bmm([B,M,K], [B,K,N])` → `[B,M,N]`.

### Sub-phase 10g: Comparisons & Activations (T337–T340)

#### T337 — Comparison ops
- **File**: `runtime/axon_tensor_compare.c`
- **Description**: `axon_tensor_eq(a, b)`, `_ne`, `_lt`, `_gt`, `_le`,
  `_ge`.  Return bool tensors (dtype=AXON_BOOL).  Support broadcasting.
- **Acceptance**: `eq(tensor([1,2,3]), tensor([1,0,3]))` → `[true, false, true]`.

#### T338 — Where (conditional select)
- **File**: `runtime/axon_tensor_compare.c`
- **Description**: `axon_tensor_where(cond, a, b)` — element-wise
  `cond ? a : b`.  `cond` is bool tensor.  Broadcast all three.
- **Acceptance**: `where([true,false], [1,2], [3,4])` → `[1, 4]`.

#### T339 — Activation functions
- **File**: `runtime/axon_tensor_activate.c`
- **Description**: `axon_tensor_relu(t)` — max(0, x).
  `axon_tensor_sigmoid(t)` — 1/(1+exp(-x)).
  `axon_tensor_tanh_(t)` — tanh(x).
  `axon_tensor_gelu(t)` — x * 0.5 * (1 + tanh(sqrt(2/π) * (x + 0.044715*x³))).
- **Acceptance**: `relu(tensor([-1, 0, 1]))` → `[0, 0, 1]`.

#### T340 — Softmax
- **File**: `runtime/axon_tensor_activate.c`
- **Description**: `axon_tensor_softmax(t, dim)` — numerically stable:
  subtract max along dim, then exp, then divide by sum.
- **Acceptance**: `softmax(tensor([1,2,3]), dim=0)` sums to 1.0.

---

## Error Codes

| Code  | Name                  | Description                                   |
|-------|-----------------------|-----------------------------------------------|
| E6001 | ShapeMismatchRuntime  | Element-wise op shapes not broadcastable      |
| E6002 | MatmulDimMismatch     | Inner dimensions don't match for matmul       |
| E6003 | ReshapeNumelMismatch  | Total elements differ in reshape              |
| E6004 | IndexOutOfBounds      | Tensor index exceeds dimension size           |
| E6005 | InvalidAxis           | Reduction/permute axis out of range           |
| E6006 | DtypeMismatch         | Binary op on tensors with different dtypes    |
| E6007 | AllocationFailure     | malloc returned NULL for tensor data          |
| E6008 | NonContiguousInplace  | In-place op on non-contiguous view            |

These are **runtime panics** (not compile-time errors), printed via
`axon_panic()` with the error code and a descriptive message.

---

## Test Plan

### Unit tests (C)
- Test each runtime function in isolation using a C test harness
- Verify numerical correctness against known values
- Test edge cases: empty tensors, scalar tensors, very large tensors

### Integration tests (Axon → compile → run → check stdout)
```
tests/tensor_runtime_tests.rs  — Rust integration tests that:
  1. Write an Axon program to a temp file
  2. Compile with axonc
  3. Run the binary
  4. Assert stdout matches expected output
```

### Numerical accuracy tests
- Compare results against NumPy for 50+ test cases
- Tolerance: < 1e-6 relative error for f32, < 1e-12 for f64

### Performance tests
- 512×512 matmul: < 2× NumPy (with BLAS) or < 50× (without BLAS)
- 1M element-wise add: < 5× NumPy

---

## Exit Criteria

1. All 35 tasks complete and tested
2. Tensor creation, element-wise, reduction, matmul, activation all produce
   numerically correct results
3. Broadcasting works for all ops
4. Reference counting has no leaks (valgrind/ASAN clean)
5. At least 60 integration tests pass
6. Matmul performance within 2× of NumPy when linked with OpenBLAS
