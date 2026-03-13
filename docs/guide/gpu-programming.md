# GPU Programming

Axon provides first-class GPU support through device annotations, automatic
kernel compilation, and device-aware tensor operations. Write GPU code in
Axon — no CUDA C required.

---

## Device Annotations

### `@gpu` Functions

Mark a function for GPU execution:

```axon
@gpu
fn vector_add(a: Tensor<Float32, [1024]>, b: Tensor<Float32, [1024]>): Tensor<Float32, [1024]> {
    a + b
}
```

The Axon compiler lowers `@gpu` functions through the MLIR backend to produce
optimized GPU kernels for the target platform (CUDA, ROCm, or Vulkan).

### `@cpu` Functions

Explicitly mark a function for CPU-only execution:

```axon
@cpu
fn save_results(data: Tensor<Float32, [?, 10]>, path: String) {
    val file = File.create(path);
    file.write(data);
}
```

### `@device` Annotation

Specify a device target explicitly:

```axon
@device("cuda:0")
fn forward_gpu0(x: Tensor<Float32, [?, 784]>): Tensor<Float32, [?, 10]> {
    // executes on CUDA device 0
    val w = randn([784, 10]);
    x @ w
}
```

---

## Device Transfer

Tensors are transferred between devices with `.to_gpu()` and `.to_cpu()`:

```axon
fn gpu_example() {
    // Create on CPU
    val cpu_tensor = randn([1024, 1024]);

    // Transfer to GPU
    val gpu_tensor = cpu_tensor.to_gpu();

    // Compute on GPU — fast!
    val result = gpu_tensor @ gpu_tensor;

    // Transfer back to CPU for I/O
    val cpu_result = result.to_cpu();
    println("{}", cpu_result);
}
```

### Transfer is a Move

Device transfer follows ownership rules — the source tensor is consumed:

```axon
val data = randn([256, 256]);
val gpu_data = data.to_gpu();   // data is moved
// println("{}", data);          // ERROR[E4001]: use of moved value `data`
```

To keep a CPU copy, clone first:

```axon
val data = randn([256, 256]);
val backup = data.clone();
val gpu_data = data.to_gpu();
println("{}", backup);           // OK — backup is a separate copy
```

---

## Tensor Device Placement

Tensors track their device in the type system. Operations between tensors
on different devices are compile-time errors:

```axon
val cpu_a = randn([100]);
val gpu_b = randn([100]).to_gpu();
// val c = cpu_a + gpu_b;  // ERROR: device mismatch — cpu and gpu tensors
```

### Creating Tensors Directly on GPU

```axon
@gpu
fn init_weights(): Tensor<Float32, [784, 256]> {
    randn([784, 256])   // created directly on GPU — no transfer needed
}
```

---

## GPU Kernel Compilation

When you compile with `--gpu`, Axon compiles `@gpu` functions into GPU kernels:

```bash
# Compile for NVIDIA GPUs
axonc build model.axon --gpu cuda -O 3

# Compile for AMD GPUs
axonc build model.axon --gpu rocm -O 3

# Compile for Vulkan (cross-platform)
axonc build model.axon --gpu vulkan -O 3
```

### How It Works

1. **Frontend**: Axon source → AST → Typed AST (same for all targets)
2. **MIR**: Typed AST → Mid-level IR with device annotations
3. **MLIR**: GPU-annotated MIR → MLIR dialects (GPU, Linalg, Tensor)
4. **Lowering**: MLIR → NVVM (CUDA) / ROCDL (ROCm) / SPIR-V (Vulkan)
5. **Linking**: Host code + GPU kernels → single binary

### Optimization Pipeline

Axon applies GPU-specific optimizations:

- **Kernel fusion** — combine adjacent operations into single kernels
- **Memory coalescing** — optimize memory access patterns
- **Shared memory tiling** — tile matrix multiplications for cache efficiency
- **Async transfers** — overlap computation with host↔device transfers

---

## Multi-GPU Programming

### Selecting a Device

```axon
use std.device.{Device, cuda};

fn main() {
    val dev0 = cuda(0);   // first GPU
    val dev1 = cuda(1);   // second GPU

    val a = randn([1024, 1024]).to_device(dev0);
    val b = randn([1024, 1024]).to_device(dev1);
}
```

### Data Parallelism

Split batches across GPUs:

```axon
fn train_multi_gpu(model: &mut NeuralNet, data: &DataLoader) {
    val devices = [cuda(0), cuda(1)];

    for batch in data {
        val (inputs, targets) = batch;

        // Split batch across devices
        val chunks = inputs.chunk(devices.len(), dim: 0);

        var losses = Vec.new();
        for i in 0..devices.len() {
            val chunk = chunks[i].to_device(devices[i]);
            val pred = model.forward(chunk);
            val loss = cross_entropy(pred, targets);
            losses.push(loss);
        }

        // Aggregate gradients
        val total_loss = losses.sum();
        total_loss.backward();
    }
}
```

### Device Query

```axon
use std.device;

fn main() {
    val count = device.gpu_count();
    println("Available GPUs: {}", count);

    for i in 0..count {
        val dev = device.cuda(i);
        println("  GPU {}: {} ({}MB)", i, dev.name(), dev.memory_mb());
    }
}
```

---

## Complete Example: GPU Matrix Multiplication

```axon
use std.device.cuda;

@gpu
fn matmul_gpu(
    a: Tensor<Float32, [?, ?]>,
    b: Tensor<Float32, [?, ?]>,
): Tensor<Float32, [?, ?]> {
    a @ b
}

fn main() {
    val size = 2048;

    // Create tensors on CPU
    val a = randn([size, size]);
    val b = randn([size, size]);

    // Transfer to GPU
    val ga = a.to_gpu();
    val gb = b.to_gpu();

    // GPU matrix multiply
    val gc = matmul_gpu(ga, gb);

    // Get result
    val c = gc.to_cpu();
    println("Result shape: {}", c.shape);
    println("Result[0][0]: {}", c[0][0]);
}
```

Compile and run:

```bash
axonc build matmul.axon --gpu cuda -O 3 -o matmul
./matmul
# Result shape: [2048, 2048]
# Result[0][0]: 12.3456
```

---

## Best Practices

1. **Minimize transfers** — keep data on GPU as long as possible
2. **Batch operations** — GPU shines with large, parallel workloads
3. **Use `@gpu` functions** — let the compiler handle kernel generation
4. **Profile first** — not everything benefits from GPU acceleration
5. **Clone before transfer** if you need the CPU copy

---

## See Also

- [Tensor Programming](tensors.md) — tensor types, shapes, and operations
- [Ownership & Borrowing](ownership-borrowing.md) — device-aware borrowing rules
- [Tutorial: MNIST Classifier](../tutorial/03-mnist-classifier.md) — GPU training example
- [CLI Reference](../reference/cli-reference.md) — `--gpu` build flag details
