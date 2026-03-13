# Modules and Packages

Axon provides a hierarchical module system for organizing code and a built-in
package manager for dependency management.

---

## Modules

### Declaring Modules

Use `mod` to define a module inline:

```axon
mod math {
    pub fn square(x: Float64): Float64 {
        x * x
    }

    pub fn cube(x: Float64): Float64 {
        x * x * x
    }

    fn helper() {
        // private — not visible outside `math`
    }
}

fn main() {
    println("{}", math.square(5.0));   // 25.0
    println("{}", math.cube(3.0));     // 27.0
    // math.helper();                  // ERROR: `helper` is private
}
```

### File-Based Modules

For larger projects, modules map to files:

```
my_project/
├── Axon.toml
└── src/
    ├── main.axon         # crate root
    ├── model.axon         # mod model
    ├── data/
    │   ├── mod.axon       # mod data (directory module)
    │   ├── loader.axon    # mod data.loader
    │   └── transform.axon # mod data.transform
    └── utils.axon         # mod utils
```

In `src/main.axon`:

```axon
mod model;
mod data;
mod utils;

fn main() {
    val net = model.build_network();
    val loader = data.loader.DataLoader.new("train.csv");
    utils.log("Training started");
}
```

In `src/data/mod.axon`:

```axon
pub mod loader;
pub mod transform;
```

---

## Visibility (`pub`)

Items are **private by default**. Use `pub` to make them visible outside
their module:

```axon
mod network {
    pub model Layer {
        pub size: Int32,        // public field
        weights: Vec<Float32>,  // private field
    }

    pub fn new_layer(size: Int32): Layer {
        Layer {
            size,
            weights: Vec.new(),
        }
    }

    extend Layer {
        pub fn forward(&self, input: &Vec<Float32>): Vec<Float32> {
            // public method
        }

        fn init_weights(&mut self) {
            // private method — internal use only
        }
    }
}
```

### Visibility Rules

| Declaration      | Visible To                             |
| ---------------- | -------------------------------------- |
| `fn foo()`       | Current module only                    |
| `pub fn foo()`   | Parent module and beyond               |
| `pub model Foo`  | Public type, fields default to private |
| `pub field: T`   | Public field on a public model         |

---

## Importing with `use`

Bring items into scope with `use`:

```axon
use std.collections.HashMap;
use std.io.{File, Read, Write};

fn main() {
    var map = HashMap.new();
    map.insert("key", 42);
}
```

### Path Forms

```axon
// Absolute path
use std.math.sin;

// Nested imports
use std.collections.{Vec, HashMap, HashSet};

// Wildcard import (use sparingly)
use std.prelude.*;

// Aliased import
use std.collections.HashMap as Map;
```

### Re-exports

Modules can re-export items for a cleaner public API:

```axon
mod internal {
    pub fn core_function(): Int32 { 42 }
}

// Re-export so users see `my_lib.core_function`
pub use internal.core_function;
```

---

## The `Axon.toml` Manifest

Every Axon project has an `Axon.toml` at its root:

```toml
[package]
name = "my_ml_project"
version = "0.2.1"
edition = "2026"
authors = ["Jane Doe <jane@example.com>"]
description = "A neural network toolkit"
license = "MIT"
repository = "https://github.com/jane/my_ml_project"

[dependencies]
axon-vision = "0.3.0"
axon-nlp = { version = "0.1.0", features = ["transformers"] }
axon-data = { git = "https://github.com/axon-lang/axon-data.git", branch = "main" }

[dev-dependencies]
axon-test = "0.1.0"

[build]
opt-level = 2
gpu = "cuda"
```

### Manifest Fields

| Section              | Field                     | Description                          |
| -------------------- | ------------------------- | ------------------------------------ |
| `[package]`          | `name`                    | Package name (lowercase, hyphens)    |
|                      | `version`                 | Semantic version (MAJOR.MINOR.PATCH) |
|                      | `edition`                 | Language edition year                |
|                      | `authors`                 | List of authors                      |
|                      | `description`             | One-line description                 |
|                      | `license`                 | SPDX license identifier              |
| `[dependencies]`     | `name = "ver"`            | Registry dependency                  |
|                      | `name = { git = "..." }`  | Git dependency                       |
|                      | `name = { path = "..." }` | Local path dependency                |
| `[dev-dependencies]` |                           | Dependencies for tests only          |
| `[build]`            | `opt-level`               | Default optimization level           |
|                      | `gpu`                     | Default GPU target                   |

