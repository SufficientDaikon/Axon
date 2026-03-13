# Migrating from PyTorch to Axon

A side-by-side guide for PyTorch developers. Axon's ML framework is heavily
inspired by PyTorch's API, with the added benefits of compile-time shape
checking, ownership-based memory safety, and native GPU compilation.

---

## Tensor Creation

```python
# PyTorch
import torch

x = torch.zeros(3, 4)
y = torch.ones(5)
z = torch.randn(128, 256)
a = torch.tensor([1.0, 2.0, 3.0])
e = torch.eye(4)
r = torch.arange(0, 10)
```

```axon
// Axon
val x = zeros([3, 4]);
val y = ones([5]);
val z = randn([128, 256]);
val a = Tensor.from_vec([1.0, 2.0, 3.0], [3]);
val e = Tensor.eye(4);
val r = arange(0, 10);
```

Key difference: Axon tensors carry their shape in the type system:
`Tensor<Float32, [3, 4]>` vs PyTorch's dynamic `torch.Tensor`.

---

## Tensor Operations

```python
# PyTorch
c = a + b
c = a * b
c = a @ b          # matmul
c = torch.matmul(a, b)
m = x.mean(dim=0)
s = x.sum(dim=1)
r = x.reshape(3, 4)
t = x.T             # transpose
```

```axon
// Axon
val c = a + b;
val c = a * b;
val c = a @ b;          // matmul (same!)
// no functional form needed
val m = x.mean(dim: 0);
val s = x.sum(dim: 1);
val r = x.reshape([3, 4]);
val t = x.transpose();
```

---

## Model Definition

```python
# PyTorch
import torch.nn as nn

class MyModel(nn.Module):
    def __init__(self):
        super().__init__()
        self.fc1 = nn.Linear(784, 256)
        self.fc2 = nn.Linear(256, 128)
        self.fc3 = nn.Linear(128, 10)

    def forward(self, x):
        x = torch.relu(self.fc1(x))
        x = torch.relu(self.fc2(x))
        return self.fc3(x)

model = MyModel()
```

```axon
// Axon
use std.nn.{Linear, Module};

model MyModel {
    fc1: Linear<784, 256>,
    fc2: Linear<256, 128>,
    fc3: Linear<128, 10>,
}

extend MyModel {
    fn new(): MyModel {
        MyModel {
            fc1: Linear.new(),
            fc2: Linear.new(),
            fc3: Linear.new(),
        }
    }
}

extend Module for MyModel {
    fn forward(&self, x: Tensor<Float32, [?, 784]>): Tensor<Float32, [?, 10]> {
        val h = relu(self.fc1.forward(x));
        val h = relu(self.fc2.forward(h));
        self.fc3.forward(h)
    }
}

val model = MyModel.new();
```

Key differences:

- `model` + `extend Module` instead of `class(nn.Module)`
- Linear layer sizes are part of the type: `Linear<784, 256>`
- Input/output shapes are checked at compile time
- No `super().__init__()` boilerplate

---

## Training Loop

```python
# PyTorch
optimizer = torch.optim.Adam(model.parameters(), lr=0.001)
criterion = nn.CrossEntropyLoss()

for epoch in range(10):
    for inputs, targets in dataloader:
        optimizer.zero_grad()
        outputs = model(inputs)
        loss = criterion(outputs, targets)
        loss.backward()
        optimizer.step()
    print(f"Epoch {epoch}: loss = {loss.item():.4f}")
```

```axon
// Axon
use std.optim.Adam;
use std.loss.cross_entropy;

var optimizer = Adam.new(model.parameters(), lr: 0.001);

for epoch in 0..10 {
    for batch in &dataloader {
        val (inputs, targets) = batch;
        val outputs = model.forward(inputs);
        val loss = cross_entropy(outputs, targets);
        loss.backward();
        optimizer.step();
        optimizer.zero_grad();
    }
    println("Epoch {}: loss = {:.4}", epoch, loss.item());
}
```

Almost identical! The main differences:

- `model.forward(x)` instead of `model(x)`
- `cross_entropy(outputs, targets)` is a function, not a class
- `optimizer.zero_grad()` typically called after `step()` (same effect)
- Borrow semantics: `&dataloader` to iterate without consuming

---

## CNN Layers

```python
# PyTorch
self.conv1 = nn.Conv2d(1, 32, kernel_size=3, padding=1)
self.pool = nn.MaxPool2d(2)
self.bn = nn.BatchNorm2d(32)
self.dropout = nn.Dropout(0.5)
```

```axon
// Axon
self.conv1 = Conv2d.new(in_channels: 1, out_channels: 32, kernel_size: 3, padding: 1);
self.pool = MaxPool2d.new(kernel_size: 2, stride: 2);
self.bn = BatchNorm.new(32);
self.dropout = Dropout.new(rate: 0.5);
```

---

## RNN / Transformer Layers

```python
# PyTorch
lstm = nn.LSTM(input_size=128, hidden_size=256, num_layers=2, batch_first=True)
attention = nn.MultiheadAttention(embed_dim=512, num_heads=8)
encoder = nn.TransformerEncoder(
    nn.TransformerEncoderLayer(d_model=512, nhead=8),
    num_layers=6
)
```

