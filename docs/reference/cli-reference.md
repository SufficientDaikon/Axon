# CLI Reference

Complete reference for the `axonc` command-line compiler.

---

## Synopsis

```
axonc <COMMAND> [OPTIONS]
axonc --help
axonc --version
```

---

## Commands

### `axonc lex`

Tokenize an Axon source file and print the token stream.

```bash
axonc lex <FILE>
```

**Arguments:**

- `<FILE>` — Path to the `.axon` source file

**Example:**

```bash
axonc lex hello.axon
# Token(Fn, 1:1)
# Token(Identifier("main"), 1:4)
# Token(LeftParen, 1:8)
# Token(RightParen, 1:9)
# Token(LeftBrace, 1:11)
# ...
```

---

### `axonc parse`

Parse an Axon source file and print the AST.

```bash
axonc parse <FILE> [OPTIONS]
```

**Arguments:**

- `<FILE>` — Path to the `.axon` source file

**Options:**
| Flag | Description |
|------|-------------|
| `--error-format <FORMAT>` | Output format: `human` (default) or `json` |
| `--errors-only` | Print only errors, suppress AST output |

**Example:**

```bash
axonc parse hello.axon
axonc parse hello.axon --error-format=json
axonc parse hello.axon --errors-only
```

---

### `axonc check`

Type-check an Axon source file. Runs the full frontend pipeline: lex → parse →
name resolution → type inference → shape checking → borrow checking.

```bash
axonc check <FILE> [OPTIONS]
```

**Arguments:**

- `<FILE>` — Path to the `.axon` source file

**Options:**
| Flag | Description |
|------|-------------|
| `--error-format <FORMAT>` | Output format: `human` (default) or `json` |
| `--emit-tast` | Emit the typed AST as JSON |

**Example:**

```bash
axonc check model.axon
axonc check model.axon --emit-tast
axonc check model.axon --error-format=json
```

**Exit codes:**

- `0` — No errors
- `1` — One or more errors found

---

### `axonc build`

Compile an Axon source file to a native binary.

```bash
axonc build <FILE> [OPTIONS]
```

**Arguments:**

- `<FILE>` — Path to the `.axon` source file

**Options:**
| Flag | Description | Default |
|------|-------------|---------|
| `-o, --output <PATH>` | Output file path | Input filename without extension |
| `-O, --opt-level <LEVEL>` | Optimization level: `0`, `1`, `2`, `3` | `0` |
| `--emit-llvm` | Emit LLVM IR text instead of compiling | — |
| `--emit-mir` | Emit Axon MIR (debug intermediate representation) | — |
| `--emit-obj` | Emit object file (`.o`) instead of binary | — |
| `--gpu <TARGET>` | GPU target: `none`, `cuda`, `rocm`, `vulkan` | `none` |
| `--error-format <FORMAT>` | Output format: `human` or `json` | `human` |

**Examples:**

```bash
# Basic compilation
axonc build hello.axon

# Optimized build with custom output
axonc build model.axon -O 3 -o model

# Emit LLVM IR for inspection
axonc build model.axon --emit-llvm -o model.ll

# GPU compilation for NVIDIA
axonc build model.axon --gpu cuda -O 3

# AMD GPU target
axonc build model.axon --gpu rocm -O 2

# Emit object file for linking
axonc build model.axon --emit-obj -o model.o
```

**Optimization levels:**
| Level | Description |
|-------|-------------|
| `-O 0` | No optimization — fastest compile, easiest to debug |
| `-O 1` | Basic optimizations (constant folding, dead code elimination) |
| `-O 2` | Standard optimizations (inlining, loop unrolling, vectorization) |
| `-O 3` | Aggressive optimizations (LTO, auto-vectorization, FMA) |

---

### `axonc fmt`

Format an Axon source file according to the standard style.

```bash
axonc fmt <FILE>
```

**Arguments:**

- `<FILE>` — Path to the `.axon` source file

The formatter modifies the file in place. Formatting is idempotent — running
it twice produces the same output.

**Example:**

```bash
axonc fmt src/main.axon
```

---

### `axonc lint`

Run the linter on an Axon source file. Reports style and best-practice warnings.

```bash
axonc lint <FILE>
```

**Arguments:**

- `<FILE>` — Path to the `.axon` source file

