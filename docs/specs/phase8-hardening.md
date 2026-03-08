# Phase 8: Hardening & Release — Implementation Specification

## 1. Overview

Phase 8 is the final phase before Axon's public release. It focuses on **quality assurance, performance validation, security, documentation polish, and release engineering**. No major new features are added — this phase hardens everything built in Phases 0–7.

### Deliverables

| Deliverable                     | Purpose                                                             |
| ------------------------------- | ------------------------------------------------------------------- |
| Benchmark suite                 | Quantify compiler speed, runtime performance, GPU throughput        |
| Fuzz testing infrastructure     | Find crashes, panics, and edge cases in lexer/parser/typeck/codegen |
| Security audit                  | Review unsafe code, FFI boundaries, memory safety guarantees        |
| Documentation site              | Comprehensive user guide, API reference, tutorials                  |
| Release binaries                | Prebuilt binaries for Linux (x86_64, aarch64), macOS, Windows       |
| CI/CD pipeline                  | Automated build, test, benchmark, publish                           |
| Migration guide                 | How to port Python/PyTorch code to Axon                             |
| Specification compliance report | Verify all FR/NFR requirements from Phase 1 are met                 |

### Dependencies

- All previous phases (0–7) must be complete and stable.

---

## 2. Benchmarks

### 2.1 Compiler Benchmarks

| Benchmark           | Target              | Description                        |
| ------------------- | ------------------- | ---------------------------------- |
| Lex throughput      | > 5M tokens/sec     | Lex a large .axon file             |
| Parse throughput    | > 1M lines/sec      | Parse all stdlib files             |
| Type check speed    | < 2s for 50K LOC    | Full type check on a large project |
| Full compile (cold) | < 10s for 10K LOC   | From source to native binary       |
| Incremental compile | < 500ms             | Change one function, recompile     |
| Memory usage        | < 500MB for 50K LOC | Peak RSS during compilation        |

### 2.2 Runtime Benchmarks

| Benchmark             | Comparison  | Description                                  |
| --------------------- | ----------- | -------------------------------------------- |
| Matrix multiply (CPU) | vs. C, Rust | 1024×1024 Float32 matmul                     |
| Matrix multiply (GPU) | vs. CUDA C  | 4096×4096 Float32 matmul                     |
| MLP training          | vs. PyTorch | 3-layer MLP on MNIST, 10 epochs              |
| CNN training          | vs. PyTorch | ResNet-18 on CIFAR-10, 1 epoch               |
| Transformer inference | vs. PyTorch | GPT-2-small, 1 forward pass                  |
| Tensor creation       | vs. NumPy   | Create 1M element tensor                     |
| Elementwise ops       | vs. NumPy   | 1M element add/mul/exp                       |
| Memory overhead       | vs. PyTorch | Per-tensor memory overhead                   |
| Startup time          | < 10ms      | Time to first instruction of compiled binary |
| Collection ops        | vs. Rust    | Vec push/pop, HashMap insert/lookup          |

### 2.3 Benchmark Infrastructure

```
benches/
├── compiler/
│   ├── lex_throughput.rs
│   ├── parse_throughput.rs
│   ├── typeck_speed.rs
│   └── compile_speed.rs
├── runtime/
│   ├── matmul_cpu.axon
│   ├── matmul_gpu.axon
│   ├── mlp_train.axon
│   ├── cnn_train.axon
│   ├── transformer_infer.axon
│   ├── tensor_creation.axon
│   └── collections.axon
├── baselines/
│   ├── matmul_pytorch.py
│   ├── matmul_numpy.py
│   └── matmul_c.c
└── report/
    └── generate_report.py     # Produces comparison tables + charts
```

Use **criterion.rs** for Rust-side benchmarks. Custom harness for Axon runtime benchmarks.

---

## 3. Fuzz Testing

### 3.1 Fuzz Targets

