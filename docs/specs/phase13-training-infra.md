# Phase 13: Training Infrastructure

## Summary

Phase 13 delivers everything needed to train real neural networks: working
optimizers, loss functions, the nn.Module system, data loading, and
checkpointing.  After this phase:

```axon
use std::nn::{Linear, ReLU, Sequential};
use std::optim::Adam;
use std::loss::CrossEntropyLoss;
use std::data::{DataLoader, MnistDataset};

fn main() {
    let model = Sequential::new([
        Linear::new(784, 256),
        ReLU::new(),
        Linear::new(256, 10),
    ]);
    let optimizer = Adam::new(model.parameters(), lr: 0.001);
    let loss_fn = CrossEntropyLoss::new();

    for epoch in 0..10 {
        for (images, labels) in DataLoader::new(MnistDataset::train(), batch_size: 64) {
            let output = model.forward(images.reshape([-1, 784]));
            let loss = loss_fn.forward(output, labels);
            optimizer.zero_grad();
            loss.backward();
            optimizer.step();
        }
        println("Epoch {epoch}: loss = {loss}");
    }
    model.save("mnist_model.axon");
}
```

## Prerequisites

| Requirement | Provided by |
|-------------|-------------|
| Tensor ops on CPU + GPU | Phase 10, 12 |
| Autograd (backward, gradients) | Phase 11 |
| to_gpu / to_cpu transfers | Phase 12 |
| cuBLAS/rocBLAS matmul | Phase 12 |
| cuDNN/MIOpen conv, batchnorm | Phase 12 |

## Architecture

### Design Decisions

1. **nn.Module as C struct + vtable** — Since Axon's trait system is
   compile-time only, nn.Module is implemented as a C runtime struct
   with function pointers for `forward`, `parameters`, `train`, `eval`.
   Axon traits map to these vtables.

2. **Parameter registration** — Each module's `new()` allocates its
   parameters (weight, bias) with `requires_grad: true` and registers
   them in a parameter list.  `model.parameters()` collects all
   recursively.

3. **Optimizers operate on parameter lists** — `Adam::new(params, lr)`
   takes a slice of tensor pointers.  `step()` updates each in-place.

4. **DataLoader as a C iterator** — Yields `(batch_inputs, batch_labels)`
   tuples.  Data loading uses a thread pool for parallel I/O.

5. **Checkpoint format** — Binary format: header (magic, version, dtype
   counts), then parameter tensors in order.  No need for backward-
   compatibility in v1.0 — just a simple dump.

### File Layout

```
runtime/
├── axon_nn.h               # nn.Module struct, layer prototypes
├── axon_nn.c               # Module base: parameters(), train(), eval()
├── axon_nn_linear.c        # Linear layer: y = x @ w + b
├── axon_nn_conv.c          # Conv2d layer (wraps cuDNN/MIOpen or CPU impl)
├── axon_nn_norm.c          # BatchNorm, LayerNorm
├── axon_nn_activate.c      # ReLU, GELU, Sigmoid modules (stateless)
├── axon_nn_dropout.c       # Dropout layer
├── axon_nn_embed.c         # Embedding layer
├── axon_nn_attention.c     # MultiHeadAttention
├── axon_nn_container.c     # Sequential, ModuleList
├── axon_optim.h            # Optimizer struct, prototypes
├── axon_optim.c            # SGD, Adam, AdamW implementations
├── axon_loss.h             # Loss function prototypes
├── axon_loss.c             # MSE, CrossEntropy, BCE, L1
├── axon_data.h             # DataLoader, Dataset prototypes
├── axon_data.c             # DataLoader: batching, shuffling, threading
├── axon_data_csv.c         # CSV dataset reader
├── axon_data_image.c       # Image dataset reader (PNG/JPG → tensor)
├── axon_data_mnist.c       # MNIST binary format reader
├── axon_checkpoint.h       # Save/load prototypes
├── axon_checkpoint.c       # Binary checkpoint format
├── axon_lr_scheduler.c     # Learning rate schedulers
└── axon_metrics.c          # Accuracy, confusion matrix, F1
```

---

## Task List

### Sub-phase 13a: nn.Module System (T396–T404)

