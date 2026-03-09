# Phase 14: Model Validation & Dogfooding

## Summary

Phase 14 is where Axon proves itself.  We train progressively complex
models, benchmark against PyTorch, and fix every bug we find.  This phase
is less about new features and more about **making everything work
together reliably**.

After this phase, these models train successfully in Axon:
1. Linear regression on synthetic data
2. MLP on MNIST (>98% accuracy)
3. CNN on CIFAR-10 (>85% accuracy)
4. Small transformer (character-level language model)
5. User's custom model

## Prerequisites

| Requirement | Provided by |
|-------------|-------------|
| All tensor ops on CPU + GPU | Phase 10, 12 |
| Autograd engine | Phase 11 |
| nn.Module, optimizers, loss functions | Phase 13 |
| DataLoader, MNIST reader | Phase 13 |
| Checkpointing | Phase 13 |

## Architecture

This phase has no new architecture — it's validation of Phases 9–13.
The output is:

1. **Reference model implementations** in `examples/`
2. **Performance benchmark results** in `docs/benchmarks/`
3. **Bug fixes** across the entire codebase
4. **Regression tests** for every bug found

### File Layout

```
examples/
├── linear_regression.axon     # Simplest model
├── mnist_mlp.axon             # MLP classifier
├── cifar10_cnn.axon           # CNN classifier
├── char_rnn.axon              # Character-level RNN/Transformer
├── custom_model.axon          # User's model (defined in T445)
└── README.md                  # How to run each example

docs/benchmarks/
├── results.md                 # Performance comparison tables
├── training_curves/           # Loss/accuracy plots (CSV data)
└── memory_profiles/           # Peak memory usage reports
```

---

## Task List

### Sub-phase 14a: Linear Regression (T433–T436)

#### T433 — Write linear regression example
- **File**: `examples/linear_regression.axon`
- **Description**: Generate synthetic data: `y = 3x + 2 + noise`.
  Train a single Linear(1, 1) with MSE loss and SGD optimizer.
  Print loss every 10 steps.  Should converge to w≈3, b≈2.
- **Acceptance**: Final w within 0.1 of 3.0, b within 0.1 of 2.0.

#### T434 — Verify convergence on CPU
- **File**: `tests/model_validation_tests.rs`
- **Description**: Compile and run `linear_regression.axon` on CPU.
  Parse stdout, verify final loss < 0.1.
- **Acceptance**: Test passes in < 10 seconds.

#### T435 — Verify convergence on GPU
- **File**: `tests/model_validation_tests.rs`
- **Description**: Same as T434 but with `.to_gpu()` calls.
  Results should match CPU within tolerance.
- **Acceptance**: GPU and CPU final loss within 1% of each other.

#### T436 — Bug fix sprint: linear regression
- **Description**: Fix all bugs discovered during T433–T435.
  Add regression tests for each bug.
- **Acceptance**: All bugs have corresponding regression tests.

### Sub-phase 14b: MNIST MLP (T437–T441)

#### T437 — Write MNIST MLP example
- **File**: `examples/mnist_mlp.axon`
- **Description**:
  ```
  Model: Linear(784, 256) → ReLU → Linear(256, 128) → ReLU → Linear(128, 10)
  Loss: CrossEntropyLoss
  Optimizer: Adam(lr=0.001)
  Data: MNIST, batch_size=64
  Epochs: 10
  ```
  Print train loss and test accuracy per epoch.
- **Acceptance**: Test accuracy > 98% after 10 epochs.

#### T438 — MNIST training test (CPU)
- **File**: `tests/model_validation_tests.rs`
- **Description**: Run MNIST MLP on CPU.  Verify accuracy > 97% after
  5 epochs (relaxed for test speed).
- **Acceptance**: Test passes in < 5 minutes.

#### T439 — MNIST training test (GPU)
- **File**: `tests/model_validation_tests.rs`
- **Description**: Run MNIST MLP on GPU.  Same accuracy target.
  Time the training and compare to CPU time.
- **Acceptance**: GPU at least 2× faster than CPU for batch_size=64.