| Target        | Input                    | Goal                                             |
| ------------- | ------------------------ | ------------------------------------------------ |
| Lexer         | Random byte sequences    | No panics, no infinite loops                     |
| Parser        | Random token sequences   | No panics, graceful error recovery               |
| Type checker  | Random valid ASTs        | No panics, correct rejection of invalid programs |
| Shape checker | Random tensor operations | No panics on shape mismatches                    |
| Codegen       | Random valid typed ASTs  | No LLVM crashes, no segfaults in output          |
| REPL          | Random input lines       | No panics, no hangs                              |
| Formatter     | Random .axon source      | No panics, idempotent output                     |

### 3.2 Tools

- **cargo-fuzz** (libFuzzer) for Rust fuzzing.
- **AFL++** as alternative fuzzer.
- **Structured fuzzing**: generate syntactically valid programs via grammar-based generation.
- **Regression corpus**: all crash-inducing inputs saved for CI.

### 3.3 Targets

```rust
// Example fuzz target
#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(source) = std::str::from_utf8(data) {
        let _ = axon::parse_source(source, "<fuzz>");
    }
});
```

### 3.4 Coverage Goals

- **Line coverage**: > 90% for lexer, parser, type checker.
- **Branch coverage**: > 80% for all compiler passes.
- Zero unhandled panics after 48 hours of continuous fuzzing per target.

---

## 4. Security Audit

### 4.1 Areas to Audit

| Area                 | Concern                                      |
| -------------------- | -------------------------------------------- |
| `unsafe` code blocks | Memory safety violations, undefined behavior |
| FFI boundaries       | Buffer overflows, invalid pointer usage      |
| LLVM IR generation   | Malformed IR leading to crashes              |
| GPU kernel launch    | Out-of-bounds memory access on device        |
| Package registry     | Supply chain attacks, package typosquatting  |
| Deserialization      | Model/checkpoint loading arbitrary data      |
| REPL                 | Code injection via untrusted input           |

### 4.2 Actions

1. **Audit all `unsafe` blocks**: each must have a `// SAFETY:` comment explaining why it's sound.
2. **Run Miri** on the compiler itself (where possible) to detect UB.
3. **Run AddressSanitizer / MemorySanitizer** on compiled Axon programs.
4. **Dependency audit**: `cargo audit` for known vulnerabilities.
5. **Threat model** document for the package registry.

### 4.3 Memory Safety Guarantees

Document and verify:

- Safe Axon code cannot cause segfaults, use-after-free, or buffer overflows.
- `unsafe fn` is required for all operations that bypass the borrow checker.
- Tensor operations perform bounds checking unless explicitly opted out.

---

## 5. Documentation

### 5.1 Documentation Structure

```
docs/
├── guide/
│   ├── getting-started.md        # Install, hello world, first project
│   ├── language-tour.md          # Quick overview of Axon syntax
│   ├── ownership-borrowing.md    # Ownership model explained
│   ├── tensors.md                # Tensor types, shapes, operations
│   ├── gpu-programming.md        # @gpu, @cpu, @device, device transfer
│   ├── generics-traits.md        # Generic programming and traits
│   ├── error-handling.md         # Result, Option, ? operator
│   ├── modules-packages.md       # Module system, package manager
│   ├── concurrency.md            # Threads, channels, sync primitives
│   └── ffi.md                    # Calling C code, unsafe
├── tutorial/
│   ├── 01-hello-tensor.md        # First tensor program
│   ├── 02-linear-regression.md   # ML hello world
│   ├── 03-mnist-classifier.md    # CNN on MNIST
│   ├── 04-transformer.md         # Building a transformer from scratch
│   ├── 05-training-loop.md       # Custom training loop
│   └── 06-deployment.md          # ONNX export, model serving
├── reference/
│   ├── language-spec.md          # Complete language reference
│   ├── stdlib-api/               # Auto-generated API docs
│   ├── compiler-errors.md        # All error codes with explanations
│   └── cli-reference.md          # All CLI commands and flags
├── internals/
│   ├── architecture.md           # Compiler architecture overview
│   ├── type-system.md            # Type system internals
│   ├── codegen.md                # Code generation internals
│   └── contributing.md           # How to contribute to Axon
└── migration/
    ├── from-python.md            # Python/NumPy → Axon
    ├── from-pytorch.md           # PyTorch → Axon
    └── from-rust.md              # Rust → Axon (differences)
```