#### T396 — Module base struct and vtable
- **File**: `runtime/axon_nn.h`, `runtime/axon_nn.c`
- **Description**: Define `AxonModule` struct:
  ```c
  typedef struct AxonModule {
      const char *name;
      AxonTensor **params;       // owned parameter tensors
      int n_params;
      struct AxonModule **children; // sub-modules
      int n_children;
      uint8_t training;          // 1 = train mode, 0 = eval mode
      // vtable
      AxonTensor* (*forward)(struct AxonModule*, AxonTensor*);
      void (*to_device)(struct AxonModule*, AxonDevice);
  } AxonModule;
  ```
  Implement `axon_module_parameters(m)` — recursively collect all params.
  `axon_module_train(m)` / `axon_module_eval(m)` — set training flag recursively.
  `axon_module_to_gpu(m)` / `axon_module_to_cpu(m)` — move all params.
- **Acceptance**: Module with 2 Linear children reports 4 parameters (2 weights + 2 biases).

#### T397 — Linear layer
- **File**: `runtime/axon_nn_linear.c`
- **Description**: `axon_linear_new(in_features, out_features, bias)`:
  - Allocate weight `[out_features, in_features]` with Xavier init
  - Allocate bias `[out_features]` with zeros (if bias=true)
  - Both with `requires_grad = true`
  `axon_linear_forward(m, x)`: `x @ w^T + b` (handles batched input).
- **Acceptance**: Linear(784, 256) forward on `[32, 784]` → `[32, 256]`.

#### T398 — Conv2d layer
- **File**: `runtime/axon_nn_conv.c`
- **Description**: `axon_conv2d_new(in_ch, out_ch, kernel_size, stride, padding)`:
  - Weight `[out_ch, in_ch, kH, kW]` with Kaiming init
  - Bias `[out_ch]` with zeros
  Forward: on GPU, use cuDNN/MIOpen.  On CPU, implement im2col + matmul.
- **Acceptance**: Conv2d(3, 16, 3, padding=1) on `[1, 3, 32, 32]` → `[1, 16, 32, 32]`.

#### T399 — BatchNorm and LayerNorm
- **File**: `runtime/axon_nn_norm.c`
- **Description**:
  `axon_batchnorm_new(num_features)`: scale, bias (learnable), running_mean,
  running_var (not learnable).  Forward: normalize, scale, shift.
  Training mode: compute batch stats, update running stats.
  Eval mode: use running stats.
  `axon_layernorm_new(normalized_shape)`: similar but normalizes last N dims.
- **Acceptance**: BatchNorm output has ~0 mean, ~1 std along batch dim.

#### T400 — Activation modules
- **File**: `runtime/axon_nn_activate.c`
- **Description**: `axon_relu_module_new()`, `axon_gelu_module_new()`,
  `axon_sigmoid_module_new()`.  Stateless — `forward` just calls the
  tensor activation function.
- **Acceptance**: ReLU module forward matches tensor relu.

#### T401 — Dropout
- **File**: `runtime/axon_nn_dropout.c`
- **Description**: `axon_dropout_new(p)`.  Forward:
  Training mode: generate random mask, multiply input by mask / (1-p).
  Eval mode: identity (return input).
- **Acceptance**: ~p fraction of outputs are zero in training.

#### T402 — Embedding
- **File**: `runtime/axon_nn_embed.c`
- **Description**: `axon_embedding_new(num_embeddings, embedding_dim)`:
  Weight `[num_embeddings, embedding_dim]` with normal init.
  Forward: index into weight table.  Input is integer tensor.
- **Acceptance**: Embedding(1000, 64) on `[32, 10]` → `[32, 10, 64]`.

#### T403 — MultiHeadAttention
- **File**: `runtime/axon_nn_attention.c`
- **Description**: `axon_mha_new(embed_dim, num_heads)`:
  Q, K, V projection weights + output projection.
  Forward: split heads, scaled dot-product attention, concat, project.
  `attention(Q, K, V) = softmax(Q @ K^T / sqrt(d_k)) @ V`
  Optional: attention mask for causal/padding masking.
- **Acceptance**: MHA(512, 8) on `[32, 100, 512]` → `[32, 100, 512]`.

#### T404 — Sequential and ModuleList
- **File**: `runtime/axon_nn_container.c`
- **Description**: `axon_sequential_new(modules, n)`:
  Forward: chain `forward()` calls.
  `axon_modulelist_new()`: container for dynamic module lists.
- **Acceptance**: Sequential of 3 layers produces correct output.

### Sub-phase 13b: Optimizers (T405–T410)

