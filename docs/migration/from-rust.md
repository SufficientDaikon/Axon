# Migrating from Rust to Axon

Axon draws significant inspiration from Rust's syntax and safety model, but is
purpose-built for ML/AI workloads. If you know Rust, you'll feel at home quickly.
This guide covers the key differences.

## At a Glance

| Feature | Rust | Axon |
|---------|------|------|
| Ownership | Borrow checker | Simplified borrow checker |
| Generics | `<T: Trait>` | `<T: Trait>` (same syntax) |
| Tensors | External crate (ndarray) | First-class `Tensor<D, Shape>` |
| GPU | External crate (cuda-rs) | Built-in `@device` annotation |
| Macros | `macro_rules!` / proc macros | Not yet supported |
| Async | `async/await` | Not yet supported |
| Package manager | Cargo | `axonc pkg` (compatible workflow) |
| Strings | `String` / `&str` | `String` / `&str` (same model) |
| Error handling | `Result<T, E>` | `Result<T, E>` (same model) |

## Syntax Differences

### Function Declarations

```rust
// Rust
fn add(x: i32, y: i32) -> i32 {
    x + y
}
```

```axon
// Axon — explicit return required, types use PascalCase
fn add(x: Int32, y: Int32) -> Int32 {
    return x + y;
}
```

**Key differences:**
- Axon requires explicit `return` statements (no implicit last-expression return)
- Primitive types use PascalCase: `Int32`, `Int64`, `Float32`, `Float64`, `Bool`
- Semicolons are required on all statements

### Type Names

| Rust | Axon |
|------|------|
| `i32` | `Int32` |
| `i64` | `Int64` |
| `f32` | `Float32` |
| `f64` | `Float64` |
| `bool` | `Bool` |
| `String` | `String` |
| `Vec<T>` | `Vec<T>` |

### Variable Bindings

```rust
// Rust
let x = 5;          // immutable
let mut y = 10;     // mutable
```

```axon
// Axon — same syntax
let x = 5;
let mut y = 10;
```

## Ownership Model

Axon uses a **simplified** version of Rust's ownership model:

```axon
fn take_ownership(data: Tensor<Float32, [_]>) {
    // `data` is owned here — moved from caller
    println(data.sum());
}
// `data` is dropped here

fn borrow_data(data: &Tensor<Float32, [_]>) {
    // `data` is borrowed — caller keeps ownership
    println(data.sum());
}
```

**Simplifications vs Rust:**
- No lifetime annotations (`'a`) — lifetimes are inferred or scoped
- No `Rc<T>` / `Arc<T>` in user code — runtime reference counting where needed
- Borrow checker is less strict — focuses on preventing use-after-free and double-free

## First-Class Tensors

The biggest difference from Rust: tensors are a **built-in type** with shape tracking:

```axon
// Axon — tensors are first-class citizens
let x: Tensor<Float32, [3, 3]> = tensor([[1.0, 2.0, 3.0],
                                          [4.0, 5.0, 6.0],
                                          [7.0, 8.0, 9.0]]);

// Shape-checked matrix multiply (compiler verifies dimensions)
let y: Tensor<Float32, [3, 1]> = tensor([[1.0], [0.0], [1.0]]);
let result = x @ y;  // result: Tensor<Float32, [3, 1]>
```

In Rust, you'd need an external crate:

```rust
// Rust — requires ndarray or nalgebra
use ndarray::Array2;
let x = Array2::<f32>::from_shape_vec((3, 3), vec![...]).unwrap();
```

### Dynamic Shapes

Use `_` for dimensions known only at runtime:

```axon
fn process_batch(input: Tensor<Float32, [_, 784]>) -> Tensor<Float32, [_, 10]> {
    // batch size (_) is dynamic, feature dimensions are static
    return model.forward(input);
}
```

## GPU Support

Axon has built-in device management — no external CUDA bindings needed:

```axon
// Move tensor to GPU
let gpu_data = data.to(Device::GPU(0));

// GPU-annotated functions
@device(GPU)
fn matmul_kernel(a: Tensor<Float32, [_, _]>, b: Tensor<Float32, [_, _]>) -> Tensor<Float32, [_, _]> {
    return a @ b;
}
```

In Rust, GPU support requires unsafe FFI to CUDA/OpenCL libraries.

## What's Not (Yet) in Axon

Coming from Rust, you'll miss these features (planned for future releases):

- **Traits/Interfaces** — Use structural typing for now
- **Macros** — No compile-time metaprogramming yet
- **Async/Await** — No async runtime yet
- **Closures** — Limited closure support (planned)
- **Pattern matching in let** — `let (a, b) = tuple;` not yet supported
- **Crate ecosystem** — Axon's package registry is young

## Build & Run Comparison

```bash
# Rust
cargo build
cargo run
cargo test

# Axon
axonc build main.axon
./main
axonc pkg test
```

## Migration Strategy

1. **Start with compute kernels** — Port tensor-heavy code first (biggest Axon advantage)
2. **Keep Rust for infrastructure** — Build systems, CLI tools, networking stay in Rust
3. **Use Axon for models** — Training loops, inference pipelines, data processing
4. **Interop via C ABI** — Axon and Rust can call each other through C FFI

## Further Reading

- [Tutorial 01: Hello Tensor](../tutorial/01-hello-tensor.md)
- [Migration from Python](from-python.md)
- [Migration from PyTorch](from-pytorch.md)
