# Axon Language — VS Code Extension

Language support for the **Axon** programming language, an ML/AI-first systems language.

## Features

- **Syntax highlighting** — Full TextMate grammar for Axon source files (`.axon`)
- **Code completion** — Keywords, types, and standard library functions
- **Hover information** — Type information for symbols
- **Go to definition** — Jump to symbol declarations
- **Document symbols** — Outline view of functions, structs, enums, and traits
- **Formatting** — Auto-format Axon code via the built-in formatter
- **Diagnostics** — Real-time type-checking errors and warnings
- **Snippets** — Common code patterns (fn, struct, enum, trait, model, tensor, etc.)

## Requirements

- The `axonc` compiler binary must be installed and available on your `PATH`, or configure `axon.lsp.path` in settings.

## Extension Settings

| Setting              | Default | Description               |
| -------------------- | ------- | ------------------------- |
| `axon.lsp.path`      | `axonc` | Path to the axonc binary  |
| `axon.format.onSave` | `true`  | Format Axon files on save |

## Getting Started

1. Install the extension
2. Ensure `axonc` is on your PATH (or set `axon.lsp.path`)
3. Open any `.axon` file — syntax highlighting and LSP features activate automatically

## Snippets

| Prefix   | Description          |
| -------- | -------------------- |
| `fn`     | Function declaration |
| `struct` | Struct definition    |
| `enum`   | Enum definition      |
| `trait`  | Trait definition     |
| `impl`   | Implementation block |
| `if`     | If-else expression   |
| `match`  | Match expression     |
| `for`    | For loop             |
| `while`  | While loop           |
| `let`    | Let binding          |
| `letmut` | Mutable let binding  |
| `test`   | Test function        |
| `main`   | Main function        |
| `tensor` | Tensor creation      |
| `model`  | Neural network model |

## Development

```bash
cd editors/vscode
npm install
npm run compile
```

To test locally, press `F5` in VS Code to launch an Extension Development Host.
