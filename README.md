<div align="center">

# 🔷 Axon

**The ML/AI-first systems programming language**

_Compile-time tensor shapes · Ownership-based safety · Native GPU execution_

[![Build Status](https://img.shields.io/badge/build-passing-brightgreen)]()
[![Tests](https://img.shields.io/badge/tests-1034%20passing-brightgreen)]()
[![License](https://img.shields.io/badge/license-MIT-blue)]()
[![Version](https://img.shields.io/badge/version-0.1.0--alpha.1-orange)]()
[![Docs](https://img.shields.io/badge/docs-online-blue)](https://sufficientdaikon.github.io/Axon/documentation.html)

</div>

---

## What is Axon?

Axon is a compiled, statically-typed language purpose-built for machine learning
and AI. It combines Rust's memory safety with PyTorch's ergonomics and adds
**compile-time tensor shape checking**, so dimension mismatches are caught before
your code ever runs.

```axon
use std.nn.{Linear, Module};
use std.optim.Adam;
use std.loss.cross_entropy;

model Classifier {
    fc1: Linear<784, 256>,
    fc2: Linear<256, 10>,
}

extend Module for Classifier {
    fn forward(&self, x: Tensor<Float32, [?, 784]>): Tensor<Float32, [?, 10]> {
        val h = relu(self.fc1.forward(x));
        self.fc2.forward(h)
    }
}

fn main() {
    var net = Classifier { fc1: Linear.new(), fc2: Linear.new() };
    var opt = Adam.new(net.parameters(), lr: 0.001);

    for epoch in 0..10 {
        for (images, labels) in &train_loader {
            val loss = cross_entropy(net.forward(images), labels);
            loss.backward();
            opt.step();
            opt.zero_grad();
        }
    }
}
```

---

## Feature Highlights

| Feature                    | Description                                                     |
| -------------------------- | --------------------------------------------------------------- |
| 🔷 **First-class tensors** | `Tensor<Float32, [?, 784]>` — shapes verified at compile time   |
| 🔒 **Memory safety**       | Ownership + borrowing — no GC, no data races                    |
| 🧠 **Built-in autograd**   | Reverse-mode automatic differentiation                          |
| 🚀 **Native GPU**          | `@gpu` functions compiled via MLIR to CUDA/ROCm/Vulkan          |
| 📐 **Shape checker**       | Matmul, broadcast, reshape errors caught before runtime         |
| 🔮 **Type inference**      | Hindley-Milner — write less, know more                          |
| 📦 **Package manager**     | `axonc pkg` — create, build, test, publish                      |
| 🛠️ **Rich tooling**        | LSP, formatter, linter, REPL, doc generator, VS Code extension  |
| 🏗️ **ML framework**        | Neural network layers, optimizers, loss functions, data loading |

---

## Quick Start

### Install

```bash
# From source
git clone https://github.com/axon-lang/axon.git
cd axon
cargo build --release
```

### Hello, World!

```axon
fn main() {
    println("Hello, Axon!");
}
```

```bash
axonc build hello.axon -o hello
./hello
```

### Hello, Tensor!

```axon
fn main() {
    val A: Tensor<Float32, [2, 3]> = randn([2, 3]);
    val B: Tensor<Float32, [3, 4]> = randn([3, 4]);
    val C = A @ B;   // Tensor<Float32, [2, 4]> — shape checked!
    println("{}", C);
}
```

---

## Project Structure

```
axon/
├── src/                    # Compiler source (Rust)
│   ├── lexer.rs            # Lexer
│   ├── parser.rs           # Parser
│   ├── typeck.rs           # Type checker (HM inference)
│   ├── shapes.rs           # Shape checker
│   ├── borrow.rs           # Borrow checker
│   ├── mir.rs              # Mid-level IR
│   ├── codegen/            # LLVM + MLIR backends
│   ├── stdlib/             # Standard library (200+ functions)
│   ├── lsp/                # Language server
│   ├── pkg/                # Package manager
│   ├── fmt.rs              # Formatter
│   ├── lint.rs             # Linter
│   ├── repl.rs             # REPL
│   └── doc.rs              # Doc generator
├── stdlib/                 # Axon source stubs (.axon)
├── tests/                  # 1,034 tests
├── editors/vscode/         # VS Code extension
├── benches/                # Benchmarks
├── fuzz/                   # Fuzz testing
└── docs/                   # Documentation
    ├── guide/              # Language guides
    ├── tutorial/           # Step-by-step tutorials
    ├── reference/          # CLI and error reference
    ├── migration/          # Python/PyTorch migration guides
    └── internals/          # Compiler internals
```

---

## Building from Source

### Prerequisites

- [Rust](https://rustup.rs/) 1.75+ (stable)
- Git

### Build

```bash
git clone https://github.com/axon-lang/axon.git
cd axon
cargo build --release
```

The compiler binary is at `target/release/axonc`.

### Verify

```bash
./target/release/axonc --version
# axonc 0.1.0
```

---

## Running Tests

```bash
# Full test suite (1,034 tests)
cargo test

# Specific phases
cargo test --lib lexer           # Lexer tests
cargo test --lib parser          # Parser tests
cargo test --test type_tests     # Type checker tests
cargo test --test codegen_tests  # Code generation tests
cargo test --test stdlib_tests   # Standard library tests
cargo test --test ai_framework_tests  # ML framework tests
cargo test --test tooling_tests  # Tooling tests
```

---

## Documentation

**[Read the full documentation online](https://sufficientdaikon.github.io/Axon/documentation.html)**

| Document                                                   | Description                          |
| ---------------------------------------------------------- | ------------------------------------ |
| [Getting Started](docs/guide/getting-started.md)           | Installation and first project       |
| [Language Tour](docs/guide/language-tour.md)               | Quick syntax overview                |
| [Tensor Guide](docs/guide/tensors.md)                      | Tensor types, shapes, and operations |
| [GPU Programming](docs/guide/gpu-programming.md)           | Native GPU support                   |
| [Ownership & Borrowing](docs/guide/ownership-borrowing.md) | Memory safety model                  |
| [Error Handling](docs/guide/error-handling.md)             | Option, Result, and `?`              |
| [Modules & Packages](docs/guide/modules-packages.md)       | Module system and `Axon.toml`        |
| [CLI Reference](docs/reference/cli-reference.md)           | Complete `axonc` command reference   |
| [Compiler Errors](docs/reference/compiler-errors.md)       | All error codes explained            |
| [From Python](docs/migration/from-python.md)               | Python → Axon migration guide        |
| [From PyTorch](docs/migration/from-pytorch.md)             | PyTorch → Axon migration guide       |

### Tutorials

1. [Hello, Tensor!](docs/tutorial/01-hello-tensor.md) — tensor basics
2. [Linear Regression](docs/tutorial/02-linear-regression.md) — autograd from scratch
3. [MNIST Classifier](docs/tutorial/03-mnist-classifier.md) — CNN training
4. [Transformer](docs/tutorial/04-transformer.md) — build a transformer encoder

---

## Contributing

We welcome contributions! See the [Contributing Guide](docs/internals/contributing.md)
for details on:

- Building from source
- Running and writing tests
- Code style guidelines
- Pull request process

```bash
# Quick start for contributors
git clone https://github.com/axon-lang/axon.git
cd axon
cargo build
cargo test
```

---

## License

This project is licensed under the MIT License. See [LICENSE](LICENSE) for details.
