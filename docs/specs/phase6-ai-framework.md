# Phase 6: AI Framework — Implementation Specification

## 1. Overview

Phase 6 builds a **built-in AI/ML framework** directly into the Axon language ecosystem. Unlike Python where PyTorch/TensorFlow are external libraries, Axon's AI framework is a first-class citizen that leverages compile-time shape checking, the borrow checker, and native GPU codegen from earlier phases.

### Deliverables

| Module                  | Path                          | Purpose                                                        |
| ----------------------- | ----------------------------- | -------------------------------------------------------------- |
| `std::nn`               | `stdlib/nn/`                  | Neural network layers (Linear, Conv2d, RNN, Transformer, etc.) |
| `std::autograd`         | `stdlib/autograd/`            | Automatic differentiation engine                               |
| `std::optim`            | `stdlib/optim/`               | Optimizers (SGD, Adam, AdamW, etc.)                            |
| `std::loss`             | `stdlib/loss/`                | Loss functions (MSE, CrossEntropy, etc.)                       |
| `std::nn::init`         | `stdlib/nn/init.axon`         | Weight initialization strategies                               |
| `std::data::transforms` | `stdlib/data/transforms.axon` | Data augmentation and preprocessing                            |
| `std::export`           | `stdlib/export/`              | ONNX export, model serialization                               |
| `std::train`            | `stdlib/train/`               | Training loop utilities, checkpointing                         |
| `std::metrics`          | `stdlib/metrics/`             | Accuracy, precision, recall, F1, confusion matrix              |

### Dependencies

- Phase 5: `std::tensor`, `std::collections`, `std::io`, `std::device`
- Phase 4: GPU codegen for `@gpu` kernel compilation
- Phase 3: Shape checking for compile-time dimension verification

---

## 2. Autograd Engine (`std::autograd`)

### 2.1 Design Philosophy

Axon uses **define-by-run** (eager) automatic differentiation, similar to PyTorch. Every tensor operation records its computation graph; calling `.backward()` traverses the graph in reverse to compute gradients.

### 2.2 Core Types

```axon
struct GradTensor<DType, Shape> {
    data: Tensor<DType, Shape>,
    grad: Option<Tensor<DType, Shape>>,
    requires_grad: Bool,
    grad_fn: Option<GradFn>,
    // Internal: node in computation graph
}

impl<DType: Numeric, Shape> GradTensor<DType, Shape> {
    fn new(data: Tensor<DType, Shape>, requires_grad: Bool) -> GradTensor<DType, Shape>;
    fn backward(&self);                                // Compute all gradients
    fn grad(&self) -> Option<&Tensor<DType, Shape>>;   // Access gradient
    fn zero_grad(&mut self);                           // Reset gradient to zero
    fn detach(&self) -> Tensor<DType, Shape>;          // Detach from graph
    fn no_grad<F: Fn() -> T, T>(f: F) -> T;           // Run without tracking
    fn data(&self) -> &Tensor<DType, Shape>;           // Access underlying data
}
```

### 2.3 Computation Graph

```axon
// Internal representation — not user-facing
struct ComputationGraph {
    nodes: Vec<GraphNode>,
}

struct GraphNode {
    id: NodeId,
    op: GradOp,
    inputs: Vec<NodeId>,
    output_shape: Vec<Int64>,
}

enum GradOp {
    Add, Sub, Mul, Div, MatMul,
    Relu, Sigmoid, Tanh, Softmax,
    Sum, Mean, Reshape, Transpose,
    Conv2d { kernel, stride, padding },
    MaxPool2d { kernel_size, stride },
    BatchNorm { running_mean, running_var },
    Linear { weight, bias },
    Dropout { p },
    // ... one per differentiable operation
}
```

### 2.4 Backward Pass Rules

| Forward Operation   | Backward Rule (∂L/∂input)      |
| ------------------- | ------------------------------ |
| `c = a + b`         | `∂a = ∂c`, `∂b = ∂c`           |
| `c = a * b`         | `∂a = ∂c * b`, `∂b = ∂c * a`   |
| `c = a @ b`         | `∂a = ∂c @ bᵀ`, `∂b = aᵀ @ ∂c` |
| `c = relu(a)`       | `∂a = ∂c * (a > 0)`            |
| `c = sigmoid(a)`    | `∂a = ∂c * c * (1 - c)`        |
| `c = tanh(a)`       | `∂a = ∂c * (1 - c²)`           |
| `c = softmax(a)`    | Jacobian-vector product        |
| `c = sum(a)`        | `∂a = broadcast(∂c)`           |
| `c = mean(a)`       | `∂a = broadcast(∂c / n)`       |
| `c = reshape(a, s)` | `∂a = reshape(∂c, a.shape)`    |
| `c = transpose(a)`  | `∂a = transpose(∂c)`           |

