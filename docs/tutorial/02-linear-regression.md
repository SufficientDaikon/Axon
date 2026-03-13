# Tutorial 2: Linear Regression

Build a simple linear regression model from scratch using tensors and autograd.
This tutorial demonstrates Axon's automatic differentiation engine.

**Prerequisites**: [Tutorial 1: Hello, Tensor!](01-hello-tensor.md)

---

## The Problem

We'll fit a line **y = wx + b** to synthetic data using gradient descent.

---

## Step 1: Generate Synthetic Data

```axon
fn generate_data(n: Int32): (Tensor<Float32, [?, 1]>, Tensor<Float32, [?, 1]>) {
    // True parameters: y = 3.5x + 2.0 + noise
    val x = randn([n, 1]);
    val noise = randn([n, 1]) * 0.3;
    val y = x * 3.5 + 2.0 + noise;
    (x, y)
}
```

---

## Step 2: Define the Model

Linear regression is a single linear transformation:

```axon
model LinearRegression {
    weight: Tensor<Float32, [1, 1]>,
    bias: Tensor<Float32, [1]>,
}

extend LinearRegression {
    fn new(): LinearRegression {
        LinearRegression {
            weight: randn([1, 1]),
            bias: zeros([1]),
        }
    }

    fn forward(&self, x: Tensor<Float32, [?, 1]>): Tensor<Float32, [?, 1]> {
        x @ self.weight + self.bias
    }
}
```

---

## Step 3: Define the Loss Function

Mean Squared Error — the standard loss for regression:

```axon
fn mse_loss(
    predictions: Tensor<Float32, [?, 1]>,
    targets: Tensor<Float32, [?, 1]>,
): Tensor<Float32, []> {
    val diff = predictions - targets;
    (diff * diff).mean()
}
```

---

## Step 4: Training with Autograd

Now we use Axon's autograd to compute gradients and update parameters:

```axon
use std.autograd.GradTensor;
use std.optim.SGD;

fn train() {
    // Generate training data
    val (x_train, y_train) = generate_data(200);

    // Initialize model
    var net = LinearRegression.new();

    // Optimizer: SGD with learning rate 0.01
    var optimizer = SGD.new(
        [&net.weight, &net.bias],
        lr: 0.01,
    );

    // Training loop
    for epoch in 0..100 {
        // Forward pass
        val predictions = net.forward(x_train);

        // Compute loss
        val loss = mse_loss(predictions, y_train);

        // Backward pass — compute gradients
        loss.backward();

        // Update parameters
        optimizer.step();
        optimizer.zero_grad();

        if epoch % 10 == 0 {
            println("Epoch {}: loss = {:.4}", epoch, loss.item());
        }
    }

    // Print learned parameters
    println("Learned weight: {:.4} (true: 3.5)", net.weight.item());
    println("Learned bias:   {:.4} (true: 2.0)", net.bias.item());
}
```

---

## Step 5: Evaluate the Model

```axon
fn evaluate(net: &LinearRegression) {
    // Generate test data
    val (x_test, y_test) = generate_data(50);

    // Predict
    val predictions = net.forward(x_test);

    // Compute test loss
    val test_loss = mse_loss(predictions, y_test);
    println("Test MSE: {:.4}", test_loss.item());

    // Print a few predictions
    println("\nSample predictions:");
    println("  x       | predicted | actual");
    println("  --------|-----------|-------");
    for i in 0..5 {
        println("  {:.4}  | {:.4}    | {:.4}",
            x_test[i].item(),
            predictions[i].item(),
            y_test[i].item()
        );
    }
}
```

---

## Step 6: Full Program

```axon
use std.autograd.GradTensor;
use std.optim.SGD;

model LinearRegression {
    weight: Tensor<Float32, [1, 1]>,
    bias: Tensor<Float32, [1]>,
}

extend LinearRegression {
    fn new(): LinearRegression {
        LinearRegression {
            weight: randn([1, 1]),
            bias: zeros([1]),
        }
    }

    fn forward(&self, x: Tensor<Float32, [?, 1]>): Tensor<Float32, [?, 1]> {
        x @ self.weight + self.bias
    }
}

fn mse_loss(
    predictions: Tensor<Float32, [?, 1]>,
    targets: Tensor<Float32, [?, 1]>,
): Tensor<Float32, []> {
    val diff = predictions - targets;
    (diff * diff).mean()
}

fn main() {
    println("=== Linear Regression in Axon ===\n");

    // Data
    val (x_train, y_train) = generate_data(200);

    // Model
    var net = LinearRegression.new();
    var optimizer = SGD.new(
        [&net.weight, &net.bias],
        lr: 0.01,
    );

    // Train
    for epoch in 0..200 {
        val pred = net.forward(x_train);
        val loss = mse_loss(pred, y_train);
        loss.backward();
        optimizer.step();
        optimizer.zero_grad();

        if epoch % 50 == 0 {
            println("Epoch {:>3}: loss = {:.6}", epoch, loss.item());
        }
    }

    println("\nFinal parameters:");
    println("  weight = {:.4} (true: 3.5)", net.weight.item());
    println("  bias   = {:.4} (true: 2.0)", net.bias.item());

    // Evaluate
    val (x_test, y_test) = generate_data(50);
    val test_pred = net.forward(x_test);
    val test_loss = mse_loss(test_pred, y_test);
    println("\nTest MSE: {:.6}", test_loss.item());
}

fn generate_data(n: Int32): (Tensor<Float32, [?, 1]>, Tensor<Float32, [?, 1]>) {
    val x = randn([n, 1]);
    val noise = randn([n, 1]) * 0.3;
    val y = x * 3.5 + 2.0 + noise;
    (x, y)
}
```

Run:

```bash
axonc build linear_reg.axon -O 2 -o linear_reg
./linear_reg
# === Linear Regression in Axon ===
#
# Epoch   0: loss = 14.832901
# Epoch  50: loss = 0.129384
# Epoch 100: loss = 0.091203
# Epoch 150: loss = 0.089847
#
# Final parameters:
#   weight = 3.4821 (true: 3.5)
#   bias   = 1.9934 (true: 2.0)
#
# Test MSE: 0.092145
```

---

## Key Concepts Covered

| Concept          | Axon Feature                   |
| ---------------- | ------------------------------ |
| Model definition | `model` with tensor fields     |
| Forward pass     | `@` operator, element-wise ops |
| Loss function    | Tensor reductions (`.mean()`)  |
| Backpropagation  | `loss.backward()`              |
| Parameter update | `optimizer.step()`             |
| Gradient reset   | `optimizer.zero_grad()`        |

---

## Next Steps

- [Tutorial 3: MNIST Classifier](03-mnist-classifier.md) — build a CNN
- [Autograd Guide](../guide/tensors.md) — how autograd works under the hood
