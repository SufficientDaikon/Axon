# Phase 12: GPU Backend (CUDA + ROCm)

## Summary

Phase 12 brings GPU execution to Axon.  After this phase:

```axon
let x = Tensor::rand([1024, 784]).to_gpu();
let w = Tensor::rand([784, 256], requires_grad: true).to_gpu();
let y = (x @ w).relu().sum();
y.backward();                  // all on GPU
let grad = w.grad.to_cpu();    // bring gradient back
println(grad.shape());         // [784, 256]
```

The backend supports **NVIDIA GPUs via CUDA** and **AMD GPUs via
ROCm/HIP**, using vendor BLAS and DNN libraries for maximum performance.

## Prerequisites

| Requirement | Provided by |
|-------------|-------------|
| CPU tensor runtime working | Phase 10 |
| Autograd engine working on CPU | Phase 11 |
| Tensor struct has device field | Phase 10 |
| AxonTensor refcounting works | Phase 10 |

## Architecture

### Design Decisions

1. **Runtime library, not compiled kernels** — GPU ops are implemented
   in the C runtime via CUDA Runtime API / HIP API calls, not by
   emitting PTX from the compiler.  This is simpler and lets us use
   vendor libraries (cuBLAS, rocBLAS) directly.

2. **Unified abstraction layer** — A thin C abstraction (`axon_gpu.h`)
   wraps both CUDA and HIP.  Since HIP is source-compatible with CUDA
   for most APIs, we use HIP-style calls and compile with either
   `nvcc` (CUDA) or `hipcc` (ROCm).

3. **Lazy initialization** — GPU context is created on first
   `.to_gpu()` call, not at program start.

4. **Async execution with sync points** — GPU ops are async.  We
   synchronize before `.to_cpu()` or `.print()`.

5. **Caching allocator** — Reuse freed GPU memory blocks to avoid
   expensive `cudaMalloc` calls during training loops.

### Abstraction Layer

```c
// axon_gpu.h — unified GPU API

#if defined(AXON_USE_CUDA)
  #include <cuda_runtime.h>
  #include <cublas_v2.h>
  #include <cudnn.h>
  #define gpuMalloc          cudaMalloc
  #define gpuFree            cudaFree
  #define gpuMemcpy          cudaMemcpy
  #define gpuMemcpyH2D       cudaMemcpyHostToDevice
  #define gpuMemcpyD2H       cudaMemcpyDeviceToHost
  #define gpuMemcpyD2D       cudaMemcpyDeviceToDevice
  #define gpuDeviceSynchronize cudaDeviceSynchronize
  #define gpuGetDeviceCount  cudaGetDeviceCount
  #define gpuSetDevice       cudaSetDevice
  #define gpuStreamCreate    cudaStreamCreate
  #define gpuStreamDestroy   cudaStreamDestroy
  #define GPU_SUCCESS        cudaSuccess
  typedef cudaStream_t       gpuStream_t;
  typedef cublasHandle_t     gpuBlasHandle_t;
#elif defined(AXON_USE_ROCM)
  #include <hip/hip_runtime.h>
  #include <rocblas/rocblas.h>
  #include <miopen/miopen.h>
  #define gpuMalloc          hipMalloc
  #define gpuFree            hipFree
  #define gpuMemcpy          hipMemcpy
  #define gpuMemcpyH2D       hipMemcpyHostToDevice
  #define gpuMemcpyD2H       hipMemcpyDeviceToHost
  #define gpuMemcpyD2D       hipMemcpyDeviceToDevice
  #define gpuDeviceSynchronize hipDeviceSynchronize
  #define gpuGetDeviceCount  hipGetDeviceCount
  #define gpuSetDevice       hipSetDevice
  #define gpuStreamCreate    hipStreamCreate
  #define gpuStreamDestroy   hipStreamDestroy
  #define GPU_SUCCESS        hipSuccess
  typedef hipStream_t        gpuStream_t;
  typedef rocblas_handle     gpuBlasHandle_t;
#else
  // CPU-only build — GPU ops panic with "GPU support not compiled"
#endif
```

### File Layout

```
runtime/
├── axon_gpu.h              # unified CUDA/HIP abstraction
├── axon_gpu.c              # GPU context init, device management
├── axon_gpu_memory.c       # device alloc/free, caching allocator, transfers
├── axon_gpu_elemwise.cu    # element-wise GPU kernels
├── axon_gpu_reduce.cu      # reduction GPU kernels
├── axon_gpu_matmul.c       # cuBLAS/rocBLAS matmul dispatch
├── axon_gpu_activate.cu    # activation GPU kernels
├── axon_gpu_autograd.c     # GPU-aware backward pass
└── axon_gpu_nn.c           # cuDNN/MIOpen conv, batchnorm, attention
```

