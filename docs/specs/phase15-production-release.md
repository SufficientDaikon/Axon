# Phase 15: Production Polish (v1.0.0)

## Summary

Phase 15 is the final phase before public release.  Every rough edge is
smoothed, documentation is rewritten with real working examples, installers
are built for every platform, and the language gets a website.

After this phase, a new user can:
1. Install Axon with a single command
2. Follow a tutorial and train a model
3. Get editor support with real-time errors
4. Publish packages to a registry
5. Deploy models via ONNX export

## Prerequisites

| Requirement | Provided by |
|-------------|-------------|
| All models train correctly | Phase 14 |
| GPU (CUDA + ROCm) working | Phase 12 |
| Autograd, optimizers, data loading | Phase 11, 13 |
| Tooling (formatter, linter, LSP, REPL) | Phase 7 |
| Basic docs and CI/CD exist | Phase 8 |

## Architecture

No new architecture.  This phase is about quality, documentation,
packaging, and community readiness.

---

## Task List

### Sub-phase 15a: Error Message Quality (T459–T463)

#### T459 — Audit all compiler errors
- **Files**: `src/error.rs`, `src/typeck.rs`, `src/borrow.rs`, `src/shapes.rs`
- **Description**: Review every error code (E0xxx–E9xxx).  For each:
  - Clear, jargon-free primary message
  - Span points to the exact problematic token
  - "help" suggestion with actionable fix
  - "note" with additional context where useful
  Example of good error:
  ```
  error[E3002]: tensor shape mismatch in matrix multiply
   --> model.axon:5:12
    |
  5 |     let y = x @ w;
    |             ^^^^^
    |  left shape: [32, 784]
    |  right shape: [256, 10]
    |  note: inner dimensions must match (784 ≠ 256)
    |  help: reshape w to [784, 10]
  ```
- **Acceptance**: Every error has message + span + help.  0 "internal error" messages.

#### T460 — Audit all runtime panics
- **Files**: `runtime/axon_*.c`
- **Description**: Every `axon_panic()` call should include:
  - Error code
  - Descriptive message
  - Relevant values (shapes, indices, types)
  - Source location if available (file:line from debug info)
- **Acceptance**: All panics are descriptive, no bare "assertion failed".

#### T461 — Colored error output
- **Files**: `src/error.rs`
- **Description**: Terminal colors for errors (red), warnings (yellow),
  help (cyan), notes (blue).  Detect terminal capability (isatty).
  `--no-color` flag to disable.
- **Acceptance**: Errors look good in terminals, don't break in pipes.

#### T462 — Error index webpage
- **File**: `docs/reference/compiler-errors.md` (rewrite)
- **Description**: Comprehensive error reference.  For each error code:
  - What it means
  - Common causes
  - How to fix it
  - Code example that triggers it
  Auto-generated from error definitions in source.
- **Acceptance**: Every error code has documentation.

#### T463 — Friendly messages for common mistakes
- **Files**: `src/typeck.rs`, `src/parser.rs`
- **Description**: Detect common beginner mistakes and give targeted help:
  - Missing semicolons → "did you mean to add `;`?"
  - Python-style `def` → "Axon uses `fn`, not `def`"
  - `print()` without import → "use `println` from prelude"
  - `=` in if condition → "did you mean `==`?"
  - Mismatched brackets → show where the opening bracket was
- **Acceptance**: 10+ friendly error patterns implemented.

### Sub-phase 15b: Documentation Rewrite (T464–T470)

#### T464 — Rewrite Getting Started guide
- **File**: `docs/guide/getting-started.md`
- **Description**: Full rewrite with REAL commands that work:
  1. Install Axon (link to installer)
  2. Create first project (`axon pkg new hello`)
  3. Write hello world
  4. Compile and run (`axonc build hello.axon && ./hello`)
  5. Write a simple tensor program
  6. Train a tiny model
  Every code block must be tested and known to work.
