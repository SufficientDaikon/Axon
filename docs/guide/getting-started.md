# Getting Started with Axon

Welcome to **Axon** — the ML/AI-first systems programming language that combines
the safety of Rust, the ergonomics of Python, and first-class support for tensors,
automatic differentiation, and GPU computing.

## What is Axon?

Axon is a compiled, statically-typed language purpose-built for machine learning
and AI workloads. It provides:

- **First-class tensor types** with compile-time shape checking
- **Ownership-based memory safety** — no garbage collector, no data races
- **Automatic differentiation** (reverse-mode autograd)
- **Native GPU execution** via CUDA, ROCm, and Vulkan backends
- **Hindley-Milner type inference** — write less, know more
- **A rich standard library** with neural network layers, optimizers, and data loading

Axon compiles to native code through LLVM and to GPU kernels through MLIR,
delivering performance on par with C++ while remaining approachable for ML researchers.

---

## Installation

### From Cargo (Recommended)

If you have [Rust](https://rustup.rs/) installed:

```bash
cargo install axonc
```

### Binary Download

Pre-built binaries are available for:

- **Linux** (x86_64, aarch64)
- **macOS** (x86_64, Apple Silicon)
- **Windows** (x86_64)

Download from the [releases page](https://github.com/axon-lang/axon/releases)
and add the binary to your `PATH`.

### From Source

```bash
git clone https://github.com/axon-lang/axon.git
cd axon
cargo build --release
# Binary is at target/release/axonc
```

Verify the installation:

```bash
axonc --version
# axonc 0.1.0
```

---

## Hello, World!

Create a file named `hello.axon`:

```axon
fn main() {
    println("Hello, Axon!");
}
```

Compile and run:

```bash
axonc build hello.axon -o hello
./hello
# Hello, Axon!
```

Or use the REPL for quick experimentation:

```bash
axonc repl
>>> println("Hello from the REPL!")
Hello from the REPL!
```

---

## Your First Project

Axon includes a built-in package manager. Create a new project:

```bash
axonc pkg new my_project
cd my_project
```

This generates the following structure:

```
my_project/
├── Axon.toml          # Project manifest
├── src/
│   └── main.axon      # Entry point
└── tests/
    └── test_main.axon  # Test file
```

The generated `Axon.toml`:

```toml
[package]
name = "my_project"
version = "0.1.0"
edition = "2026"

[dependencies]
```

The generated `src/main.axon`:

```axon
fn main() {
    println("Hello from my_project!");
}
```

Build and run the project:

```bash
axonc pkg build
axonc pkg run
# Hello from my_project!
```

---

## Compiling and Running

### Single File

```bash
# Parse and check for errors
axonc check hello.axon

# Build an optimized binary
axonc build hello.axon -O 3 -o hello

# Emit LLVM IR for inspection
axonc build hello.axon --emit-llvm
```

### Project-Based

```bash
axonc pkg build        # Build the project
axonc pkg run          # Build and run
axonc pkg test         # Run tests
axonc pkg fmt          # Format all source files
axonc pkg lint         # Lint all source files
```

### Optimization Levels

| Flag   | Description                                |
| ------ | ------------------------------------------ |
| `-O 0` | No optimization (default, fastest compile) |
| `-O 1` | Basic optimizations                        |
| `-O 2` | Standard optimizations                     |
| `-O 3` | Aggressive optimizations                   |

---

## Editor Setup

### VS Code (Recommended)

The official [Axon VS Code extension](../editors/vscode/) provides:

- Syntax highlighting for `.axon` files
- Real-time error diagnostics via the Axon LSP
- Go-to-definition, hover types, and find references
- Code completion with type-aware suggestions
- Inlay hints for inferred types
- Semantic token highlighting
- Code snippets for common patterns

**Install from the marketplace** or build from source:

```bash
cd editors/vscode
npm install
npm run build
```

### Other Editors

For any editor that supports the Language Server Protocol:

```bash
axonc lsp
```

This starts the Axon language server over stdio, compatible with Neovim
(via `nvim-lspconfig`), Emacs (via `lsp-mode`), Helix, Zed, and others.

---

## What's Next?

- [**Language Tour**](language-tour.md) — quick overview of all Axon syntax
- [**Tensor Programming**](tensors.md) — working with tensors and shapes
- [**Tutorials**](../tutorial/01-hello-tensor.md) — hands-on projects from simple to advanced
- [**CLI Reference**](../reference/cli-reference.md) — complete command reference
