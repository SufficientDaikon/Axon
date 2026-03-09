# Tutorial 05: Structs and Enums

Axon supports user-defined data types through **structs** and **enums**, providing
type-safe data modeling for ML pipelines and systems programming.

## Structs

Structs group related data together with named fields:

```axon
struct Point {
    x: Float64,
    y: Float64,
}

fn distance(a: Point, b: Point) -> Float64 {
    let dx = a.x - b.x;
    let dy = a.y - b.y;
    return (dx * dx + dy * dy).sqrt();
}

fn main() {
    let origin = Point { x: 0.0, y: 0.0 };
    let target = Point { x: 3.0, y: 4.0 };
    println(distance(origin, target));  // 5.0
}
```

### Struct Methods

Attach functions to structs using `impl` blocks:

```axon
struct ModelConfig {
    learning_rate: Float64,
    batch_size: Int32,
    epochs: Int32,
}

impl ModelConfig {
    fn default() -> ModelConfig {
        return ModelConfig {
            learning_rate: 0.001,
            batch_size: 32,
            epochs: 10,
        };
    }

    fn with_lr(self, lr: Float64) -> ModelConfig {
        return ModelConfig {
            learning_rate: lr,
            batch_size: self.batch_size,
            epochs: self.epochs,
        };
    }
}
```

### Structs with Tensors

Structs are ideal for encapsulating model parameters:

```axon
struct LinearLayer {
    weights: Tensor<Float32, [_, _]>,
    bias: Tensor<Float32, [_]>,
}

impl LinearLayer {
    fn forward(self, input: Tensor<Float32, [_, _]>) -> Tensor<Float32, [_, _]> {
        return input @ self.weights + self.bias;
    }
}
```

## Enums

Enums define types that can be one of several variants:

```axon
enum Activation {
    ReLU,
    Sigmoid,
    Tanh,
    LeakyReLU(Float64),  // variant with data
}

fn apply_activation(x: Float64, act: Activation) -> Float64 {
    match act {
        Activation::ReLU => if x > 0.0 { x } else { 0.0 },
        Activation::Sigmoid => 1.0 / (1.0 + (-x).exp()),
        Activation::Tanh => x.tanh(),
        Activation::LeakyReLU(alpha) => if x > 0.0 { x } else { alpha * x },
    }
}
```

### Pattern Matching

Use `match` for exhaustive enum handling — the compiler verifies all variants are covered:

```axon
enum Device {
    CPU,
    GPU(Int32),  // GPU with device index
}

fn device_name(d: Device) -> String {
    match d {
        Device::CPU => "cpu",
        Device::GPU(idx) => format("cuda:{}", idx),
    }
}
```

## Ownership and Structs

Axon's ownership rules apply to struct fields. When a struct goes out of scope,
its owned fields are dropped automatically:

```axon
struct DataBatch {
    images: Tensor<Float32, [_, 3, 224, 224]>,
    labels: Tensor<Int64, [_]>,
}

fn process_batch(batch: DataBatch) {
    // `batch` is moved here — caller can no longer use it
    let predictions = model.forward(batch.images);
    let loss = cross_entropy(predictions, batch.labels);
}
```

Use references (`&`) to borrow without transferring ownership:

```axon
fn inspect_batch(batch: &DataBatch) {
    println(batch.images.shape());
    println(batch.labels.shape());
    // batch is borrowed — caller retains ownership
}
```

## Next Steps

- [Tutorial 06: Error Handling](06-error-handling.md) — Result and Option types
- [Tutorial 04: Transformer](04-transformer.md) — Building a full model with structs