### 5.2 Documentation Site

- Built with **mdBook** or custom static site generator.
- Hosted at `axonlang.org/docs`.
- Search functionality.
- Code examples are tested as part of CI (doc tests).
- Dark/light theme.

### 5.3 Code Examples in Docs

Every code example in documentation must be:

1. **Compilable**: extracted and compiled as part of CI.
2. **Runnable**: produces expected output.
3. **Annotated**: with comments explaining key concepts.

---

## 6. Release Engineering

### 6.1 CI/CD Pipeline

```yaml
# Runs on every push / PR
pipeline:
  - stage: lint
    steps:
      - cargo fmt --check
      - cargo clippy -- -D warnings
  - stage: test
    steps:
      - cargo test --all
      - cargo test --all --release
  - stage: fuzz
    steps:
      - cargo fuzz run lexer_fuzz -- -max_total_time=300
      - cargo fuzz run parser_fuzz -- -max_total_time=300
  - stage: benchmark
    steps:
      - cargo bench -- --output-format json > bench_results.json
      - python3 benches/report/check_regression.py bench_results.json
  - stage: docs
    steps:
      - axonc doc --all
      - mdbook build docs/

# Runs on tag (release)
release:
  - stage: build
    matrix:
      - os:
          [
            linux-x86_64,
            linux-aarch64,
            macos-x86_64,
            macos-aarch64,
            windows-x86_64,
          ]
    steps:
      - cargo build --release --target ${{ matrix.target }}
      - strip + compress binary
      - sign binary
  - stage: publish
    steps:
      - Upload binaries to GitHub Releases
      - Publish cargo crate
      - Update docs site
      - Update Homebrew formula / APT repo / winget manifest
```

### 6.2 Release Binaries

| Platform       | Binary      | Package Formats                     |
| -------------- | ----------- | ----------------------------------- |
| Linux x86_64   | `axonc`     | `.tar.gz`, `.deb`, `.rpm`, AppImage |
| Linux aarch64  | `axonc`     | `.tar.gz`, `.deb`                   |
| macOS x86_64   | `axonc`     | `.tar.gz`, Homebrew                 |
| macOS aarch64  | `axonc`     | `.tar.gz`, Homebrew                 |
| Windows x86_64 | `axonc.exe` | `.zip`, `.msi`, winget              |

### 6.3 Versioning

- Semantic versioning: `MAJOR.MINOR.PATCH`
- Pre-release: `0.1.0-alpha.1`, `0.1.0-beta.1`, `0.1.0-rc.1`
- Release cadence: monthly beta → quarterly stable

### 6.4 Installation

```bash
# One-line install (Unix)
curl -sSf https://axonlang.org/install.sh | sh

# Homebrew (macOS)
brew install axonlang/tap/axon

# Cargo
cargo install axonc

# Windows
winget install axonlang.axon
```

---

## 7. Specification Compliance

### 7.1 Functional Requirements Verification

Verify every FR from the Phase 1 specification:

| FR Range         | Area                    | Status                            |
| ---------------- | ----------------------- | --------------------------------- |
| FR-001 to FR-010 | Core language syntax    | Verify against lexer/parser       |
| FR-011 to FR-014 | Tensor types and shapes | Verify against shape checker      |
| FR-015 to FR-016 | Ownership and borrowing | Verify against borrow checker     |
| FR-017 to FR-030 | Type system features    | Verify against type checker       |
| FR-031 to FR-040 | Standard library        | Verify against stdlib             |
| FR-041 to FR-045 | Error reporting and CLI | Verify against CLI + error system |
| FR-046 to FR-060 | GPU and device support  | Verify against codegen + runtime  |
| FR-061 to FR-072 | AI/ML features          | Verify against AI framework       |