`.cu` files are compiled with `nvcc` (CUDA) or `hipcc` (ROCm).
`.c` files use the C API only (no kernel launches).

---

## Task List

### Sub-phase 12a: GPU Context & Memory (T367–T374)

#### T367 — GPU abstraction header
- **File**: `runtime/axon_gpu.h`
- **Description**: Define the unified `gpu*` macros shown above.
  Include device query functions: `axon_gpu_device_count()`,
  `axon_gpu_set_device(id)`, `axon_gpu_get_device()`,
  `axon_gpu_device_name(id)`, `axon_gpu_device_memory(id)`.
- **Acceptance**: Header compiles with both nvcc and hipcc.

#### T368 — GPU context initialization
- **File**: `runtime/axon_gpu.c`
- **Description**: `axon_gpu_init()` — lazy singleton.  Creates GPU
  context, cuBLAS/rocBLAS handle, stream.
  `axon_gpu_shutdown()` — destroys handles, frees cache.
  Register `atexit(axon_gpu_shutdown)`.
- **Acceptance**: Init/shutdown cycle completes without errors.

#### T369 — Device memory allocation
- **File**: `runtime/axon_gpu_memory.c`
- **Description**: `axon_gpu_alloc(size)` → `void*` on device.
  `axon_gpu_free(ptr)`.  Thin wrappers around gpuMalloc/gpuFree.
- **Acceptance**: Can allocate and free 1GB on GPU.

#### T370 — Caching allocator
- **File**: `runtime/axon_gpu_memory.c`
- **Description**: Pool-based caching allocator.  On free, blocks go
  to a free list keyed by size (rounded up to power-of-2).
  On alloc, check free list first.  Reduces malloc overhead in
  training loops from ~1ms to ~1μs.
  `axon_gpu_cache_clear()` — release all cached blocks.
- **Acceptance**: 1000 alloc/free cycles of same size: 0 gpuMalloc after first.

#### T371 — Host-to-device transfer (to_gpu)
- **File**: `runtime/axon_gpu_memory.c`
- **Description**: `axon_tensor_to_gpu(t)`:
  1. If already on GPU, retain and return.
  2. Allocate device memory.
  3. Copy data (gpuMemcpyH2D).
  4. Copy shape/strides to device (or keep on host — design choice).
  5. Return new tensor with device=AXON_CUDA/AXON_ROCM.
- **Acceptance**: `to_gpu()` of a `[1000,1000]` f32 tensor succeeds.

#### T372 — Device-to-host transfer (to_cpu)
- **File**: `runtime/axon_gpu_memory.c`
- **Description**: `axon_tensor_to_cpu(t)`:
  1. If already on CPU, retain and return.
  2. Synchronize stream.
  3. Allocate host memory.
  4. Copy data (gpuMemcpyD2H).
  5. Return new tensor with device=AXON_CPU.
- **Acceptance**: Round-trip `to_gpu()` → `to_cpu()` preserves all values.

#### T373 — Device-to-device copy
- **File**: `runtime/axon_gpu_memory.c`
- **Description**: `axon_tensor_gpu_clone(t)` — deep copy on device.
  Used for operations that need to modify data without affecting views.
- **Acceptance**: Clone creates independent copy on GPU.

#### T374 — GPU tensor lifecycle
- **File**: `runtime/axon_tensor.c` (modify)
- **Description**: Update `axon_tensor_release()` to call `axon_gpu_free()`
  for GPU tensors instead of `free()`.  Update `axon_tensor_print()` to
  auto-transfer to CPU before printing.
- **Acceptance**: GPU tensor alloc/free cycle is leak-free.

### Sub-phase 12b: GPU Element-wise Kernels (T375–T378)

#### T375 — Element-wise binary kernel
- **File**: `runtime/axon_gpu_elemwise.cu`
- **Description**: CUDA/HIP kernels for add, sub, mul, div, pow.
  One generic kernel with op enum parameter.  Handle broadcasting
  with stride-based indexing.  Block size 256, grid = ceil(numel/256).
- **Acceptance**: GPU `a + b` matches CPU result for broadcast cases.

#### T376 — Element-wise unary kernel
- **File**: `runtime/axon_gpu_elemwise.cu`
- **Description**: Kernels for neg, exp, log, sqrt, abs, sign.
- **Acceptance**: GPU results match CPU within 1e-6.

#### T377 — Scalar-tensor kernel
- **File**: `runtime/axon_gpu_elemwise.cu`
- **Description**: Kernels for `tensor + scalar`, etc.  Avoids
  creating a scalar tensor on GPU.
- **Acceptance**: `gpu_ones([1000]) * 3.0` → all 3.0.