---

## Dependencies

### Adding Dependencies

```bash
# From registry
axonc pkg add axon-vision
axonc pkg add axon-nlp --version 0.2.0

# Remove a dependency
axonc pkg remove axon-vision
```

### Using Dependencies

After adding a dependency, import it like any module:

```axon
use axon_vision.transforms.{resize, normalize};
use axon_nlp.tokenizer.BPETokenizer;

fn preprocess(image: Tensor<Float32, [?, ?, 3]>): Tensor<Float32, [?, 224, 224, 3]> {
    val resized = resize(image, [224, 224]);
    normalize(resized, mean: [0.485, 0.456, 0.406], std: [0.229, 0.224, 0.225])
}
```

### Lock File

Running `axonc pkg build` generates an `Axon.lock` file that pins exact
dependency versions for reproducible builds. Commit this file to source control.

---

## Package Manager Commands

| Command                  | Description                     |
| ------------------------ | ------------------------------- |
| `axonc pkg new <name>`   | Create a new project            |
| `axonc pkg init`         | Initialize in current directory |
| `axonc pkg build`        | Build the project               |
| `axonc pkg run`          | Build and run                   |
| `axonc pkg test`         | Run tests                       |
| `axonc pkg add <pkg>`    | Add a dependency                |
| `axonc pkg remove <pkg>` | Remove a dependency             |
| `axonc pkg clean`        | Clean build artifacts           |
| `axonc pkg fmt`          | Format all source files         |
| `axonc pkg lint`         | Lint all source files           |

---

## Standard Library Modules

Axon ships with a comprehensive standard library:

| Module             | Contents                                                   |
| ------------------ | ---------------------------------------------------------- |
| `std.prelude`      | Auto-imported basics (println, Clone, Copy, Display, etc.) |
| `std.collections`  | `Vec`, `HashMap`, `HashSet`, `Option`, `Result`            |
| `std.string`       | `String` with UTF-8 operations                             |
| `std.io`           | `File`, `Read`, `Write`, formatting                        |
| `std.math`         | Trigonometry, logarithms, constants                        |
| `std.tensor`       | Tensor creation, shape ops, reductions, linalg             |
| `std.nn`           | Neural network layers (Linear, Conv2d, LSTM, Transformer)  |
| `std.autograd`     | Automatic differentiation                                  |
| `std.optim`        | Optimizers (SGD, Adam, AdamW) + LR schedulers              |
| `std.loss`         | Loss functions (CrossEntropy, MSE, BCE)                    |
| `std.data`         | DataLoader, CSV/JSON loading                               |
| `std.metrics`      | Accuracy, precision, recall, F1, ROC-AUC                   |
| `std.transforms`   | Image and text preprocessing                               |
| `std.sync`         | `Mutex`, `RwLock`, `Arc`, `Channel`                        |
| `std.thread`       | `spawn`, `JoinHandle`                                      |
| `std.device`       | Device abstraction, GPU query                              |
| `std.random`       | Random number generation                                   |
| `std.ops`          | Operator traits (Add, Mul, MatMul, Index)                  |
| `std.convert`      | `From`, `Into`, `TryFrom`, `TryInto`                       |

---

## Project Organization Best Practices

```
my_ml_project/
├── Axon.toml
├── Axon.lock
├── src/
│   ├── main.axon           # entry point
│   ├── lib.axon             # library root (if building a library)
│   ├── model/
│   │   ├── mod.axon
│   │   ├── encoder.axon
│   │   └── decoder.axon
│   ├── data/
│   │   ├── mod.axon
│   │   └── preprocessing.axon
│   └── utils.axon
├── tests/
│   ├── test_model.axon
│   └── test_data.axon
├── benches/
│   └── bench_model.axon
└── examples/
    └── inference.axon
```

---

## See Also

- [Getting Started](getting-started.md) — project creation walkthrough
- [CLI Reference](../reference/cli-reference.md) — full `axonc pkg` documentation
- [Contributing](../internals/contributing.md) — how to contribute packages