### 2.5 Gradient Checkpointing

For memory-constrained training:

```axon
fn checkpoint<F: Fn(&GradTensor) -> GradTensor>(f: F, input: &GradTensor) -> GradTensor {
    // Forward: compute without storing intermediates
    // Backward: recompute forward pass, then compute gradients
}
```

---

## 3. Neural Network Layers (`std::nn`)

### 3.1 Module Trait

```axon
trait Module {
    fn forward(&self, input: &GradTensor<Float32, [?]>) -> GradTensor<Float32, [?]>;
    fn parameters(&self) -> Vec<&GradTensor<Float32, [?]>>;
    fn train(&mut self);       // Set training mode
    fn eval(&mut self);        // Set evaluation mode
    fn to_device(&mut self, device: Device);
    fn save(&self, path: &String) -> Result<(), IoError>;
    fn load(&mut self, path: &String) -> Result<(), IoError>;
}
```

### 3.2 Core Layers

```axon
// Linear (fully connected)
struct Linear<const IN: Int64, const OUT: Int64> {
    weight: GradTensor<Float32, [OUT, IN]>,
    bias: Option<GradTensor<Float32, [OUT]>>,
}
impl<const IN: Int64, const OUT: Int64> Linear<IN, OUT> {
    fn new(in_features: Int64, out_features: Int64, bias: Bool) -> Linear<IN, OUT>;
}

// Convolution
struct Conv2d<const IN_CH: Int64, const OUT_CH: Int64> {
    weight: GradTensor<Float32, [OUT_CH, IN_CH, ?, ?]>,
    bias: Option<GradTensor<Float32, [OUT_CH]>>,
    stride: (Int64, Int64),
    padding: (Int64, Int64),
}

// Batch Normalization
struct BatchNorm<const FEATURES: Int64> {
    weight: GradTensor<Float32, [FEATURES]>,
    bias: GradTensor<Float32, [FEATURES]>,
    running_mean: Tensor<Float32, [FEATURES]>,
    running_var: Tensor<Float32, [FEATURES]>,
    momentum: Float32,
    eps: Float32,
}

// Layer Normalization
struct LayerNorm {
    weight: GradTensor<Float32, [?]>,
    bias: GradTensor<Float32, [?]>,
    eps: Float32,
}

// Dropout
struct Dropout {
    p: Float32,
    training: Bool,
}

// Pooling
struct MaxPool2d { kernel_size: (Int64, Int64), stride: (Int64, Int64) }
struct AvgPool2d { kernel_size: (Int64, Int64), stride: (Int64, Int64) }
struct AdaptiveAvgPool2d { output_size: (Int64, Int64) }

// Recurrent
struct LSTM<const INPUT: Int64, const HIDDEN: Int64> { /* weights, biases */ }
struct GRU<const INPUT: Int64, const HIDDEN: Int64> { /* weights, biases */ }

// Transformer
struct MultiHeadAttention {
    embed_dim: Int64,
    num_heads: Int64,
    q_proj: Linear,
    k_proj: Linear,
    v_proj: Linear,
    out_proj: Linear,
}

struct TransformerEncoderLayer {
    self_attn: MultiHeadAttention,
    ff1: Linear,
    ff2: Linear,
    norm1: LayerNorm,
    norm2: LayerNorm,
    dropout: Dropout,
}

struct TransformerEncoder {
    layers: Vec<TransformerEncoderLayer>,
}

// Embedding
struct Embedding<const VOCAB: Int64, const DIM: Int64> {
    weight: GradTensor<Float32, [VOCAB, DIM]>,
}

// Activation modules (stateless, implement Module)
struct ReLU {}
struct GELU {}
struct SiLU {}
struct Softmax { dim: Int64 }
struct LogSoftmax { dim: Int64 }
```

### 3.3 Sequential Container

