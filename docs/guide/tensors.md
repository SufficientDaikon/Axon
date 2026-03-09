# Tensor Programming

Tensors are first-class citizens in Axon. The type system tracks tensor shapes
at compile time, catching dimension mismatches before your code ever runs.

---

## Tensor Types and Shapes

Every tensor has a dtype and a shape encoded in its type:

```axon
// Static shape — all dimensions known at compile time
let weights: Tensor<Float32, [784, 256]> = randn([784, 256]);

// Dynamic batch dimension (?)
let input: Tensor<Float32, [?, 784]> = load_batch();

// Fully dynamic shape
let dynamic: Tensor<Float32, [?, ?]> = some_function();
```

### Shape Syntax

| Syntax      | Meaning                              |
| ----------- | ------------------------------------ |
| `[3, 4]`    | Static shape: 3 rows, 4 columns      |
| `[?, 784]`  | Dynamic first dim, static second dim |
| `[?, ?, 3]` | Batch × height × width, 3 channels   |
| `[N]`       | Named dimension (generic)            |

---

## Creating Tensors

### Initialization Functions

```axon
// Zeros and ones
let z = zeros([3, 4]);           // Tensor<Float32, [3, 4]>
let o = ones([256]);             // Tensor<Float32, [256]>

// Random initialization
let r = randn([128, 64]);       // normal distribution
let u = rand([10, 10]);         // uniform [0, 1)

// From data
let t = Tensor::from_vec([1.0, 2.0, 3.0, 4.0], [2, 2]);

// Range
let seq = arange(0, 10);        // [0, 1, 2, ..., 9]

// Identity matrix
let eye = Tensor::eye(4);       // 4×4 identity

// From file
let data = load_data("weights.npy");
```

### Dtype Selection

```axon
let f16: Tensor<Float16, [1024]> = zeros([1024]);    // half precision
let f32: Tensor<Float32, [1024]> = zeros([1024]);    // single precision
let f64: Tensor<Float64, [1024]> = zeros([1024]);    // double precision
let i32: Tensor<Int32, [10]> = arange(0, 10);        // integer tensor
```

---

## Shape Operations

### Reshape

Change the shape without changing the data:

```axon
let a: Tensor<Float32, [2, 6]> = randn([2, 6]);
let b = a.reshape([3, 4]);      // Tensor<Float32, [3, 4]>
let c = a.reshape([12]);        // Tensor<Float32, [12]>
// let d = a.reshape([5, 5]);   // ERROR[E3002]: cannot reshape [2,6] (12 elements) to [5,5] (25 elements)
```

### Transpose

```axon
let m: Tensor<Float32, [3, 4]> = randn([3, 4]);
let mt = m.transpose();         // Tensor<Float32, [4, 3]>

// For higher-rank tensors, specify axes
let t: Tensor<Float32, [2, 3, 4]> = randn([2, 3, 4]);
let tp = t.permute([0, 2, 1]);  // Tensor<Float32, [2, 4, 3]>
```

### Squeeze and Unsqueeze

```axon
let a: Tensor<Float32, [1, 3, 1, 4]> = randn([1, 3, 1, 4]);
let b = a.squeeze();            // Tensor<Float32, [3, 4]>

let c: Tensor<Float32, [3, 4]> = randn([3, 4]);
let d = c.unsqueeze(0);         // Tensor<Float32, [1, 3, 4]>
```

### Concatenation and Stacking

```axon
let a: Tensor<Float32, [2, 3]> = randn([2, 3]);
let b: Tensor<Float32, [2, 3]> = randn([2, 3]);

let cat = Tensor::cat([a, b], 0);    // Tensor<Float32, [4, 3]>
let stk = Tensor::stack([a, b], 0);  // Tensor<Float32, [2, 2, 3]>
```

### Slicing

```axon
let t: Tensor<Float32, [10, 20]> = randn([10, 20]);
let row = t[0];                  // Tensor<Float32, [20]>
let sub = t[2..5];               // Tensor<Float32, [3, 20]>
```

---

## Element-Wise Operations

Standard arithmetic operators work element-wise on tensors:

```axon
let a = randn([3, 4]);
let b = randn([3, 4]);

let sum  = a + b;     // element-wise addition
let diff = a - b;     // element-wise subtraction
let prod = a * b;     // element-wise multiplication (Hadamard)
let quot = a / b;     // element-wise division

// Scalar broadcasting
let scaled = a * 2.0;
let shifted = a + 1.0;
```

### Math Functions

```axon
let x = randn([100]);

let s  = x.sin();
let c  = x.cos();
let e  = x.exp();
let l  = x.log();
let sq = x.sqrt();
let ab = x.abs();
let cl = x.clamp(-1.0, 1.0);
```

### Activation Functions

```axon
let h = relu(x);
let g = gelu(x);
let s = sigmoid(x);
let t = tanh(x);
let p = softmax(logits, dim: 1);
```

---

## Reduction Operations

Reduce tensors along axes:

