# Tutorial 3: MNIST Classifier

Build a convolutional neural network to classify handwritten digits from the
MNIST dataset using Axon's `std::nn` module.

**Prerequisites**: [Tutorial 2: Linear Regression](02-linear-regression.md)

---

## Overview

We'll build a CNN with:

- Two convolutional layers with ReLU and max pooling
- Two fully connected layers
- Softmax output for 10 digit classes

---

## Step 1: Data Loading

```axon
use std::data::DataLoader;
use std::transforms::{normalize, to_tensor};

fn load_mnist() -> (DataLoader, DataLoader) {
    let train_loader = DataLoader::from_csv("data/mnist_train.csv")
        .batch_size(64)
        .shuffle(true)
        .transform(|img| {
            let tensor = to_tensor(img, [1, 28, 28]);
            normalize(tensor, mean: [0.1307], std: [0.3081])
        });

    let test_loader = DataLoader::from_csv("data/mnist_test.csv")
        .batch_size(256)
        .shuffle(false)
        .transform(|img| {
            let tensor = to_tensor(img, [1, 28, 28]);
            normalize(tensor, mean: [0.1307], std: [0.3081])
        });

    (train_loader, test_loader)
}
```

---

## Step 2: Define the Model

```axon
use std::nn::{Conv2d, Linear, MaxPool2d, Module, Sequential};

struct MNISTNet {
    conv1: Conv2d,
    conv2: Conv2d,
    pool: MaxPool2d,
    fc1: Linear<9216, 128>,
    fc2: Linear<128, 10>,
}

impl MNISTNet {
    fn new() -> MNISTNet {
        MNISTNet {
            conv1: Conv2d::new(in_channels: 1, out_channels: 32, kernel_size: 3, padding: 1),
            conv2: Conv2d::new(in_channels: 32, out_channels: 64, kernel_size: 3, padding: 1),
            pool: MaxPool2d::new(kernel_size: 2, stride: 2),
            fc1: Linear::new(),
            fc2: Linear::new(),
        }
    }
}

impl Module for MNISTNet {
    fn forward(&self, x: Tensor<Float32, [?, 1, 28, 28]>) -> Tensor<Float32, [?, 10]> {
        // Conv block 1: [?, 1, 28, 28] → [?, 32, 14, 14]
        let h = self.conv1.forward(x);
        let h = relu(h);
        let h = self.pool.forward(h);

        // Conv block 2: [?, 32, 14, 14] → [?, 64, 7, 7]
        let h = self.conv2.forward(h);
        let h = relu(h);
        let h = self.pool.forward(h);

        // Flatten: [?, 64, 7, 7] → [?, 3136]
        let batch_size = h.shape[0];
        let h = h.reshape([batch_size, 3136]);

        // Fully connected layers
        let h = relu(self.fc1.forward(h));
        self.fc2.forward(h)
    }
}
```

---

## Step 3: Training Loop

```axon
use std::optim::Adam;
use std::loss::cross_entropy;
use std::metrics::accuracy;

fn train_epoch(
    model: &mut MNISTNet,
    data: &DataLoader,
    optimizer: &mut Adam,
) -> (Float32, Float32) {
    let mut total_loss = 0.0;
    let mut correct = 0;
    let mut total = 0;

    for batch in data {
        let (images, labels) = batch;

        // Forward
        let logits = model.forward(images);
        let loss = cross_entropy(logits, labels);

        // Track metrics
        total_loss += loss.item();
        let predicted = logits.argmax(dim: 1);
        correct += (predicted == labels).sum().item() as Int32;
        total += labels.shape[0];

        // Backward
        loss.backward();
        optimizer.step();
        optimizer.zero_grad();
    }

    let avg_loss = total_loss / data.num_batches() as Float32;
    let acc = correct as Float32 / total as Float32;
    (avg_loss, acc)
}
```

---

## Step 4: Evaluation