#### T440 — MNIST checkpoint test
- **File**: `tests/model_validation_tests.rs`
- **Description**: Train 5 epochs, save checkpoint, load checkpoint,
  train 5 more.  Compare to training 10 epochs straight.
  Final accuracy should be within 0.5%.
- **Acceptance**: Checkpoint resume gives equivalent results.

#### T441 — Bug fix sprint: MNIST
- **Description**: Fix all bugs.  Common expected issues:
  - Cross-entropy numerics (log of zero)
  - Softmax overflow with large logits
  - Batch dimension handling in loss functions
  - Memory leaks in training loops
- **Acceptance**: All bugs have regression tests.

### Sub-phase 14c: CIFAR-10 CNN (T442–T446)

#### T442 — Write CIFAR-10 CNN example
- **File**: `examples/cifar10_cnn.axon`
- **Description**:
  ```
  Model:
    Conv2d(3, 32, 3, padding=1) → BatchNorm → ReLU → MaxPool2d(2)
    Conv2d(32, 64, 3, padding=1) → BatchNorm → ReLU → MaxPool2d(2)
    Conv2d(64, 128, 3, padding=1) → BatchNorm → ReLU → MaxPool2d(2)
    Linear(128*4*4, 256) → ReLU → Dropout(0.5)
    Linear(256, 10)
  Loss: CrossEntropyLoss
  Optimizer: AdamW(lr=0.001, weight_decay=0.01)
  Data: CIFAR-10, batch_size=128, with random flip + normalize
  Epochs: 30
  ```
- **Acceptance**: Test accuracy > 85% after 30 epochs.

#### T443 — CIFAR-10 dataset reader
- **File**: `runtime/axon_data_cifar.c`
- **Description**: Read CIFAR-10 binary format (5 train batches + 1 test).
  Images: `[N, 3, 32, 32]` f32 (normalize to [0,1]).
  Labels: `[N]` i64.
- **Acceptance**: Loads 50k train / 10k test images.

#### T444 — CIFAR-10 training test (GPU)
- **File**: `tests/model_validation_tests.rs`
- **Description**: Train CNN on CIFAR-10 on GPU for 10 epochs.
  Verify accuracy > 70% (relaxed for test speed).
- **Acceptance**: Training doesn't crash, accuracy improves over epochs.

#### T445 — Benchmark CNN vs PyTorch
- **File**: `docs/benchmarks/results.md`
- **Description**: Run same architecture in PyTorch and Axon.
  Compare: training time per epoch, peak GPU memory, final accuracy.
  Target: Axon within 2× of PyTorch training speed.
- **Acceptance**: Benchmark results documented with clear methodology.

#### T446 — Bug fix sprint: CNN
- **Description**: Fix all bugs.  Expected issues:
  - Conv2d backward correctness
  - BatchNorm running stats update
  - MaxPool2d backward (index tracking)
  - Memory pressure from conv workspace
- **Acceptance**: All bugs have regression tests.

### Sub-phase 14d: Transformer (T447–T451)

#### T447 — Write character-level transformer example
- **File**: `examples/char_rnn.axon`
- **Description**:
  ```
  Model:
    Embedding(vocab_size, 128)
    4× TransformerBlock:
      MultiHeadAttention(128, 4) + LayerNorm + residual
      Linear(128, 512) → GELU → Linear(512, 128) + LayerNorm + residual
    Linear(128, vocab_size)
  Loss: CrossEntropyLoss
  Optimizer: Adam(lr=3e-4) with cosine scheduler
  Data: Shakespeare text file, context_length=128
  ```
  Generate 200 characters after training.
- **Acceptance**: Generated text is recognizably English-like after 1000 steps.

#### T448 — Text dataset reader
- **File**: `runtime/axon_data_text.c`
- **Description**: `axon_text_dataset(path, context_length, vocab)`:
  Character-level tokenizer.  Sliding window over text file.
  Returns `(input[context_length], target[context_length])` pairs.
- **Acceptance**: Shakespeare dataset produces correct input/target pairs.

#### T449 — Transformer training test (GPU)
- **File**: `tests/model_validation_tests.rs`
- **Description**: Train char transformer for 500 steps on GPU.
  Verify loss decreases by at least 50% from initial.
- **Acceptance**: Loss curve shows clear learning.