```axon
struct Sequential {
    layers: Vec<Box<dyn Module>>,
}

impl Sequential {
    fn new(layers: Vec<Box<dyn Module>>) -> Sequential;
}

impl Module for Sequential {
    fn forward(&self, input: &GradTensor) -> GradTensor {
        let mut x = input.clone();
        for layer in &self.layers {
            x = layer.forward(&x);
        }
        x
    }
}
```

### 3.4 Weight Initialization (`std::nn::init`)

```axon
mod std::nn::init {
    fn xavier_uniform(tensor: &mut GradTensor, gain: Float32);
    fn xavier_normal(tensor: &mut GradTensor, gain: Float32);
    fn kaiming_uniform(tensor: &mut GradTensor, mode: &String, nonlinearity: &String);
    fn kaiming_normal(tensor: &mut GradTensor, mode: &String, nonlinearity: &String);
    fn uniform(tensor: &mut GradTensor, low: Float32, high: Float32);
    fn normal(tensor: &mut GradTensor, mean: Float32, std: Float32);
    fn zeros(tensor: &mut GradTensor);
    fn ones(tensor: &mut GradTensor);
    fn constant(tensor: &mut GradTensor, value: Float32);
}
```

---

## 4. Optimizers (`std::optim`)

### 4.1 Optimizer Trait

```axon
trait Optimizer {
    fn step(&mut self);          // Update parameters using gradients
    fn zero_grad(&mut self);     // Reset all parameter gradients
    fn state_dict(&self) -> OptimizerState;
    fn load_state_dict(&mut self, state: OptimizerState);
}
```

### 4.2 Implementations

```axon
// Stochastic Gradient Descent
struct SGD {
    params: Vec<&mut GradTensor>,
    lr: Float32,
    momentum: Float32,
    weight_decay: Float32,
    dampening: Float32,
    nesterov: Bool,
}

// Adam
struct Adam {
    params: Vec<&mut GradTensor>,
    lr: Float32,
    betas: (Float32, Float32),   // default: (0.9, 0.999)
    eps: Float32,                 // default: 1e-8
    weight_decay: Float32,
}

// AdamW (decoupled weight decay)
struct AdamW {
    params: Vec<&mut GradTensor>,
    lr: Float32,
    betas: (Float32, Float32),
    eps: Float32,
    weight_decay: Float32,        // default: 0.01
}

// Learning rate schedulers
trait LrScheduler {
    fn step(&mut self);
    fn get_lr(&self) -> Float32;
}

struct StepLR { optimizer: &mut dyn Optimizer, step_size: Int64, gamma: Float32 }
struct CosineAnnealingLR { optimizer: &mut dyn Optimizer, t_max: Int64, eta_min: Float32 }
struct ReduceLROnPlateau { optimizer: &mut dyn Optimizer, patience: Int64, factor: Float32 }
struct OneCycleLR { optimizer: &mut dyn Optimizer, max_lr: Float32, total_steps: Int64 }
```

---

## 5. Loss Functions (`std::loss`)

```axon
trait LossFn {
    fn forward(&self, prediction: &GradTensor, target: &Tensor) -> GradTensor;
}

struct MSELoss {}                     // Mean Squared Error
struct L1Loss {}                      // Mean Absolute Error
struct CrossEntropyLoss {}            // Softmax + NLL
struct BCELoss {}                     // Binary Cross-Entropy
struct BCEWithLogitsLoss {}           // BCE + Sigmoid
struct NLLLoss {}                     // Negative Log Likelihood
struct HuberLoss { delta: Float32 }   // Smooth L1
struct KLDivLoss {}                   // KL Divergence
struct CosineEmbeddingLoss {}
struct TripletMarginLoss { margin: Float32 }
struct CTCLoss {}                     // Connectionist Temporal Classification
```

---

## 6. Training Utilities (`std::train`)

### 6.1 Training Loop

```axon
struct Trainer<M: Module, O: Optimizer, L: LossFn> {
    model: M,
    optimizer: O,
    loss_fn: L,
    device: Device,
    callbacks: Vec<Box<dyn Callback>>,
}

impl<M: Module, O: Optimizer, L: LossFn> Trainer<M, O, L> {
    fn new(model: M, optimizer: O, loss_fn: L) -> Trainer<M, O, L>;
    fn fit(&mut self, train_loader: &DataLoader, epochs: Int64);
    fn evaluate(&self, val_loader: &DataLoader) -> Metrics;
    fn predict(&self, input: &Tensor) -> Tensor;
}

trait Callback {
    fn on_epoch_start(&mut self, epoch: Int64);
    fn on_epoch_end(&mut self, epoch: Int64, metrics: &Metrics);
    fn on_batch_start(&mut self, batch: Int64);
    fn on_batch_end(&mut self, batch: Int64, loss: Float32);
    fn on_train_end(&mut self);
}
```

