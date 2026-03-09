# Phase 11: Autograd Engine

## Summary

Phase 11 implements automatic differentiation — the core mechanism that
enables training neural networks.  After this phase, a user can write:

```axon
let x = Tensor::rand([32, 784]);
let w = Tensor::rand([784, 10], requires_grad: true);
let y = (x @ w).relu().sum();
y.backward();
println(w.grad);   // 784×10 gradient tensor
```

The engine uses **reverse-mode autodiff (backpropagation)** with a
tape-based computation graph recorded during the forward pass.

## Prerequisites

| Requirement | Provided by |
|-------------|-------------|
| All tensor ops execute correctly on CPU | Phase 10 |
| Tensor struct has refcounting and views | Phase 10 |
| Element-wise, reduction, matmul, activations work | Phase 10 |

## Architecture

### Design Decisions

1. **Tape-based (define-by-run)** — Like PyTorch, not TensorFlow's
   define-then-run.  The graph is recorded dynamically as ops execute.
   This simplifies control flow (if/else in forward pass just works).

2. **Gradient stored on tensors** — Each tensor with `requires_grad=true`
   has a `.grad` field (initially null, populated by `backward()`).

3. **GradFn nodes** — Each output tensor records its creation op and
   input tensors.  `backward()` traverses this graph in reverse
   topological order, calling each op's backward function.

4. **Implemented in C** — Extends the tensor runtime.  The autograd
   tape and GradFn are C structs.

5. **Gradient accumulation** — If a tensor is used in multiple ops,
   gradients are summed (accumulated), not replaced.

6. **No-grad context** — `axon_autograd_set_enabled(0)` disables
   recording for inference, enabling/disabling via RAII in Axon.

### Data Structures

```c
// Forward declaration
typedef struct AxonGradFn AxonGradFn;

// Extended tensor struct (additions to AxonTensor)
// - requires_grad: bool
// - grad: AxonTensor* (null until backward called)
// - grad_fn: AxonGradFn* (null for leaf tensors)
// - retain_grad: bool (keep grad for non-leaf tensors)

typedef void (*BackwardFn)(AxonGradFn *self, AxonTensor *grad_output);

typedef struct AxonGradFn {
    BackwardFn backward;       // the backward function pointer
    AxonTensor **inputs;       // saved tensors needed for backward
    int n_inputs;              // number of saved tensors
    AxonGradFn **input_fns;    // grad_fns of inputs (for graph traversal)
    int n_input_fns;
    AxonTensor **saved_tensors; // any tensors saved for backward
    int n_saved;
    void *metadata;            // op-specific metadata (e.g., axis for sum)
    int visited;               // for topological sort
} AxonGradFn;
```

### Computation Graph Example

```
Forward: y = relu(x @ w + b).sum()

Tape records:
  matmul_backward  ←  x, w
       ↓
  add_backward     ←  (matmul_result), b
       ↓
  relu_backward    ←  (add_result)
       ↓
  sum_backward     ←  (relu_result)

backward() traverses in reverse: sum → relu → add → matmul
```

---

## Task List

### Sub-phase 11a: Autograd Core (T341–T347)

#### T341 — Extend AxonTensor with grad fields
- **File**: `runtime/axon_tensor.h`, `runtime/axon_tensor.c`
- **Description**: Add `requires_grad`, `grad`, `grad_fn`, `retain_grad`
  fields to AxonTensor.  Initialize to 0/NULL in `axon_tensor_new`.
  Add `axon_tensor_set_requires_grad(t, bool)`.
- **Acceptance**: Creating a tensor with requires_grad=true stores the flag.

#### T342 — GradFn struct and allocation
- **File**: `runtime/axon_autograd.h`, `runtime/axon_autograd.c`
- **Description**: Define `AxonGradFn` struct.  Implement
  `axon_gradfn_new(backward_fn, n_inputs)`, `axon_gradfn_free(gf)`,
  `axon_gradfn_save_tensor(gf, tensor)`.
- **Acceptance**: GradFn alloc/free cycle is leak-free.

#### T343 — Autograd tape and enable/disable
- **File**: `runtime/axon_autograd.c`
- **Description**: Thread-local `autograd_enabled` flag (default true).
  `axon_autograd_set_enabled(bool)`, `axon_autograd_is_enabled()`.
  When disabled, ops skip GradFn creation.
  `axon_no_grad_begin()` / `axon_no_grad_end()` for scoped disable.
- **Acceptance**: Ops inside no_grad don't create GradFn nodes.

#### T344 — Topological sort
- **File**: `runtime/axon_autograd.c`
- **Description**: `axon_autograd_topo_sort(root_grad_fn)` — DFS from
  root, produce reverse topological order of GradFn nodes.  Handle
  diamond dependencies (a tensor used by multiple ops).
