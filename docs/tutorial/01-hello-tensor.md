# Tutorial 1: Hello, Tensor!

In this tutorial you'll create, manipulate, and inspect tensors — the
fundamental data type in Axon.

**Prerequisites**: Axon installed ([Getting Started](../guide/getting-started.md))

---

## Step 1: Create a Project

```bash
axonc pkg new hello_tensor
cd hello_tensor
```

---

## Step 2: Your First Tensor

Open `src/main.axon` and replace its contents:

```axon
fn main() {
    // Create a 1D tensor from values
    val numbers = Tensor.from_vec([1.0, 2.0, 3.0, 4.0, 5.0], [5]);
    println("Numbers: {}", numbers);
    println("Shape:   {}", numbers.shape);
    println("Sum:     {}", numbers.sum());
    println("Mean:    {}", numbers.mean());
}
```

Run it:

```bash
axonc pkg run
# Numbers: [1.0, 2.0, 3.0, 4.0, 5.0]
# Shape:   [5]
# Sum:     15.0
# Mean:    3.0
```

---

## Step 3: Creating Tensors

Axon offers several tensor constructors:

```axon
fn main() {
    // Zeros and ones
    val z: Tensor<Float32, [2, 3]> = zeros([2, 3]);
    println("Zeros:\n{}", z);

    val o: Tensor<Float32, [3]> = ones([3]);
    println("Ones: {}", o);

    // Random tensors
    val r = randn([2, 2]);     // normal distribution
    println("Random:\n{}", r);

    // Range
    val seq = arange(0, 5);
    println("Range: {}", seq);   // [0, 1, 2, 3, 4]

    // Identity matrix
    val eye = Tensor.eye(3);
    println("Identity:\n{}", eye);
}
```

---

## Step 4: Arithmetic Operations

Tensors support element-wise arithmetic:

```axon
fn main() {
    val a = Tensor.from_vec([1.0, 2.0, 3.0], [3]);
    val b = Tensor.from_vec([4.0, 5.0, 6.0], [3]);

    println("a + b = {}", a + b);    // [5.0, 7.0, 9.0]
    println("a * b = {}", a * b);    // [4.0, 10.0, 18.0]
    println("a * 2 = {}", a * 2.0);  // [2.0, 4.0, 6.0]

    // Math functions
    val x = Tensor.from_vec([0.0, 1.5708, 3.1416], [3]);
    println("sin(x) = {}", x.sin());
    println("exp(x) = {}", x.exp());
}
```

---

## Step 5: Matrix Multiplication

The `@` operator performs matrix multiplication:

```axon
fn main() {
    val A: Tensor<Float32, [2, 3]> = Tensor.from_vec(
        [1.0, 2.0, 3.0,
         4.0, 5.0, 6.0], [2, 3]
    );

    val B: Tensor<Float32, [3, 2]> = Tensor.from_vec(
        [7.0,  8.0,
         9.0,  10.0,
         11.0, 12.0], [3, 2]
    );

    val C = A @ B;   // [2, 3] @ [3, 2] → [2, 2]
    println("A @ B =\n{}", C);
    println("Shape: {}", C.shape);
    // A @ B =
    // [[58.0, 64.0],
    //  [139.0, 154.0]]
}
```

---

## Step 6: Reshaping and Transposing

```axon
fn main() {
    val t: Tensor<Float32, [2, 6]> = arange(0, 12).reshape([2, 6]);
    println("Original [2, 6]:\n{}", t);

    // Reshape
    val r = t.reshape([3, 4]);
    println("Reshaped [3, 4]:\n{}", r);

    val flat = t.reshape([12]);
    println("Flat: {}", flat);

    // Transpose
    val m: Tensor<Float32, [2, 3]> = Tensor.from_vec(
        [1.0, 2.0, 3.0,
         4.0, 5.0, 6.0], [2, 3]
    );
    val mt = m.transpose();
    println("Transposed [3, 2]:\n{}", mt);
}
```

---

## Step 7: Reductions

```axon
fn main() {
    val data: Tensor<Float32, [3, 4]> = Tensor.from_vec(
        [1.0, 2.0, 3.0, 4.0,
         5.0, 6.0, 7.0, 8.0,
         9.0, 10.0, 11.0, 12.0], [3, 4]
    );

    println("Sum (all):   {}", data.sum());          // 78.0
    println("Mean (all):  {}", data.mean());         // 6.5
    println("Max (all):   {}", data.max());          // 12.0
    println("Sum (dim 0): {}", data.sum(dim: 0));    // [15.0, 18.0, 21.0, 24.0]
    println("Sum (dim 1): {}", data.sum(dim: 1));    // [10.0, 26.0, 42.0]
}
```

---

## Step 8: Putting It All Together

A small program that normalizes a dataset:

```axon
fn normalize(data: Tensor<Float32, [?, ?]>): Tensor<Float32, [?, ?]> {
    val mean = data.mean(dim: 0);
    val std = data.std(dim: 0);
    (data - mean) / std
}

fn main() {
    // Simulate a dataset: 100 samples, 4 features
    val dataset = randn([100, 4]);

    println("Before normalization:");
    println("  Mean: {}", dataset.mean(dim: 0));

    val normed = normalize(dataset);

    println("After normalization:");
    println("  Mean: {}", normed.mean(dim: 0));   // ≈ [0, 0, 0, 0]
    println("  Std:  {}", normed.std(dim: 0));    // ≈ [1, 1, 1, 1]
}
```

---

## What You Learned

- Creating tensors with `from_vec`, `zeros`, `ones`, `randn`, `arange`
- Element-wise operations (`+`, `-`, `*`, `/`)
- Matrix multiplication with `@`
- Reshaping, transposing, and slicing
- Reduction operations (`sum`, `mean`, `max`)
- Compile-time shape checking

---

## Next Steps

- [Tutorial 2: Linear Regression](02-linear-regression.md) — build a model from scratch
- [Tensor Guide](../guide/tensors.md) — full tensor reference