### 6.2 Checkpointing

```axon
struct Checkpoint {
    fn save(model: &dyn Module, optimizer: &dyn Optimizer, path: &String, epoch: Int64) -> Result<(), IoError>;
    fn load(path: &String) -> Result<(ModelState, OptimizerState, Int64), IoError>;
}
```

### 6.3 Mixed Precision Training

```axon
struct GradScaler {
    fn new(init_scale: Float32) -> GradScaler;
    fn scale(&self, loss: &GradTensor) -> GradTensor;
    fn step(&mut self, optimizer: &mut dyn Optimizer);
    fn update(&mut self);
}

fn autocast<F: Fn() -> T, T>(f: F) -> T;  // Auto-cast to Float16 where safe
```

---

## 7. Metrics (`std::metrics`)

```axon
mod std::metrics {
    fn accuracy(predictions: &Tensor<Int64, [?]>, targets: &Tensor<Int64, [?]>) -> Float32;
    fn precision(predictions: &Tensor, targets: &Tensor, average: &String) -> Float32;
    fn recall(predictions: &Tensor, targets: &Tensor, average: &String) -> Float32;
    fn f1_score(predictions: &Tensor, targets: &Tensor, average: &String) -> Float32;
    fn confusion_matrix(predictions: &Tensor<Int64, [?]>, targets: &Tensor<Int64, [?]>, num_classes: Int64) -> Tensor<Int64, [?, ?]>;
    fn mean_squared_error(predictions: &Tensor, targets: &Tensor) -> Float32;
    fn mean_absolute_error(predictions: &Tensor, targets: &Tensor) -> Float32;
    fn r2_score(predictions: &Tensor, targets: &Tensor) -> Float32;
    fn roc_auc(predictions: &Tensor, targets: &Tensor) -> Float32;
}
```

---

## 8. Model Export (`std::export`)

### 8.1 ONNX Export

```axon
mod std::export::onnx {
    fn export(
        model: &dyn Module,
        sample_input: &Tensor,
        path: &String,
        opset_version: Int32,
        input_names: Vec<String>,
        output_names: Vec<String>,
    ) -> Result<(), ExportError>;
}
```

### 8.2 Native Serialization

```axon
mod std::export {
    fn save_model(model: &dyn Module, path: &String) -> Result<(), IoError>;
    fn load_model<M: Module>(path: &String) -> Result<M, IoError>;
    fn save_tensor(tensor: &Tensor, path: &String) -> Result<(), IoError>;
    fn load_tensor(path: &String) -> Result<Tensor, IoError>;
}
```

---

## 9. Data Transforms (`std::data::transforms`)

```axon
mod std::data::transforms {
    // Image transforms
    fn resize(img: &Tensor, size: (Int64, Int64)) -> Tensor;
    fn center_crop(img: &Tensor, size: (Int64, Int64)) -> Tensor;
    fn random_crop(img: &Tensor, size: (Int64, Int64)) -> Tensor;
    fn random_horizontal_flip(img: &Tensor, p: Float32) -> Tensor;
    fn normalize(tensor: &Tensor, mean: &Tensor, std: &Tensor) -> Tensor;
    fn to_tensor(data: &[UInt8], shape: (Int64, Int64, Int64)) -> Tensor<Float32, [?, ?, ?]>;

    // Text transforms
    fn tokenize(text: &String, vocab: &Vocab) -> Vec<Int64>;
    fn pad_sequence(sequences: &Vec<Vec<Int64>>, max_len: Int64) -> Tensor<Int64, [?, ?]>;

    // Compose multiple transforms
    struct Compose {
        fn new(transforms: Vec<Box<dyn Transform>>) -> Compose;
    }
    trait Transform {
        fn apply(&self, input: &Tensor) -> Tensor;
    }
}
```

---

## 10. Task Breakdown

### Phase 6a: Autograd