- **Acceptance**: A new user can follow every step without errors.

#### T465 — Rewrite language tour
- **File**: `docs/guide/language-tour.md`
- **Description**: Update with working examples for every feature:
  variables, types, functions, structs, enums, pattern matching,
  traits, generics, ownership, tensors, error handling.
  Each example compiles and runs.
- **Acceptance**: All code examples pass `axonc check`.

#### T466 — Rewrite ML tutorials
- **Files**: `docs/tutorial/01-hello-tensor.md` through `04-transformer.md`
- **Description**: Replace stub examples with the REAL working examples
  from `examples/`.  Show actual output, explain each step.
  Include screenshots of training progress.
- **Acceptance**: Each tutorial trains to expected accuracy.

#### T467 — API reference generation
- **File**: `src/doc.rs` (enhance)
- **Description**: Enhance doc generator to produce comprehensive API
  reference for all stdlib modules: functions, types, traits with
  signatures, descriptions, examples.  Output as HTML + searchable.
- **Acceptance**: `axonc doc --stdlib` produces browsable HTML.

#### T468 — Write "Axon for Python developers" guide
- **File**: `docs/migration/from-python.md` (rewrite)
- **Description**: Side-by-side Axon vs Python comparisons.
  Cover: syntax, type system, ownership (vs GC), tensors (vs NumPy),
  training (vs PyTorch), packaging (vs pip).
  Working code examples on both sides.
- **Acceptance**: Python developer can map their knowledge to Axon.

#### T469 — Write "Axon for Rust developers" guide
- **File**: `docs/migration/from-rust.md`
- **Description**: What's similar (ownership, traits, pattern matching)
  and what's different (tensor types, autograd, GPU, ML focus).
- **Acceptance**: Rust developer understands Axon's additions.

#### T470 — Changelog and release notes
- **File**: `CHANGELOG.md` (rewrite)
- **Description**: Professional changelog for v1.0.0.
  List all features, all supported platforms, all known limitations.
  Acknowledgments section.
- **Acceptance**: Changelog is complete and professional.

### Sub-phase 15c: VS Code Extension Polish (T471–T475)

#### T471 — Publish to VS Code Marketplace
- **File**: `editors/vscode/`
- **Description**: Package extension with `vsce package`.  Set up
  publisher account.  Publish to marketplace.  Include icon, README,
  screenshots, demo GIF.
- **Acceptance**: Extension installable via VS Code marketplace search.

#### T472 — Real-time error diagnostics
- **File**: `src/lsp/handlers.rs`
- **Description**: LSP `textDocument/didChange` triggers incremental
  re-check.  Errors appear as red squiggles within 200ms of typing.
  Clear errors when fixed.
- **Acceptance**: Typing a type error shows squiggle within 200ms.

#### T473 — Go-to-definition and hover
- **File**: `src/lsp/handlers.rs`
- **Description**: `textDocument/definition` — jump to function/type
  definition.  `textDocument/hover` — show type signature and doc
  comment on hover.  Work for stdlib and user code.
- **Acceptance**: Ctrl+click on a function jumps to its definition.

#### T474 — Autocomplete
- **File**: `src/lsp/handlers.rs`
- **Description**: `textDocument/completion` — suggest:
  - Local variables and function params
  - Stdlib functions after `Tensor::`
  - Struct fields after `.`
  - Method completions with signatures
  Trigger characters: `.`, `::`, `(`
- **Acceptance**: Typing `Tensor::` shows tensor methods.

#### T475 — Code actions and quick fixes
- **File**: `src/lsp/handlers.rs`
- **Description**: Quick fixes for common errors:
  - "Add missing import" for unresolved names
  - "Change type to X" for type mismatches
  - "Add `.clone()`" for move errors
  - "Remove unused variable" for warnings
- **Acceptance**: Light bulb icon appears on errors with actionable fixes.