```axon
fn evaluate(model: &MNISTNet, data: &DataLoader) -> (Float32, Float32) {
    let mut total_loss = 0.0;
    let mut correct = 0;
    let mut total = 0;

    for batch in data {
        let (images, labels) = batch;
        let logits = model.forward(images);
        let loss = cross_entropy(logits, labels);

        total_loss += loss.item();
        let predicted = logits.argmax(dim: 1);
        correct += (predicted == labels).sum().item() as Int32;
        total += labels.shape[0];
    }

    let avg_loss = total_loss / data.num_batches() as Float32;
    let acc = correct as Float32 / total as Float32;
    (avg_loss, acc)
}
```

---

## Step 5: Full Training Program

```axon
use std::nn::{Conv2d, Linear, MaxPool2d, Module};
use std::optim::Adam;
use std::loss::cross_entropy;
use std::data::DataLoader;
use std::transforms::{normalize, to_tensor};

fn main() {
    println("=== MNIST Classifier ===\n");

    // Load data
    let (train_loader, test_loader) = load_mnist();
    println("Train: {} samples", train_loader.len());
    println("Test:  {} samples\n", test_loader.len());

    // Create model and optimizer
    let mut model = MNISTNet::new();
    let mut optimizer = Adam::new(
        model.parameters(),
        lr: 0.001,
    );

    // Training
    let epochs = 10;
    for epoch in 0..epochs {
        let (train_loss, train_acc) = train_epoch(&mut model, &train_loader, &mut optimizer);
        let (test_loss, test_acc) = evaluate(&model, &test_loader);

        println("Epoch {:>2}/{} | Train Loss: {:.4} Acc: {:.2}% | Test Loss: {:.4} Acc: {:.2}%",
            epoch + 1, epochs,
            train_loss, train_acc * 100.0,
            test_loss, test_acc * 100.0,
        );
    }

    // Final evaluation
    let (_, final_acc) = evaluate(&model, &test_loader);
    println("\nFinal test accuracy: {:.2}%", final_acc * 100.0);
}
```

Expected output:

```
=== MNIST Classifier ===

Train: 60000 samples
Test:  10000 samples

Epoch  1/10 | Train Loss: 0.2134 Acc: 93.41% | Test Loss: 0.0712 Acc: 97.82%
Epoch  2/10 | Train Loss: 0.0583 Acc: 98.19% | Test Loss: 0.0498 Acc: 98.41%
...
Epoch 10/10 | Train Loss: 0.0089 Acc: 99.71% | Test Loss: 0.0312 Acc: 99.12%

Final test accuracy: 99.12%
```

---

## Step 6: GPU Training (Optional)

To train on GPU, simply transfer data and model:

```axon
fn main() {
    let mut model = MNISTNet::new().to_gpu();
    let mut optimizer = Adam::new(model.parameters(), lr: 0.001);

    for epoch in 0..10 {
        for batch in &train_loader {
            let (images, labels) = batch;
            let images = images.to_gpu();
            let labels = labels.to_gpu();

            let logits = model.forward(images);
            let loss = cross_entropy(logits, labels);
            loss.backward();
            optimizer.step();
            optimizer.zero_grad();
        }
    }
}
```

Compile with GPU support:

```bash
axonc build mnist.axon --gpu cuda -O 3 -o mnist
```

---

## Step 7: Save the Model

```axon
use std::export::save;

// After training
save(&model, "mnist_model.axon");
println("Model saved!");

// Load later
use std::export::load;
let loaded_model: MNISTNet = load("mnist_model.axon");
```

---

## Key Concepts Covered

| Concept          | Axon Feature                             |
| ---------------- | ---------------------------------------- |
| CNN architecture | `Conv2d`, `MaxPool2d`, `Linear`          |
| Data loading     | `DataLoader` with transforms             |
| Training loop    | `forward` → `loss` → `backward` → `step` |
| Metrics          | `argmax`, accuracy calculation           |
| GPU training     | `.to_gpu()` + `--gpu cuda`               |
| Model saving     | `std::export::save` / `load`             |

---

## Next Steps

- [Tutorial 4: Transformer](04-transformer.md) — build a transformer encoder
- [GPU Programming Guide](../guide/gpu-programming.md) — advanced GPU patterns