**Lint rules:**
| Code | Rule |
|------|------|
| W5001 | Unused variable |
| W5002 | Unused import |
| W5003 | Dead code |
| W5004 | Unnecessary mutability |
| W5005 | Shadowed variable |
| W5006 | Naming convention violation |
| W5007 | Redundant type annotation |
| W5008 | Missing documentation on public items |

See [Compiler Errors](compiler-errors.md) for details on each warning.

**Example:**

```bash
axonc lint src/main.axon
# warning[W5001]: unused variable `temp`
#   --> src/main.axon:12:9
```

---

### `axonc repl`

Start the interactive Read-Eval-Print Loop.

```bash
axonc repl
```

**REPL Commands:**
| Command | Description |
|---------|-------------|
| `:type <expr>` | Show the type of an expression |
| `:ast <expr>` | Show the AST for an expression |
| `:load <file>` | Load and evaluate an Axon source file |
| `:save <file>` | Save REPL history to a file |
| `:clear` | Clear the REPL state |
| `:help` | Show help |
| `:quit` | Exit the REPL |

**Example:**

```
$ axonc repl
Axon REPL v0.1.0 — type :help for commands

>>> val x = 42
>>> x * 2
84
>>> :type x
Int32
>>> val t = randn([3, 3])
>>> t.shape
[3, 3]
>>> :quit
```

---

### `axonc doc`

Generate HTML documentation from doc comments in Axon source files.

```bash
axonc doc <FILE> [OPTIONS]
```

**Arguments:**

- `<FILE>` — Path to the `.axon` source file

**Options:**
| Flag | Description |
|------|-------------|
| `-o, --output <PATH>` | Output file path (default: stdout) |

**Example:**

```bash
axonc doc src/lib.axon -o docs/api.html
```

---

### `axonc lsp`

Start the Axon Language Server Protocol server over stdio.

```bash
axonc lsp
```

The LSP server provides:

- Real-time diagnostics
- Go-to-definition
- Hover (type information)
- Code completion
- Find references
- Rename symbol
- Signature help
- Inlay hints
- Semantic tokens

Configure your editor to use `axonc lsp` as the language server for `.axon` files.

---

### `axonc pkg`

Package manager commands for Axon projects.

#### `axonc pkg new <NAME>`

Create a new Axon project with standard directory structure.

```bash
axonc pkg new my_project
# Created project `my_project`
```

Generated structure:

```
my_project/
├── Axon.toml
├── src/
│   └── main.axon
└── tests/
    └── test_main.axon
```

#### `axonc pkg init`

Initialize an Axon project in the current directory.

```bash
mkdir my_project && cd my_project
axonc pkg init
```

#### `axonc pkg build`

Build the current project (reads `Axon.toml`).

```bash
axonc pkg build
```

#### `axonc pkg run`

Build and run the project.

```bash
axonc pkg run
```

#### `axonc pkg test`

Run all tests in the `tests/` directory.

```bash
axonc pkg test
```

#### `axonc pkg add <PACKAGE>`

Add a dependency to `Axon.toml`.

```bash
axonc pkg add axon-vision
axonc pkg add axon-nlp --version 0.2.0
```

#### `axonc pkg remove <PACKAGE>`

Remove a dependency from `Axon.toml`.

```bash
axonc pkg remove axon-vision
```

#### `axonc pkg clean`

Remove build artifacts.

```bash
axonc pkg clean
```

#### `axonc pkg fmt`

Format all `.axon` source files in the project.

```bash
axonc pkg fmt
```

#### `axonc pkg lint`

Lint all `.axon` source files in the project.

```bash
axonc pkg lint
```

---

## Global Options

| Flag        | Description                   |
| ----------- | ----------------------------- |
| `--help`    | Print help information        |
| `--version` | Print version (`axonc 0.1.0`) |

---

## Environment Variables

| Variable    | Description                                      |
| ----------- | ------------------------------------------------ |
| `AXON_HOME` | Axon installation directory                      |
| `AXON_PATH` | Additional module search paths (colon-separated) |

---

## See Also

- [Getting Started](../guide/getting-started.md) — installation and first project
- [Compiler Errors](compiler-errors.md) — all error codes explained
- [Modules & Packages](../guide/modules-packages.md) — `Axon.toml` reference