### Sub-phase 15d: Platform Installers (T476–T481)

#### T476 — Cross-compile release binaries
- **File**: `.github/workflows/release.yml`
- **Description**: GitHub Actions workflow that builds:
  - Windows x64 (MSVC): `axonc.exe`
  - macOS x64 (Apple Clang): `axonc`
  - macOS ARM64 (Apple Silicon): `axonc`
  - Linux x64 (musl static): `axonc`
  - Linux ARM64: `axonc`
  Each binary includes the runtime library.
  Create GitHub Release with all assets.
- **Acceptance**: All 5 binaries build in CI.

#### T477 — Windows installer
- **File**: `scripts/windows/`
- **Description**: MSI or NSIS installer that:
  - Installs `axonc.exe` to Program Files
  - Adds to PATH
  - Installs VS Code extension (optional)
  - Installs runtime library
  - Creates Start Menu shortcut
  Also support `winget install axon`.
- **Acceptance**: Fresh Windows install → `axonc --version` works.

#### T478 — macOS installer
- **File**: `scripts/macos/`
- **Description**: Homebrew formula: `brew install axon`.
  Also `.pkg` installer for non-Homebrew users.
  Universal binary (x64 + ARM64).
- **Acceptance**: `brew install axon && axonc --version` works.

#### T479 — Linux packages
- **File**: `scripts/linux/`
- **Description**:
  - `.deb` package (Ubuntu/Debian)
  - `.rpm` package (Fedora/RHEL)
  - Snap package
  - `curl -sSf https://axonlang.org/install.sh | sh`
- **Acceptance**: `sudo apt install axon && axonc --version` works.

#### T480 — Docker image
- **File**: `Dockerfile`
- **Description**: Official Docker images:
  - `axonlang/axon:latest` — CPU only
  - `axonlang/axon:cuda` — with CUDA support
  - `axonlang/axon:rocm` — with ROCm support
  Based on Ubuntu, includes axonc + runtime + examples.
- **Acceptance**: `docker run axonlang/axon axonc --version` works.

#### T481 — GPU runtime distribution
- **Description**: Pre-built GPU runtime libraries:
  - `libaxon_rt_cuda.so` / `axon_rt_cuda.dll` — CUDA variant
  - `libaxon_rt_rocm.so` — ROCm variant
  - `libaxon_rt.so` / `axon_rt.lib` — CPU only
  GPU variants include cuBLAS/cuDNN or rocBLAS/MIOpen.
  Detect at compile time which to link.
- **Acceptance**: `axonc build --gpu=cuda model.axon` links GPU runtime.

### Sub-phase 15e: Package Registry (T482–T485)

#### T482 — Package registry server
- **File**: `registry/` (new sub-project)
- **Description**: Simple HTTP API server for `axon pkg publish` and
  `axon pkg install`.  Storage: S3 or filesystem.  API:
  - `POST /api/v1/packages` — publish (tarball + metadata)
  - `GET /api/v1/packages/{name}/{version}` — download
  - `GET /api/v1/packages/{name}` — list versions
  - `GET /api/v1/search?q=...` — search
  Auth: API tokens.
- **Acceptance**: Publish and install a package end-to-end.

#### T483 — Package client enhancements
- **File**: `src/pkg/commands.rs`
- **Description**: Update `axon pkg` commands to talk to real registry:
  - `axon pkg publish` — build tarball, upload
  - `axon pkg install <name>` — download, extract to `.axon_packages/`
  - `axon pkg search <query>` — search registry
  - `axon pkg login` — authenticate
- **Acceptance**: Full publish → install flow works.

#### T484 — Seed packages
- **Description**: Publish initial community packages:
  - `axon-plot` — basic plotting (output SVG/PNG)
  - `axon-cv` — computer vision transforms (resize, crop, augment)
  - `axon-nlp` — tokenizers (BPE, WordPiece)
  - `axon-datasets` — common datasets (MNIST, CIFAR, IMDB)