- [ ] T170 Design computation graph data structure — `stdlib/autograd/graph.axon`
- [ ] T171 Implement GradTensor with gradient tracking — `stdlib/autograd/grad_tensor.axon`
- [ ] T172 Implement backward pass (reverse-mode AD) — `stdlib/autograd/backward.axon`
- [ ] T173 Implement gradient rules for all operations — `stdlib/autograd/ops.axon`
- [ ] T174 Implement no_grad context and gradient checkpointing — `stdlib/autograd/context.axon`

### Phase 6b: Neural Network Layers

- [ ] T175 Implement Module trait — `stdlib/nn/module.axon`
- [ ] T176 Implement Linear, Conv2d, BatchNorm, LayerNorm — `stdlib/nn/layers.axon`
- [ ] T177 Implement Dropout, MaxPool2d, AvgPool2d — `stdlib/nn/layers.axon`
- [ ] T178 Implement LSTM, GRU — `stdlib/nn/recurrent.axon`
- [ ] T179 Implement MultiHeadAttention, TransformerEncoder — `stdlib/nn/transformer.axon`
- [ ] T180 Implement Embedding — `stdlib/nn/embedding.axon`
- [ ] T181 Implement Sequential container — `stdlib/nn/sequential.axon`
- [ ] T182 Implement activation modules (ReLU, GELU, SiLU, Softmax) — `stdlib/nn/activation.axon`
- [ ] T183 Implement weight initialization — `stdlib/nn/init.axon`

### Phase 6c: Optimizers & Loss

- [ ] T184 Implement Optimizer trait — `stdlib/optim/mod.axon`
- [ ] T185 Implement SGD — `stdlib/optim/sgd.axon`
- [ ] T186 Implement Adam, AdamW — `stdlib/optim/adam.axon`
- [ ] T187 Implement LR schedulers (StepLR, CosineAnnealing, OneCycleLR) — `stdlib/optim/scheduler.axon`
- [ ] T188 Implement all loss functions — `stdlib/loss/`

### Phase 6d: Training & Export

- [ ] T189 Implement Trainer with callbacks — `stdlib/train/trainer.axon`
- [ ] T190 Implement checkpointing (save/load) — `stdlib/train/checkpoint.axon`
- [ ] T191 Implement mixed precision training (GradScaler, autocast) — `stdlib/train/amp.axon`
- [ ] T192 Implement ONNX export — `stdlib/export/onnx.axon`
- [ ] T193 Implement native model serialization — `stdlib/export/native.axon`

### Phase 6e: Metrics & Transforms

- [ ] T194 Implement accuracy, precision, recall, F1 — `stdlib/metrics/`
- [ ] T195 Implement confusion matrix, ROC-AUC — `stdlib/metrics/`
- [ ] T196 Implement image transforms — `stdlib/data/transforms.axon`
- [ ] T197 Implement text transforms (tokenize, pad) — `stdlib/data/transforms.axon`

### Phase 6f: Testing

- [ ] T198 Test autograd: gradient computation for all ops — `tests/autograd_tests.rs`
- [ ] T199 Test: train a simple MLP on XOR — `tests/train_tests.rs`
- [ ] T200 Test: train a CNN on MNIST subset — `tests/train_tests.rs`
- [ ] T201 Test: ONNX export produces valid file — `tests/export_tests.rs`
- [ ] T202 Test: gradient checkpointing reduces memory — `tests/autograd_tests.rs`
- [ ] T203 Test: mixed precision training converges — `tests/train_tests.rs`
- [ ] T204 Benchmark: compare training speed vs PyTorch baseline — `benches/`

---

## 11. Acceptance Criteria

- [ ] Autograd correctly computes gradients for all supported operations
- [ ] A 2-layer MLP can be defined, trained on XOR, and converges to 100% accuracy
- [ ] A CNN can be trained on a MNIST subset and achieves >95% accuracy
- [ ] Transformer encoder layer produces correct output shapes
- [ ] Adam optimizer matches PyTorch behavior for identical inputs
- [ ] ONNX export produces a valid model loadable by onnxruntime
- [ ] Mixed precision training runs without NaN gradients
- [ ] Shape mismatches in layer connections caught at compile time
- [ ] All operations work on both CPU and GPU (@cpu / @gpu)
- [ ] Training checkpoints can be saved and resumed
