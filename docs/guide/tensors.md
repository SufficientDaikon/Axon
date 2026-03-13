# Tensor Programming

Tensors are first-class citizens in Axon. The type system tracks tensor shapes
at compile time, catching dimension mismatches before your code ever runs.

---

## Tensor Types and Shapes

Every tensor has a dtype and a shape encoded in its type:

```axon
// Static shape — all dimensions known at compile time
val weights: Tensor<Float32, [784, 256]> = randn([784, 256]);

// Dynamic batch dimension (?)
val input: Tensor<Float32, [?, 784]> = load_batch();

// Fully dynamic shape
val dynamic: Tensor<Float32, [?, ?]> = some_function();
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
val z = zeros([3, 4]);           // Tensor<Float32, [3, 4]>
val o = ones([256]);             // Tensor<Float32, [256]>

// Random initialization
val r = randn([128, 64]);       // normal distribution
val u = rand([10, 10]);         // uniform [0, 1)

// From data
val t = Tensor.from_vec([1.0, 2.0, 3.0, 4.0], [2, 2]);

// Range
val seq = arange(0, 10);        // [0, 1, 2, ..., 9]

// Identity matrix
val eye = Tensor.eye(4);       // 4×4 identity

// From file
val data = load_data("weights.npy");
```

### Dtype Selection

```axon
val f16: Tensor<Float16, [1024]> = zeros([1024]);    // half precision
val f32: Tensor<Float32, [1024]> = zeros([1024]);    // single precision
val f64: Tensor<Float64, [1024]> = zeros([1024]);    // double precision
val i32: Tensor<Int32, [10]> = arange(0, 10);        // integer tensor
```

---

## Shape Operations

### Reshape

Change the shape without changing the data:

```axon
val a: Tensor<Float32, [2, 6]> = randn([2, 6]);
val b = a.reshape([3, 4]);      // Tensor<Float32, [3, 4]>
val c = a.reshape([12]);        // Tensor<Float32, [12]>
// val d = a.reshape([5, 5]);   // ERROR[E3002]: cannot reshape [2,6] (12 elements) to [5,5] (25 elements)
```

### Transpose

```axon
val m: Tensor<Float32, [3, 4]> = randn([3, 4]);
val mt = m.transpose();         // Tensor<Float32, [4, 3]>

// For higher-rank tensors, specify axes
val t: Tensor<Float32, [2, 3, 4]> = randn([2, 3, 4]);
val tp = t.permute([0, 2, 1]);  // Tensor<Float32, [2, 4, 3]>
```

### Squeeze and Unsqueeze

```axon
val a: Tensor<Float32, [1, 3, 1, 4]> = randn([1, 3, 1, 4]);
val b = a.squeeze();            // Tensor<Float32, [3, 4]>

val c: Tensor<Float32, [3, 4]> = randn([3, 4]);
val d = c.unsqueeze(0);         // Tensor<Float32, [1, 3, 4]>
```

### Concatenation and Stacking

```axon
val a: Tensor<Float32, [2, 3]> = randn([2, 3]);
val b: Tensor<Float32, [2, 3]> = randn([2, 3]);

val cat = Tensor.cat([a, b], 0);    // Tensor<Float32, [4, 3]>
val stk = Tensor.stack([a, b], 0);  // Tensor<Float32, [2, 2, 3]>
```

### Slicing

```axon
val t: Tensor<Float32, [10, 20]> = randn([10, 20]);
val row = t[0];                  // Tensor<Float32, [20]>
val sub = t[2..5];               // Tensor<Float32, [3, 20]>
```

---

## Element-Wise Operations

Standard arithmetic operators work element-wise on tensors:

```axon
val a = randn([3, 4]);
val b = randn([3, 4]);

val sum  = a + b;     // element-wise addition
val diff = a - b;     // element-wise subtraction
val prod = a * b;     // element-wise multiplication (Hadamard)
val quot = a / b;     // element-wise division

// Scalar broadcasting
val scaled = a * 2.0;
val shifted = a + 1.0;
```

### Math Functions