#### T405 — SGD optimizer
- **File**: `runtime/axon_optim.c`
- **Description**: `axon_sgd_new(params, n_params, lr, momentum, weight_decay)`.
  `step()`: for each param:
  ```
  if weight_decay > 0: grad += weight_decay * param
  if momentum > 0: buf = momentum * buf + grad; update = buf
  else: update = grad
  param -= lr * update
  ```
  State: velocity buffer per parameter.
- **Acceptance**: SGD converges on `y = 2x` in < 100 steps.

#### T406 — Adam optimizer
- **File**: `runtime/axon_optim.c`
- **Description**: `axon_adam_new(params, n_params, lr, beta1, beta2, eps)`.
  `step()`: standard Adam algorithm:
  ```
  m = beta1 * m + (1-beta1) * grad
  v = beta2 * v + (1-beta2) * grad^2
  m_hat = m / (1 - beta1^t)
  v_hat = v / (1 - beta2^t)
  param -= lr * m_hat / (sqrt(v_hat) + eps)
  ```
  State: m, v buffers per parameter, step count.
- **Acceptance**: Adam converges on quadratic in < 50 steps.

#### T407 — AdamW optimizer
- **File**: `runtime/axon_optim.c`
- **Description**: Like Adam but decoupled weight decay:
  `param -= lr * weight_decay * param` (before Adam update, not in gradient).
- **Acceptance**: AdamW with weight_decay > 0 reduces param magnitudes.

#### T408 — Learning rate schedulers
- **File**: `runtime/axon_lr_scheduler.c`
- **Description**:
  `axon_lr_step(optimizer, step_size, gamma)` — multiply lr by gamma every step_size.
  `axon_lr_cosine(optimizer, T_max, eta_min)` — cosine annealing.
  `axon_lr_warmup(optimizer, warmup_steps, target_lr)` — linear warmup.
- **Acceptance**: Cosine scheduler LR matches expected formula.

#### T409 — Gradient clipping
- **File**: `runtime/axon_optim.c`
- **Description**:
  `axon_clip_grad_norm(params, n_params, max_norm)` — scale gradients
  so total L2 norm ≤ max_norm.  Returns actual norm.
  `axon_clip_grad_value(params, n_params, clip_value)` — clamp each element.
- **Acceptance**: After clipping, grad norm ≤ max_norm.

#### T410 — Optimizer state save/load
- **File**: `runtime/axon_optim.c`
- **Description**: `axon_optimizer_save(opt, path)`,
  `axon_optimizer_load(opt, path)`.  Save momentum buffers, step count.
  Needed for resume-from-checkpoint.
- **Acceptance**: Save → load → continue training gives same result as uninterrupted.

### Sub-phase 13c: Loss Functions (T411–T415)

#### T411 — MSE Loss
- **File**: `runtime/axon_loss.c`
- **Description**: `axon_mse_loss(pred, target, reduction)`:
  `loss = mean((pred - target)^2)` (or sum, or none).
  Autograd-compatible (backward gives `2*(pred-target)/N`).
- **Acceptance**: MSE of `[1,2,3]` vs `[1,2,3]` is 0.

#### T412 — Cross-Entropy Loss
- **File**: `runtime/axon_loss.c`
- **Description**: `axon_cross_entropy_loss(logits, targets, reduction)`:
  `loss = -sum(log_softmax(logits)[targets]) / N`
  Targets are integer class indices.  Numerically stable implementation
  (log-sum-exp trick).
- **Acceptance**: CE loss of perfect predictions approaches 0.

#### T413 — BCE Loss
- **File**: `runtime/axon_loss.c`
- **Description**: Binary cross-entropy for sigmoid output:
  `loss = -mean(target * log(pred) + (1-target) * log(1-pred))`
  Clamp pred to [eps, 1-eps] for numerical stability.
- **Acceptance**: BCE of 1.0 vs target 1.0 approaches 0.

#### T414 — L1 Loss
- **File**: `runtime/axon_loss.c`
- **Description**: `loss = mean(|pred - target|)`.
- **Acceptance**: L1 of identical tensors is 0.

#### T415 — NLL Loss
- **File**: `runtime/axon_loss.c`
- **Description**: Negative log-likelihood (takes log-probabilities as input):
  `loss = -mean(input[target])`.  Combined with log_softmax for CE.
- **Acceptance**: NLL loss gradient check passes.

### Sub-phase 13d: Data Loading (T416–T422)