#### T378 — GPU comparison kernels
- **File**: `runtime/axon_gpu_elemwise.cu`
- **Description**: eq, ne, lt, gt, le, ge, where on GPU.
  Output is bool tensor on GPU.
- **Acceptance**: GPU comparison matches CPU.

### Sub-phase 12c: GPU Reductions (T379–T381)

#### T379 — GPU sum reduction
- **File**: `runtime/axon_gpu_reduce.cu`
- **Description**: Tree-based parallel reduction for `sum(dim)`.
  Use shared memory for intra-block reduction, then reduce across blocks.
  Handle keepdim.  For full reduction (no dim), reduce all elements.
- **Acceptance**: GPU sum matches CPU sum for 10M elements.

#### T380 — GPU mean/max/min/argmax
- **File**: `runtime/axon_gpu_reduce.cu`
- **Description**: Mean (sum then divide), max, min, argmax, argmin
  reductions.  Argmax kernel tracks both value and index.
- **Acceptance**: GPU argmax matches CPU for random tensors.

#### T381 — GPU variance/std
- **File**: `runtime/axon_gpu_reduce.cu`
- **Description**: Two-pass: first compute mean, then sum of squared
  deviations.  Or use Welford's online algorithm in a single pass.
- **Acceptance**: GPU std matches CPU within 1e-5.

### Sub-phase 12d: GPU BLAS Integration (T382–T385)

#### T382 — cuBLAS/rocBLAS matmul
- **File**: `runtime/axon_gpu_matmul.c`
- **Description**: `axon_gpu_matmul(a, b)` — dispatch to
  `cublasSgemm` / `cublasDgemm` (CUDA) or
  `rocblas_sgemm` / `rocblas_dgemm` (ROCm).
  Handle column-major convention (BLAS) vs row-major (Axon) by
  swapping A,B and transposing result: `C = (B^T @ A^T)^T`.
- **Acceptance**: GPU matmul within 1.5× of PyTorch for 1024×1024.

#### T383 — Batch matmul on GPU
- **File**: `runtime/axon_gpu_matmul.c`
- **Description**: `cublasSgemmBatched` / `rocblas_sgemm_batched`
  for batched matmul.  Allocate pointer arrays on device.
- **Acceptance**: `bmm([B,M,K], [B,K,N])` correct on GPU.

#### T384 — GPU dot product
- **File**: `runtime/axon_gpu_matmul.c`
- **Description**: `cublasSdot` / `rocblas_sdot` for 1D dot product.
- **Acceptance**: GPU dot matches CPU.

#### T385 — Mixed precision matmul
- **File**: `runtime/axon_gpu_matmul.c`
- **Description**: Support FP16 (`cublasHgemm`), BF16, and TF32
  compute modes.  `axon_tensor_half(t)` converts f32→f16 on GPU.
  `cublasSetMathMode(CUBLAS_TF32_TENSOR_OP_MATH)` for automatic TF32.
- **Acceptance**: FP16 matmul produces results within FP16 tolerance.

### Sub-phase 12e: GPU Activations & cuDNN/MIOpen (T386–T390)

#### T386 — GPU activation kernels
- **File**: `runtime/axon_gpu_activate.cu`
- **Description**: relu, sigmoid, tanh, gelu as CUDA/HIP kernels.
  Simple element-wise kernels.
- **Acceptance**: GPU activations match CPU within 1e-6.

#### T387 — GPU softmax
- **File**: `runtime/axon_gpu_activate.cu`
- **Description**: Numerically stable softmax: find max (reduction),
  subtract, exp, sum (reduction), divide.  Or use cuDNN/MIOpen softmax.
- **Acceptance**: GPU softmax output sums to 1.0 along dim.

#### T388 — cuDNN/MIOpen conv2d forward
- **File**: `runtime/axon_gpu_nn.c`
- **Description**: 2D convolution using cuDNN `cudnnConvolutionForward`
  or MIOpen `miopenConvolutionForward`.  Descriptor setup for input,
  filter, output, convolution params (padding, stride, dilation).
  Algorithm selection: use `cudnnFindConvolutionForwardAlgorithm`.
- **Acceptance**: Conv2d output matches PyTorch for 3×3 kernel.

#### T389 — cuDNN/MIOpen batchnorm
- **File**: `runtime/axon_gpu_nn.c`
- **Description**: Batch normalization forward:
  `cudnnBatchNormalizationForwardTraining` / MIOpen equivalent.
  Running mean/var tracking.
- **Acceptance**: Batchnorm output matches PyTorch.

#### T390 — cuDNN/MIOpen pooling
- **File**: `runtime/axon_gpu_nn.c`
- **Description**: Max pooling and average pooling via cuDNN/MIOpen.
- **Acceptance**: MaxPool2d with 2×2 kernel produces correct output.