- **Acceptance**: 4+ packages on registry.

#### T485 — Package documentation
- **File**: `docs/guide/modules-packages.md` (rewrite)
- **Description**: How to create, publish, and install packages.
  Real working example: create a package, publish, install in another project.
- **Acceptance**: Tutorial creates and publishes a real package.

### Sub-phase 15f: Website & Launch (T486–T492)

#### T486 — Website: landing page
- **File**: `website/` (new sub-project)
- **Description**: `axonlang.org` landing page:
  - Hero section with Axon code snippet
  - Key features: compile-time shapes, ownership, GPU, autograd
  - Comparison table vs Python/PyTorch, Rust, Julia, Mojo
  - Install command
  - Links to docs, GitHub, Discord
  Built with static site generator (Hugo/Astro/plain HTML).
- **Acceptance**: Page loads fast, looks professional, responsive.

#### T487 — Website: documentation hosting
- **Description**: Host all docs at `docs.axonlang.org`:
  - API reference (from doc generator)
  - Guides and tutorials
  - Error reference
  - Search functionality
  Deploy with GitHub Pages or Netlify.
- **Acceptance**: All docs browsable at docs.axonlang.org.

#### T488 — Website: playground (stretch)
- **Description**: In-browser Axon playground (compile + run small programs).
  WASM-compiled type checker + limited codegen.
  Or server-side sandbox.
- **Acceptance**: Can type-check simple programs in browser.

#### T489 — Community setup
- **Description**: Create:
  - Discord server with channels (#general, #help, #showcase, #dev)
  - GitHub Discussions enabled
  - Issue templates (bug report, feature request)
  - Contributing guide with CLA or DCO
  - Code of conduct
- **Acceptance**: Community channels are live and welcoming.

#### T490 — Security audit
- **Description**: Final security review:
  - Fuzz all parsers (axon source, config files, checkpoint files)
  - Review all C runtime code for buffer overflows, use-after-free
  - Audit all network code (registry client, auto-download)
  - No arbitrary code execution paths
  Run with ASAN + UBSAN on all test suites.
- **Acceptance**: No security issues found, or all resolved.

#### T491 — Final test suite
- **Description**: All tests pass on all platforms:
  - Windows x64 (CPU)
  - macOS x64 + ARM64 (CPU)
  - Linux x64 (CPU + CUDA + ROCm)
  Total test count target: 1500+.
  CI passes on all platforms.
- **Acceptance**: Green CI on all platforms.

#### T492 — v1.0.0 release
- **Description**:
  1. Tag `v1.0.0` in git
  2. GitHub Release with all binaries
  3. Publish to package managers (brew, apt, winget, snap, docker)
  4. Publish VS Code extension
  5. Deploy website
  6. Write announcement blog post
  7. Post to Hacker News, Reddit (r/programming, r/MachineLearning),
     Twitter/X
- **Acceptance**: Public can download, install, and use Axon.

---

## Error Codes

No new error codes.  This phase improves existing error messages.

---

## Test Plan

1. **Error message tests** — snapshot tests for every error message
2. **Installer tests** — test install on fresh VMs (Windows, macOS, Linux)
3. **Documentation tests** — every code example in docs compiles
4. **LSP tests** — response time < 200ms for all operations
5. **Registry tests** — publish/install round-trip
6. **Cross-platform CI** — all tests pass on all platforms

---

## Exit Criteria

1. All 34 tasks complete
2. Every error message has a clear explanation and actionable suggestion
3. Documentation covers all features with working examples
4. VS Code extension on marketplace with diagnostics, completions, go-to-def
5. Installers work on Windows, macOS, Linux
6. Package registry operational
7. Website live at axonlang.org
8. 1500+ tests passing across all platforms
9. v1.0.0 tag created and released
10. A new user can install → tutorial → train a model without blockers