#### T416 — Dataset interface
- **File**: `runtime/axon_data.h`
- **Description**: Define `AxonDataset` struct:
  ```c
  typedef struct {
      int64_t length;
      AxonTensor* (*get_item)(struct AxonDataset*, int64_t index);
      AxonTensor* (*get_label)(struct AxonDataset*, int64_t index);
      void (*close)(struct AxonDataset*);
  } AxonDataset;
  ```
- **Acceptance**: Interface compiles.

#### T417 — DataLoader: batching and shuffling
- **File**: `runtime/axon_data.c`
- **Description**: `axon_dataloader_new(dataset, batch_size, shuffle, drop_last)`.
  Iterator interface: `axon_dataloader_next(dl, &batch_x, &batch_y)` → bool.
  Shuffle: Fisher-Yates on index array at epoch start.
  Batching: stack `batch_size` items along dim 0.
  `axon_dataloader_reset(dl)` — start new epoch.
- **Acceptance**: DataLoader yields correct number of batches for dataset size.

#### T418 — Parallel data loading
- **File**: `runtime/axon_data.c`
- **Description**: `num_workers` parameter.  Use a thread pool to
  prefetch next N batches while training on current batch.  Double-buffer
  scheme: while GPU trains on batch N, CPU loads batch N+1.
- **Acceptance**: num_workers=4 gives measurable speedup on disk-bound dataset.

#### T419 — MNIST dataset reader
- **File**: `runtime/axon_data_mnist.c`
- **Description**: Read IDX file format (MNIST, Fashion-MNIST).
  `axon_mnist_dataset(path, train)` — loads images as `[N, 28, 28]` f32
  (normalized to [0,1]) and labels as `[N]` i64.
  Auto-downloads from the internet if not found (or prints URL).
- **Acceptance**: Loads 60k train / 10k test images.

#### T420 — CSV dataset reader
- **File**: `runtime/axon_data_csv.c`
- **Description**: `axon_csv_dataset(path, label_col, dtype)` — reads
  CSV, numeric columns → tensor, one column is label.
  Handle headers, missing values (NaN), quoted strings.
- **Acceptance**: Reads Iris dataset, produces [150, 4] features + [150] labels.

#### T421 — Image folder dataset
- **File**: `runtime/axon_data_image.c`
- **Description**: `axon_image_dataset(root_path, resize_h, resize_w)`:
  Expects directory structure: `root/class_name/image.png`.
  Loads images using stb_image.h, resizes, converts to tensor.
  Class names → integer labels.
- **Acceptance**: Loads CIFAR-10 style directory into tensors.

#### T422 — Data transforms
- **File**: `runtime/axon_data.c`
- **Description**: Transform pipeline applied in DataLoader:
  `axon_transform_normalize(mean, std)` — `(x - mean) / std`
  `axon_transform_random_crop(size, padding)` — random crop with padding
  `axon_transform_random_flip(prob)` — horizontal flip
  `axon_transform_to_tensor()` — convert uint8 image to f32 [0,1]
- **Acceptance**: Transforms produce expected output shapes and value ranges.

### Sub-phase 13e: Checkpointing & Metrics (T423–T428)

#### T423 — Model save
- **File**: `runtime/axon_checkpoint.c`
- **Description**: `axon_model_save(module, path)`:
  Binary format: `[AXON magic][version][n_params]` then for each param:
  `[name_len][name][ndim][shape...][dtype][data_bytes...]`.
  Transfers GPU params to CPU before saving.
- **Acceptance**: Saves a 3-layer MLP to file < 10MB.

#### T424 — Model load
- **File**: `runtime/axon_checkpoint.c`
- **Description**: `axon_model_load(module, path)`:
  Read file, match parameter names, load data into existing module's
  params.  Error if shapes or param count don't match.
- **Acceptance**: Save → load → forward produces identical output.

#### T425 — Training state checkpoint
- **File**: `runtime/axon_checkpoint.c`
- **Description**: `axon_checkpoint_save(path, model, optimizer, epoch, loss)`:
  Save model params + optimizer state (momentum buffers, step) + epoch + loss.
  `axon_checkpoint_load(path, model, optimizer, &epoch, &loss)`: restore.
- **Acceptance**: Resume from checkpoint continues training correctly.