### Sub-phase 12f: GPU Autograd (T391–T395)

#### T391 — GPU-aware backward dispatch
- **File**: `runtime/axon_gpu_autograd.c`
- **Description**: Modify backward functions to check tensor device.
  If on GPU, use GPU kernels for gradient computation.  Gradients
  stay on GPU (no unnecessary transfers).
- **Acceptance**: `y.backward()` runs entirely on GPU for GPU tensors.

#### T392 — Conv2d backward on GPU
- **File**: `runtime/axon_gpu_autograd.c`
- **Description**: `cudnnConvolutionBackwardData` for input gradient,
  `cudnnConvolutionBackwardFilter` for weight gradient,
  `cudnnConvolutionBackwardBias` for bias gradient.
- **Acceptance**: Conv2d gradient check passes on GPU.

#### T393 — Batchnorm backward on GPU
- **File**: `runtime/axon_gpu_autograd.c`
- **Description**: `cudnnBatchNormalizationBackward` for batchnorm
  gradients (input grad, scale grad, bias grad).
- **Acceptance**: Batchnorm gradient check passes.

#### T394 — Multi-GPU device management
- **File**: `runtime/axon_gpu.c`
- **Description**: `axon_gpu_set_device(id)` — switch active device.
  Tensors track which device they're on.  Ops between tensors on
  different devices panic with a helpful error.
  `axon_tensor_to_device(t, device_id)` — move between GPUs.
- **Acceptance**: Can create tensors on GPU 0 and GPU 1 independently.

#### T395 — GPU memory diagnostics
- **File**: `runtime/axon_gpu_memory.c`
- **Description**: `axon_gpu_memory_allocated()` — total bytes currently
  allocated.  `axon_gpu_memory_cached()` — bytes in caching allocator.
  `axon_gpu_memory_peak()` — high-water mark.
  `axon_gpu_memory_summary()` — print human-readable report.
- **Acceptance**: Memory stats are accurate during training.

---

## Build System

### Compile Flags

```makefile
# CUDA build
nvcc -c -o axon_gpu_elemwise.o axon_gpu_elemwise.cu -DAXON_USE_CUDA
cc -c -o axon_gpu_matmul.o axon_gpu_matmul.c -DAXON_USE_CUDA -I/usr/local/cuda/include
# Link: -lcudart -lcublas -lcudnn

# ROCm build
hipcc -c -o axon_gpu_elemwise.o axon_gpu_elemwise.cu -DAXON_USE_ROCM
cc -c -o axon_gpu_matmul.o axon_gpu_matmul.c -DAXON_USE_ROCM -I/opt/rocm/include
# Link: -lamdhip64 -lrocblas -lMIOpen

# CPU-only build (default)
# No GPU files compiled.  GPU ops panic at runtime.
```

### Detection

The build system (Phase 9's build.rs / scripts) detects:
1. `nvcc` in PATH → CUDA build
2. `hipcc` in PATH → ROCm build
3. Neither → CPU-only build with runtime error on `.to_gpu()`

---

## Error Codes

| Code  | Name                  | Description                                |
|-------|-----------------------|--------------------------------------------|
| E8001 | GpuNotAvailable       | .to_gpu() called but no GPU compiled/detected |
| E8002 | GpuAllocFailed        | GPU memory allocation failed               |
| E8003 | GpuKernelFailed       | GPU kernel launch or execution error       |
| E8004 | DeviceMismatch        | Op on tensors from different devices       |
| E8005 | CublasError           | cuBLAS/rocBLAS returned error              |
| E8006 | CudnnError            | cuDNN/MIOpen returned error                |
| E8007 | GpuSyncFailed         | Device synchronization failed              |

---

## Test Plan

1. **Requires hardware** — GPU tests are `#[ignore]` by default, run
   with `cargo test -- --ignored` on GPU machines.
2. **CPU↔GPU round-trip** — every op tested with: compute on CPU, compute
   on GPU, transfer back, compare results.
3. **Multi-GPU** — if 2+ GPUs, test device selection and cross-device errors.
4. **Memory leak tests** — track GPU memory before and after ops.
5. **Performance benchmarks** — matmul, conv2d, training step vs PyTorch.

---

## Exit Criteria

1. All 29 tasks complete
2. `.to_gpu()` / `.to_cpu()` works for all dtypes
3. All tensor ops produce correct results on GPU (within floating-point tolerance)
4. cuBLAS/rocBLAS matmul within 1.5× of PyTorch
5. Autograd works entirely on GPU — no unnecessary CPU transfers
6. Caching allocator reduces allocation overhead by >90%
7. Conv2d, batchnorm, pooling via cuDNN/MIOpen work correctly
8. Clean build on both CUDA and ROCm systems