- **Acceptance**: Sort of `y = (a+b) * (a-b)` visits a's grad_fn once.

#### T345 — backward() implementation
- **File**: `runtime/axon_autograd.c`
- **Description**: `axon_tensor_backward(tensor)`:
  1. Assert tensor is scalar (numel == 1) or grad_output provided.
  2. Initialize grad_output to ones if scalar.
  3. Topological sort from tensor.grad_fn.
  4. For each GradFn in order, call its backward function.
  5. Accumulate gradients onto leaf tensor `.grad` fields.
- **Acceptance**: `y = x * 2; y.backward()` sets `x.grad = 2.0`.

#### T346 — zero_grad()
- **File**: `runtime/axon_autograd.c`
- **Description**: `axon_tensor_zero_grad(t)` — if `t.grad` exists,
  fill with zeros.  Also `axon_autograd_zero_all_grads(tensors, n)`.
- **Acceptance**: After zero_grad, all grad values are 0.

#### T347 — Gradient accumulation
- **File**: `runtime/axon_autograd.c`
- **Description**: When a leaf tensor receives a gradient but `.grad`
  already exists, ADD the new gradient to the existing one (don't replace).
  This handles the common case where a parameter is used in multiple ops.
- **Acceptance**: `y = x + x; y.backward()` → `x.grad = 2.0` (not 1.0).

### Sub-phase 11b: Backward Implementations (T348–T358)

#### T348 — Add/Sub backward
- **File**: `runtime/axon_autograd_ops.c`
- **Description**: 
  `add_backward`: `grad_a = grad_out`, `grad_b = grad_out` (with broadcast reduction).
  `sub_backward`: `grad_a = grad_out`, `grad_b = -grad_out`.
  Handle broadcasting: if input was broadcast, sum grad along broadcast dims.
- **Acceptance**: Gradient check passes with < 1e-5 relative error.

#### T349 — Mul/Div backward
- **File**: `runtime/axon_autograd_ops.c`
- **Description**:
  `mul_backward`: `grad_a = grad_out * b`, `grad_b = grad_out * a`.
  `div_backward`: `grad_a = grad_out / b`, `grad_b = -grad_out * a / (b*b)`.
  Save inputs for backward.  Handle broadcast reduction.
- **Acceptance**: Gradient check passes.

#### T350 — Matmul backward
- **File**: `runtime/axon_autograd_ops.c`
- **Description**:
  `matmul_backward(grad_out, a, b)`:
  `grad_a = grad_out @ b^T`, `grad_b = a^T @ grad_out`.
  For batched matmul, apply to each batch.
  Save both inputs.
- **Acceptance**: Gradient check for `y = a @ b` passes.

#### T351 — Sum/Mean backward
- **File**: `runtime/axon_autograd_ops.c`
- **Description**:
  `sum_backward`: `grad_input = broadcast grad_out to input shape`.
  If `dim` specified, unsqueeze grad_out at that dim, then broadcast.
  `mean_backward`: same as sum_backward but divide by N.
  Save input shape and dim as metadata.
- **Acceptance**: `y = x.sum(); y.backward()` → `x.grad = ones_like(x)`.

#### T352 — ReLU backward
- **File**: `runtime/axon_autograd_ops.c`
- **Description**: `relu_backward`: `grad_input = grad_out * (input > 0)`.
  Save input tensor (or just the mask).
- **Acceptance**: Gradient is 0 where input was ≤ 0.

#### T353 — Sigmoid backward
- **File**: `runtime/axon_autograd_ops.c`
- **Description**: `sigmoid_backward`:
  `grad_input = grad_out * sigmoid_output * (1 - sigmoid_output)`.
  Save output (more numerically stable than recomputing).
- **Acceptance**: Gradient check passes.

#### T354 — Tanh backward
- **File**: `runtime/axon_autograd_ops.c`
- **Description**: `tanh_backward`:
  `grad_input = grad_out * (1 - tanh_output^2)`.
  Save output.
- **Acceptance**: Gradient check passes.

#### T355 — Softmax backward
- **File**: `runtime/axon_autograd_ops.c`
- **Description**: `softmax_backward(grad_out, softmax_output, dim)`:
  `s = softmax_output`
  `grad_input = s * (grad_out - sum(grad_out * s, dim=dim, keepdim=true))`
  Save output and dim.
- **Acceptance**: Gradient check passes for softmax along dim=1.

#### T356 — Log/Exp/Sqrt backward
- **File**: `runtime/axon_autograd_ops.c`
- **Description**:
  `log_backward`: `grad_input = grad_out / input`.
  `exp_backward`: `grad_input = grad_out * exp_output`.
  `sqrt_backward`: `grad_input = grad_out / (2 * sqrt_output)`.
  Save relevant inputs/outputs.
- **Acceptance**: Gradient check passes for each.

#### T357 — Pow backward
- **File**: `runtime/axon_autograd_ops.c`
- **Description**: `pow_backward(grad_out, base, exponent)`:
  `grad_base = grad_out * exponent * base^(exponent-1)`.
  `grad_exp = grad_out * base^exponent * log(base)` (if exponent requires grad).
- **Acceptance**: Gradient check passes.

#### T358 — Neg/Abs backward
- **File**: `runtime/axon_autograd_ops.c`
- **Description**:
  `neg_backward`: `grad_input = -grad_out`.
  `abs_backward`: `grad_input = grad_out * sign(input)`.
- **Acceptance**: Gradient check passes.

### Sub-phase 11c: Wire Autograd into Tensor Ops (T359–T362)

#### T359 — Wrap element-wise ops with autograd
- **File**: `runtime/axon_tensor_elemwise.c`
- **Description**: Modify `axon_tensor_add`, `_sub`, `_mul`, etc. to:
  1. Check if any input has `requires_grad`.
  2. If yes AND autograd enabled, create GradFn, save inputs, attach to output.
  3. Set output `requires_grad = true`.
- **Acceptance**: After forward pass, output has grad_fn chain.

#### T360 — Wrap matmul with autograd
- **File**: `runtime/axon_tensor_matmul.c`
- **Description**: Same wrapping for matmul, dot, bmm.
- **Acceptance**: `y = a @ b; y.backward()` computes a.grad and b.grad.

#### T361 — Wrap reductions with autograd
- **File**: `runtime/axon_tensor_reduce.c`
- **Description**: Wrap sum, mean with autograd GradFn creation.
- **Acceptance**: `y = x.sum(); y.backward()` → x.grad = ones.

#### T362 — Wrap activations with autograd
- **File**: `runtime/axon_tensor_activate.c`
- **Description**: Wrap relu, sigmoid, tanh, softmax, gelu with autograd.
- **Acceptance**: `y = x.relu().sum(); y.backward()` computes correct grad.

### Sub-phase 11d: Numerical Verification (T363–T366)

#### T363 — Numerical gradient checker
- **File**: `runtime/axon_autograd_check.c`
- **Description**: `axon_gradient_check(fn, inputs, n_inputs, epsilon)`:
  For each element of each input, compute `(f(x+ε) - f(x-ε)) / (2ε)`.
  Compare with analytical gradient.  Return max relative error.
- **Acceptance**: Function works for scalar → scalar functions.

#### T364 — Gradient check test suite
- **File**: `tests/autograd_tests.rs`
- **Description**: Write gradient check tests for every differentiable op:
  add, sub, mul, div, matmul, sum, mean, relu, sigmoid, tanh, softmax,
  exp, log, sqrt, pow, neg, abs.  Use random inputs.
- **Acceptance**: All relative errors < 1e-4 (f32) or 1e-8 (f64).

#### T365 — Multi-op chain gradient test
- **File**: `tests/autograd_tests.rs`
- **Description**: Test gradient through chains of ops:
  `y = relu(x @ w + b).mean()`.  Verify w.grad, b.grad against numerical.
- **Acceptance**: Relative error < 1e-4.

#### T366 — Gradient accumulation and diamond test
- **File**: `tests/autograd_tests.rs`
- **Description**: Test cases where a tensor is used multiple times:
  `y = x * x` (gradient should be 2x).
  `y = (x + x).sum()` (gradient should be 2).
  `y = x @ w1 + x @ w2` (x.grad accumulates from both paths).
- **Acceptance**: All accumulation correct.

---

## Error Codes

| Code  | Name                  | Description                                  |
|-------|-----------------------|----------------------------------------------|
| E7001 | BackwardNonScalar     | backward() called on non-scalar tensor       |
| E7002 | NoGradFn             | backward() called on leaf tensor             |
| E7003 | GradRequiresGrad     | Trying to differentiate through non-grad tensor |
| E7004 | InplaceGradConflict  | In-place op on tensor needed for backward    |

---

## Test Plan

1. **Unit tests** — each backward function tested in isolation
2. **Numerical gradient checks** — every op verified against finite differences
3. **Chain tests** — multi-op forward/backward verified
4. **Memory tests** — no leaks in grad_fn graph after backward
5. **Performance** — backward pass overhead < 3× forward pass

---

## Exit Criteria

1. All 26 tasks complete
2. `backward()` computes correct gradients for all supported ops
3. Numerical gradient check passes with < 1e-4 relative error (f32)
4. Gradient accumulation works for shared parameters
5. no_grad context prevents graph recording
6. No memory leaks in autograd graph