#### T426 — ONNX export
- **File**: `runtime/axon_export.c`
- **Description**: `axon_export_onnx(module, input_shape, path)`:
  Trace the model by running a dummy forward pass, record ops into
  an ONNX graph.  Write ONNX protobuf format.
  Support: Linear, Conv2d, ReLU, BatchNorm, Softmax, Add, MatMul.
- **Acceptance**: Exported ONNX runs in onnxruntime and matches Axon output.

#### T427 — Accuracy metric
- **File**: `runtime/axon_metrics.c`
- **Description**: `axon_accuracy(predictions, targets)` — top-1 accuracy.
  `axon_topk_accuracy(predictions, targets, k)` — top-k.
  Predictions are logits (argmax taken internally).
- **Acceptance**: Perfect predictions → 100% accuracy.

#### T428 — Training logger
- **File**: `runtime/axon_metrics.c`
- **Description**: `axon_logger_new(log_file)` — logs loss, accuracy,
  learning rate per step/epoch.  Optional TensorBoard-compatible output
  (simple events file format).
  `axon_logger_log_scalar(logger, tag, value, step)`.
- **Acceptance**: Log file contains readable training progression.

### Sub-phase 13f: Distributed Training (T429–T432)

#### T429 — NCCL/RCCL integration
- **File**: `runtime/axon_distributed.c`
- **Description**: `axon_dist_init(world_size, rank)` — init NCCL/RCCL.
  `axon_dist_all_reduce(tensor, op)` — sum/avg gradients across GPUs.
  `axon_dist_broadcast(tensor, src_rank)` — broadcast params.
  `axon_dist_barrier()` — synchronize all ranks.
- **Acceptance**: All-reduce of `[1.0]` on 2 GPUs → `[2.0]`.

#### T430 — DataParallel wrapper
- **File**: `runtime/axon_distributed.c`
- **Description**: `axon_data_parallel(module, device_ids, n_devices)`:
  Replicate model to each GPU.  Split batch across GPUs.
  Forward on each, gather outputs.  Backward on each, all-reduce gradients.
- **Acceptance**: DataParallel training matches single-GPU results (within tolerance).

#### T431 — Distributed DataLoader
- **File**: `runtime/axon_data.c`
- **Description**: `axon_distributed_sampler(dataset, world_size, rank)`:
  Partition dataset across ranks.  Each rank sees a unique subset.
  Shuffle seed synchronized across ranks.
- **Acceptance**: 2 ranks see non-overlapping halves of dataset.

#### T432 — Mixed precision training
- **File**: `runtime/axon_optim.c`
- **Description**: `axon_amp_scale(loss, scale_factor)` — loss scaling for FP16.
  `axon_amp_unscale(optimizer)` — unscale gradients before step.
  `axon_amp_update_scale(scale, had_inf)` — dynamic loss scaling.
  Maintain master weights in FP32, compute in FP16.
- **Acceptance**: AMP training of MLP converges similarly to FP32.

---

## Error Codes

| Code  | Name                  | Description                                  |
|-------|-----------------------|----------------------------------------------|
| E9001 | ModuleForwardFailed   | Forward pass returned null                   |
| E9002 | ShapeMismatchLayer    | Input shape doesn't match layer expectations |
| E9003 | OptimizerNullGrad     | step() called but no gradients computed      |
| E9004 | CheckpointLoadFailed  | Checkpoint file corrupt or shape mismatch    |
| E9005 | DatasetReadFailed     | Cannot read dataset file                     |
| E9006 | DatasetEmpty          | Dataset has 0 items                          |
| E9007 | DistributedInitFailed | NCCL/RCCL initialization failed              |
| E9008 | OnnxExportFailed      | Unsupported op during ONNX tracing           |

---

## Test Plan

1. **Layer tests** — each layer's forward produces correct shape and values
2. **Optimizer tests** — convergence on simple quadratics
3. **Loss tests** — known input/output pairs, gradient checks
4. **DataLoader tests** — correct batching, shuffling, epoch boundaries
5. **Checkpoint tests** — save → load → identical forward pass
6. **Integration: MNIST** — train to >98% accuracy as a Phase 13 gate
7. **Distributed tests** — run with `mpirun -np 2` on multi-GPU machine

---

## Exit Criteria

1. All 37 tasks complete
2. Can train MNIST MLP to >98% accuracy
3. All optimizers converge on test problems
4. Checkpoint save/load preserves training state exactly
5. DataLoader handles MNIST, CSV, image folders
6. GPU training works (forward + backward + optimizer step on GPU)
7. At least 80 new tests pass