```axon
let t: Tensor<Float32, [4, 5]> = randn([4, 5]);

let total = t.sum();              // scalar
let row_sum = t.sum(dim: 1);     // Tensor<Float32, [4]>
let col_mean = t.mean(dim: 0);   // Tensor<Float32, [5]>
let max_val = t.max();            // scalar
let min_idx = t.argmin(dim: 1);  // Tensor<Int64, [4]>
```

### Common Reductions

| Method              | Description                |
| ------------------- | -------------------------- |
| `.sum()`            | Sum of all elements        |
| `.sum(dim: N)`      | Sum along dimension N      |
| `.mean()`           | Mean of all elements       |
| `.max()` / `.min()` | Maximum / minimum          |
| `.argmax(dim: N)`   | Index of maximum along dim |
| `.argmin(dim: N)`   | Index of minimum along dim |
| `.prod()`           | Product of all elements    |
| `.norm(p)`          | Lp norm                    |

---

## Linear Algebra

### Matrix Multiplication (`@` operator)

The `@` operator performs matrix multiplication with compile-time shape checking:

```axon
let A: Tensor<Float32, [3, 4]> = randn([3, 4]);
let B: Tensor<Float32, [4, 5]> = randn([4, 5]);
let C = A @ B;    // Tensor<Float32, [3, 5]>

// Inner dimensions must match
let D: Tensor<Float32, [4, 6]> = randn([4, 6]);
// let E = A @ D;   // ERROR[E3001]: matmul shape mismatch — inner dims 4 ≠ 4... wait
//                   // actually [3,4] @ [4,6] works. Let's show a real error:
let F: Tensor<Float32, [5, 6]> = randn([5, 6]);
// let G = A @ F;   // ERROR[E3001]: matmul requires inner dims to match: 4 ≠ 5
```

### Batch Matrix Multiplication

```axon
let batch_a: Tensor<Float32, [?, 8, 64]> = randn([32, 8, 64]);
let batch_b: Tensor<Float32, [?, 64, 32]> = randn([32, 64, 32]);
let batch_c = batch_a @ batch_b;   // Tensor<Float32, [?, 8, 32]>
```

### Other Linear Algebra Operations

```axon
let M = randn([4, 4]);

let d = M.det();              // determinant
let inv = M.inv();            // inverse
let (Q, R) = M.qr();         // QR decomposition
let (U, S, V) = M.svd();     // singular value decomposition
let eig = M.eigenvalues();   // eigenvalues
let tr = M.trace();           // trace
let dp = a.dot(b);            // dot product (1D tensors)
```

---

## Device Transfer

Tensors can be moved between CPU and GPU:

```axon
let cpu_tensor = randn([1024, 1024]);

// Move to GPU
let gpu_tensor = cpu_tensor.to_gpu();

// Compute on GPU
let result = gpu_tensor @ gpu_tensor;

// Move back to CPU for I/O
let cpu_result = result.to_cpu();
println("{}", cpu_result);
```

See [GPU Programming](gpu-programming.md) for details.

---

## Compile-Time Shape Checking

Axon's shape checker catches errors at compile time:

```axon
// ✓ Shapes match
let a: Tensor<Float32, [3, 4]> = randn([3, 4]);
let b: Tensor<Float32, [4, 5]> = randn([4, 5]);
let c = a @ b;    // OK: [3,4] @ [4,5] → [3,5]

// ✗ Shape mismatch
let d: Tensor<Float32, [3, 4]> = randn([3, 4]);
let e: Tensor<Float32, [5, 6]> = randn([5, 6]);
// let f = d @ e;   // ERROR[E3001]: matmul inner dim mismatch: 4 ≠ 5

// ✗ Invalid reshape
let g: Tensor<Float32, [2, 3]> = randn([2, 3]);
// let h = g.reshape([2, 2]);  // ERROR[E3002]: element count mismatch: 6 ≠ 4

// ✗ Element-wise shape mismatch
let i: Tensor<Float32, [3, 4]> = randn([3, 4]);
let j: Tensor<Float32, [3, 5]> = randn([3, 5]);
// let k = i + j;   // ERROR[E3003]: broadcast incompatible shapes [3,4] and [3,5]
```

### Dynamic Shapes

When dimensions are dynamic (`?`), shape checks happen at runtime:

```axon
fn process(input: Tensor<Float32, [?, 784]>) -> Tensor<Float32, [?, 10]> {
    let w = randn([784, 10]);
    input @ w    // batch dim (?) propagated, inner dim (784) checked statically
}
```

---

## Summary

| Feature      | Example                     |
| ------------ | --------------------------- |
| Static shape | `Tensor<Float32, [3, 4]>`   |
| Dynamic dim  | `Tensor<Float32, [?, 784]>` |
| Matmul       | `A @ B`                     |
| Element-wise | `a + b`, `a * 2.0`          |
| Reduction    | `t.sum(dim: 1)`             |
| Reshape      | `t.reshape([6, 2])`         |
| Device       | `t.to_gpu()`, `t.to_cpu()`  |
| Shape error  | Caught at compile time      |

---

## See Also

- [GPU Programming](gpu-programming.md) — device management and GPU kernels
- [Tutorial: Hello Tensor](../tutorial/01-hello-tensor.md) — hands-on tensor basics
- [Compiler Errors: E3xxx](../reference/compiler-errors.md) — shape error codes