```axon
val x = randn([100]);

val s  = x.sin();
val c  = x.cos();
val e  = x.exp();
val l  = x.log();
val sq = x.sqrt();
val ab = x.abs();
val cl = x.clamp(-1.0, 1.0);
```

### Activation Functions

```axon
val h = relu(x);
val g = gelu(x);
val s = sigmoid(x);
val t = tanh(x);
val p = softmax(logits, dim: 1);
```

---

## Reduction Operations

Reduce tensors along axes:

```axon
val t: Tensor<Float32, [4, 5]> = randn([4, 5]);

val total = t.sum();              // scalar
val row_sum = t.sum(dim: 1);     // Tensor<Float32, [4]>
val col_mean = t.mean(dim: 0);   // Tensor<Float32, [5]>
val max_val = t.max();            // scalar
val min_idx = t.argmin(dim: 1);  // Tensor<Int64, [4]>
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
val A: Tensor<Float32, [3, 4]> = randn([3, 4]);
val B: Tensor<Float32, [4, 5]> = randn([4, 5]);
val C = A @ B;    // Tensor<Float32, [3, 5]>

// Inner dimensions must match
val D: Tensor<Float32, [4, 6]> = randn([4, 6]);
// val E = A @ D;   // ERROR[E3001]: matmul shape mismatch — inner dims 4 ≠ 4... wait
//                   // actually [3,4] @ [4,6] works. Let's show a real error:
val F: Tensor<Float32, [5, 6]> = randn([5, 6]);
// val G = A @ F;   // ERROR[E3001]: matmul requires inner dims to match: 4 ≠ 5
```

### Batch Matrix Multiplication

```axon
val batch_a: Tensor<Float32, [?, 8, 64]> = randn([32, 8, 64]);
val batch_b: Tensor<Float32, [?, 64, 32]> = randn([32, 64, 32]);
val batch_c = batch_a @ batch_b;   // Tensor<Float32, [?, 8, 32]>
```

### Other Linear Algebra Operations

```axon
val M = randn([4, 4]);

val d = M.det();              // determinant
val inv = M.inv();            // inverse
val (Q, R) = M.qr();         // QR decomposition
val (U, S, V) = M.svd();     // singular value decomposition
val eig = M.eigenvalues();   // eigenvalues
val tr = M.trace();           // trace
val dp = a.dot(b);            // dot product (1D tensors)
```

---

## Device Transfer

Tensors can be moved between CPU and GPU:

```axon
val cpu_tensor = randn([1024, 1024]);

// Move to GPU
val gpu_tensor = cpu_tensor.to_gpu();

// Compute on GPU
val result = gpu_tensor @ gpu_tensor;

// Move back to CPU for I/O
val cpu_result = result.to_cpu();
println("{}", cpu_result);
```

See [GPU Programming](gpu-programming.md) for details.

---

## Compile-Time Shape Checking

Axon's shape checker catches errors at compile time:

```axon
// ✓ Shapes match
val a: Tensor<Float32, [3, 4]> = randn([3, 4]);
val b: Tensor<Float32, [4, 5]> = randn([4, 5]);
val c = a @ b;    // OK: [3,4] @ [4,5] → [3,5]

// ✗ Shape mismatch
val d: Tensor<Float32, [3, 4]> = randn([3, 4]);
val e: Tensor<Float32, [5, 6]> = randn([5, 6]);
// val f = d @ e;   // ERROR[E3001]: matmul inner dim mismatch: 4 ≠ 5

// ✗ Invalid reshape
val g: Tensor<Float32, [2, 3]> = randn([2, 3]);
// val h = g.reshape([2, 2]);  // ERROR[E3002]: element count mismatch: 6 ≠ 4

// ✗ Element-wise shape mismatch
val i: Tensor<Float32, [3, 4]> = randn([3, 4]);
val j: Tensor<Float32, [3, 5]> = randn([3, 5]);
// val k = i + j;   // ERROR[E3003]: broadcast incompatible shapes [3,4] and [3,5]
```

### Dynamic Shapes

When dimensions are dynamic (`?`), shape checks happen at runtime:

```axon
fn process(input: Tensor<Float32, [?, 784]>): Tensor<Float32, [?, 10]> {
    val w = randn([784, 10]);
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