### 7.2 Non-Functional Requirements Verification

| NFR           | Requirement                               | Verification Method |
| ------------- | ----------------------------------------- | ------------------- |
| Performance   | Compile 10K LOC < 10s                     | Benchmark suite     |
| Memory        | Compiler < 500MB for 50K LOC              | Memory profiling    |
| Error quality | All errors have code + span + suggestion  | Test suite          |
| Safety        | No UB in safe code                        | Fuzzing + Miri      |
| Compatibility | x86_64 + aarch64, Linux + macOS + Windows | CI matrix           |

---

## 8. Task Breakdown

### Phase 8a: Benchmarks

- [ ] T246 Implement compiler benchmark suite — `benches/compiler/`
- [ ] T247 Implement runtime benchmark suite — `benches/runtime/`
- [ ] T248 Create baseline comparisons (PyTorch, NumPy, C) — `benches/baselines/`
- [ ] T249 Create benchmark reporting tool — `benches/report/`
- [ ] T250 Optimize hot paths identified by benchmarks — various

### Phase 8b: Fuzz Testing

- [ ] T251 Set up cargo-fuzz infrastructure — `fuzz/`
- [ ] T252 Create fuzz targets for lexer, parser, typeck — `fuzz/`
- [ ] T253 Create grammar-based structured fuzzer — `fuzz/`
- [ ] T254 Run 48h fuzz campaigns and fix all crashes — `fuzz/`
- [ ] T255 Build regression corpus for CI — `fuzz/corpus/`

### Phase 8c: Security

- [ ] T256 Audit all `unsafe` blocks with SAFETY comments — various
- [ ] T257 Run Miri on compiler code — CI
- [ ] T258 Run sanitizers on compiled Axon programs — CI
- [ ] T259 Run `cargo audit` and fix vulnerabilities — CI
- [ ] T260 Write threat model for package registry — `docs/internals/`

### Phase 8d: Documentation

- [ ] T261 Write getting-started guide — `docs/guide/`
- [ ] T262 Write language tour — `docs/guide/`
- [ ] T263 Write tensor & GPU programming guides — `docs/guide/`
- [ ] T264 Write ML tutorials (1–6) — `docs/tutorial/`
- [ ] T265 Generate stdlib API reference — `docs/reference/`
- [ ] T266 Write compiler error reference — `docs/reference/`
- [ ] T267 Write migration guides (Python, PyTorch, Rust) — `docs/migration/`
- [ ] T268 Build and deploy documentation site — `docs/`
- [ ] T269 Test all code examples in documentation — CI

### Phase 8e: Release Engineering

- [ ] T270 Set up CI/CD pipeline (lint, test, fuzz, bench, docs) — `.github/workflows/`
- [ ] T271 Build release binaries for all platforms — CI
- [ ] T272 Create install scripts (curl, Homebrew, winget) — `scripts/`
- [ ] T273 Write CHANGELOG and release notes — `CHANGELOG.md`
- [ ] T274 Set up package signing — CI

### Phase 8f: Compliance

- [ ] T275 FR compliance matrix: verify all 72 functional requirements — `docs/compliance/`
- [ ] T276 NFR compliance matrix: verify all 28 non-functional requirements — `docs/compliance/`
- [ ] T277 Generate final specification compliance report — `docs/compliance/report.md`

---

## 9. Acceptance Criteria

- [ ] All benchmarks meet target thresholds
- [ ] Zero crashes after 48h of fuzzing per target
- [ ] All `unsafe` blocks audited and documented
- [ ] Documentation covers all language features with working examples
- [ ] Release binaries work on all 5 target platforms
- [ ] CI pipeline runs green on every commit
- [ ] All 72 functional requirements verified and passing
- [ ] All 28 non-functional requirements verified and passing
- [ ] Installation works via all documented methods
- [ ] CHANGELOG documents all features from Phases 0–7