```axon
// Axon
val lstm = LSTM.new(input_size: 128, hidden_size: 256, num_layers: 2);
val attention = MultiHeadAttention.new(d_model: 512, num_heads: 8);
val encoder = TransformerEncoder.new(
    d_model: 512,
    num_heads: 8,
    num_layers: 6,
    d_ff: 2048,
    dropout: 0.1,
);
```

---

## GPU / Device Management

```python
# PyTorch
device = torch.device("cuda" if torch.cuda.is_available() else "cpu")
model = model.to(device)
x = x.to(device)

# Multi-GPU
model = nn.DataParallel(model)
```

```axon
// Axon
use std.device.{cuda, cpu};

val model = MyModel.new().to_gpu();
val x = x.to_gpu();

// Or with explicit device
val dev = cuda(0);
val x = x.to_device(dev);
```

Axon difference: Device transfer is a **move** (ownership transfer), not a copy.

---

## Optimizers

| PyTorch                                                 | Axon                                                 |
| ------------------------------------------------------- | ---------------------------------------------------- |
| `torch.optim.SGD(params, lr=0.01)`                      | `SGD.new(params, lr: 0.01)`                         |
| `torch.optim.Adam(params, lr=0.001)`                    | `Adam.new(params, lr: 0.001)`                       |
| `torch.optim.AdamW(params, lr=1e-4, weight_decay=0.01)` | `AdamW.new(params, lr: 0.0001, weight_decay: 0.01)` |

---

## Loss Functions

| PyTorch                                 | Axon                            |
| --------------------------------------- | ------------------------------- |
| `nn.CrossEntropyLoss()(output, target)` | `cross_entropy(output, target)` |
| `nn.MSELoss()(output, target)`          | `mse_loss(output, target)`      |
| `nn.BCELoss()(output, target)`          | `bce_loss(output, target)`      |
| `nn.L1Loss()(output, target)`           | `l1_loss(output, target)`       |

Axon uses functions instead of loss classes — simpler and more direct.

---

## Autograd / Gradients

```python
# PyTorch
x = torch.randn(3, requires_grad=True)
y = x * 2 + 1
y.sum().backward()
print(x.grad)

with torch.no_grad():
    prediction = model(x)
```

```axon
// Axon
use std.autograd.{GradTensor, no_grad};

val x = GradTensor.new(randn([3]));
val y = x * 2.0 + 1.0;
y.sum().backward();
println("{}", x.grad());

no_grad(|| {
    val prediction = model.forward(x);
});
```

---

## Data Loading

```python
# PyTorch
from torch.utils.data import DataLoader, TensorDataset

dataset = TensorDataset(x_train, y_train)
loader = DataLoader(dataset, batch_size=64, shuffle=True)
```

```axon
// Axon
use std.data.DataLoader;

val loader = DataLoader.new(x_train, y_train)
    .batch_size(64)
    .shuffle(true);
```

---

## Model Saving / Loading

```python
# PyTorch
torch.save(model.state_dict(), "model.pt")
model.load_state_dict(torch.load("model.pt"))

# ONNX export
torch.onnx.export(model, dummy_input, "model.onnx")
```

```axon
// Axon
use std.export.{save, load, export_onnx};

save(&model, "model.axon");
val model: MyModel = load("model.axon");

// ONNX export
val dummy_input = randn([1, 784]);
export_onnx(&model, dummy_input, "model.onnx");
```

---

## What Axon Adds Over PyTorch

| Feature         | PyTorch           | Axon                            |
| --------------- | ----------------- | ------------------------------- |
| Shape checking  | Runtime errors    | **Compile-time errors**         |
| Memory safety   | Manual management | **Ownership system**            |
| Type safety     | Dynamic typing    | **Static types with inference** |
| GPU compilation | Python + CUDA C   | **Native GPU via MLIR**         |
| Performance     | Python overhead   | **Native binary, no GIL**       |
| Package manager | pip + setup.py    | **Built-in `axonc pkg`**        |
| Formatting      | black (separate)  | **Built-in `axonc fmt`**        |

---

## Quick Translation Table

| PyTorch                 | Axon                         |
| ----------------------- | ---------------------------- |
| `import torch`          | (built-in, no import needed) |
| `import torch.nn as nn` | `use std.nn.*`               |
| `model(x)`              | `model.forward(x)`           |
| `loss.item()`           | `loss.item()` (same!)        |
| `.backward()`           | `.backward()` (same!)        |
| `.to("cuda")`           | `.to_gpu()`                  |
| `torch.no_grad()`       | `no_grad(\|\| { ... })`      |
| `model.eval()`          | `model.eval()` (same!)       |
| `model.train()`         | `model.train()` (same!)      |

---

## See Also

- [Tensor Guide](../guide/tensors.md) — Axon tensor operations
- [GPU Programming](../guide/gpu-programming.md) — native GPU support
- [Tutorial: MNIST](../tutorial/03-mnist-classifier.md) — full training example
- [Migration from Python](from-python.md) — general Python → Axon guide