#### T450 — Attention mask and causal masking
- **Description**: Verify causal mask works in MultiHeadAttention:
  position i can only attend to positions ≤ i.
  Test: attention weights are zero above diagonal.
- **Acceptance**: Causal masking prevents future leakage.

#### T451 — Bug fix sprint: Transformer
- **Description**: Fix all bugs.  Expected issues:
  - Attention scaling (missing sqrt(d_k) division)
  - Residual connection gradient flow
  - LayerNorm backward
  - Positional encoding
  - Memory growth from attention matrices
- **Acceptance**: All bugs have regression tests.

### Sub-phase 14e: Custom Model & Final Polish (T452–T458)

#### T452 — Define user's custom model architecture
- **Description**: Work with the user to define their model architecture.
  Write it in Axon.  Could be any of: image classifier, text model,
  regression, GAN, autoencoder, etc.
- **Acceptance**: User approves the architecture.

#### T453 — Implement custom model
- **File**: `examples/custom_model.axon`
- **Description**: Implement the architecture from T452.  Include
  data loading, training loop, evaluation, checkpointing.
- **Acceptance**: Model compiles and begins training.

#### T454 — Train custom model to target metrics
- **Description**: Train until the user is satisfied with results.
  Debug any issues that arise.
- **Acceptance**: User-defined success metric met.

#### T455 — Performance profiling
- **File**: `docs/benchmarks/results.md`
- **Description**: Profile all 4 reference models:
  - Wall-clock time per epoch
  - GPU utilization %
  - Peak GPU memory
  - CPU→GPU transfer overhead
  - Autograd overhead (forward vs backward ratio)
  Identify top 3 bottlenecks.
- **Acceptance**: Profile results documented, bottlenecks identified.

#### T456 — Memory leak audit
- **Description**: Run each model for 100 epochs, monitor memory.
  No growth beyond expected model + optimizer state.
  GPU memory: check with `axon_gpu_memory_peak()`.
  CPU memory: check with `/proc/self/statm` or similar.
- **Acceptance**: No leaks found, or all leaks fixed.

#### T457 — Numerical stability audit
- **Description**: Check all models for:
  - NaN/Inf in loss or gradients
  - Loss explosion (divergence)
  - Gradient vanishing (all zeros)
  Add `axon_tensor_has_nan(t)`, `axon_tensor_has_inf(t)` utility functions.
  Add optional gradient anomaly detection mode.
- **Acceptance**: No numerical issues in standard training.

#### T458 — Final regression test suite
- **File**: `tests/model_validation_tests.rs`
- **Description**: Consolidate all bug fix regression tests.
  Ensure all 4 reference models have automated tests.
  Total test count target: 100+ new tests from this phase.
- **Acceptance**: All tests pass on both CPU and GPU.

---

## Performance Targets

| Model | Metric | Target vs PyTorch |
|-------|--------|-------------------|
| Linear Regression | Time to converge | ≤ 2× |
| MNIST MLP | Epoch time (CPU) | ≤ 3× |
| MNIST MLP | Epoch time (GPU) | ≤ 2× |
| CIFAR-10 CNN | Epoch time (GPU) | ≤ 2× |
| Transformer | Epoch time (GPU) | ≤ 2.5× |
| All models | Peak GPU memory | ≤ 1.5× |

---

## Test Plan

1. **Automated model tests** — compile, train for N steps, check metrics
2. **Performance benchmarks** — wall-clock comparison with PyTorch
3. **Memory tests** — no leaks over 100 epochs
4. **Numerical tests** — no NaN/Inf in standard training
5. **Checkpoint tests** — save/load/resume produces equivalent results
6. **Cross-platform** — models train on Windows, Linux, macOS (CPU)
   and on CUDA + ROCm (GPU)

---

## Exit Criteria

1. All 26 tasks complete
2. Linear regression converges to correct weights
3. MNIST MLP achieves > 98% test accuracy
4. CIFAR-10 CNN achieves > 85% test accuracy
5. Transformer generates recognizable text
6. User's custom model trains successfully
7. No memory leaks
8. No numerical instability in standard training
9. Performance within specified targets vs PyTorch
10. 100+ regression tests from this phase
