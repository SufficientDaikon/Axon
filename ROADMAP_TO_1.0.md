# Axon v1.0.0 Roadmap — Current State to Production Release

> Generated: 2026-03-11 | Based on: Full codebase audit, 15 phase specs, compliance reports
> Source: 28,291 lines Rust (src/), 5,201 lines tests, 15 spec documents (Phases 0-15)

---

## EXECUTIVE SUMMARY

### What Exists Today
- **Phases 1-8** are marked complete in compliance reports
- **Phases 9-15** are fully specified but unimplemented
- 28,291 lines of Rust source across ~40 files
- 461+ tests (but currently **0 pass** due to linker issue)
- Full compiler pipeline: Lexer → Parser → TypeChecker → MIR → LLVM IR → Binary
- Stdlib is **type-checked stubs** (signatures work, runtime behavior depends on LLVM backend)
- E2E compilation works (36 programs compile+run when linker is fixed)
- AI framework (nn, autograd, optim, loss) exists as **stdlib stubs only**

### What's Actually Working vs Stubs
| Component | Real Implementation | Stub/Signature Only |
|---|---|---|
| Lexer/Parser/AST | REAL | - |
| Type checker + inference | REAL (80%) | Let-polymorphism, full trait dispatch |
| Shape checker | REAL (95%) | Runtime shape checks |
| Borrow checker | REAL (70%, AST-walk) | CFG-based analysis |
| TAST builder | REAL | - |
| MIR lowering | REAL | - |
| LLVM codegen | REAL (scalar ops) | Tensor ops emit runtime calls to C stubs |
| C Runtime | PARTIAL | Tensor ops are stubs returning dummy values |
| Stdlib (.axon files) | Signatures type-check | All function bodies are pseudocode |
| AI Framework | Signatures type-check | Zero runnable autograd/nn/optim |
| LSP | REAL (basic) | No rename/refactor |
| Formatter/Linter | REAL | - |
| REPL | REAL (basic) | - |
| Package manager | REAL (basic) | No registry, no SAT solver |

### The Gap to 1.0
The original spec defined v0.1.0 at Phase 8 completion. Phases 9-15 take it to true v1.0:
- **Phase 9**: Fix E2E compilation bugs (IR correctness, runtime linking)
- **Phase 10**: Real CPU tensor runtime (matmul, relu, reshape actually compute)
- **Phase 11**: Autograd engine (backward pass, gradient computation)
- **Phase 12**: GPU backend (CUDA + ROCm)
- **Phase 13**: Training infrastructure (optimizers, data loading, checkpointing)
- **Phase 14**: Model validation (train MNIST MLP, CIFAR CNN, transformer)
- **Phase 15**: Production polish (error messages, docs, installers, website)

---

## PHASE 0: EMERGENCY — FIX THE BUILD (Day 1, ~1 hour)

**Problem**: Git's `link.exe` (`/usr/bin/link.exe`) shadows MSVC's linker. Every `cargo test` and `cargo build` fails with `link: missing operand`.

### Tasks
| # | Task | Time |
|---|---|---|
| E-001 | Fix PATH ordering: ensure MSVC `link.exe` comes before Git's | 15 min |
| E-002 | Run `cargo test` — verify all 461+ tests pass | 10 min |
| E-003 | Run `cargo build` — verify binary compiles | 5 min |
| E-004 | Run E2E tests — verify compiled Axon programs execute | 15 min |
| E-005 | Document fix in project README or CLAUDE.md | 5 min |

**Fix Options (pick one):**
1. Set environment variable: `LINK` env var pointing to MSVC's `link.exe` absolute path
2. Reorder PATH in shell profile so VS Build Tools comes before Git
3. Use `.cargo/config.toml` to specify the linker explicitly:
   ```toml
   [target.x86_64-pc-windows-msvc]
   linker = "C:\\Program Files (x86)\\Microsoft Visual Studio\\2022\\BuildTools\\VC\\Tools\\MSVC\\14.39.33519\\bin\\Hostx64\\x64\\link.exe"
   ```

**Gate**: All 461+ tests green. No linker errors.

---

## PHASE 1: COMPLETE PHASE 3 GAPS (Week 1, ~3 days)

The type system is 80-85% done. These gaps affect correctness downstream.

### Sub-phase 1a: Wire Existing Dead Code (1 day)
| # | Task | File | Description |
|---|---|---|---|
| G-001 | Wire device-aware borrow checks | `src/borrow.rs` | `check_device_transfer()` and `check_device_borrow()` exist but aren't called from `check_expr()` |
| G-002 | Wire lifetime escape checking | `src/borrow.rs` | `check_reference_escape()` exists but isn't called |
| G-003 | Wire constraint solver | `src/typeck.rs` | `Constraint` enum defined but unused — either remove or wire into inference |
| G-004 | Fix all 6 dead-code warnings | various | Suppress or integrate unused code |

### Sub-phase 1b: Type Coercions (1 day)
| # | Task | File | Description |
|---|---|---|---|
| G-005 | Implement `&mut T` → `&T` coercion | `src/typeck.rs` | Allow passing `&mut` where `&` is expected |
| G-006 | Implement auto-borrow in method calls | `src/typeck.rs` | Auto-insert `&`/`&mut` for method receivers |
| G-007 | Add coercion tests | `tests/type_tests.rs` | 10+ tests for coercion scenarios |

### Sub-phase 1c: Trait Resolution Hardening (1 day)
| # | Task | File | Description |
|---|---|---|---|
| G-008 | Trait bounds checking on generic functions | `src/typeck.rs` | `where T: Add` should actually be enforced |
| G-009 | Trait method dispatch (vtable-style) | `src/typeck.rs` | Look up methods through trait impls, not just inherent |
| G-010 | Orphan rule (basic) | `src/typeck.rs` | Prevent implementing foreign traits on foreign types |
| G-011 | Trait resolution tests | `tests/type_tests.rs` | 15+ tests for trait dispatch edge cases |

**Gate**: `cargo test` passes with new tests. Shape checker, borrow checker, and type checker have no dead-code warnings.

---

## PHASE 2: E2E COMPILATION FIX-UP (Week 1-2, ~4 days)

This is the existing Phase 9 spec. The compiler generates LLVM IR but has known bugs.

### Sub-phase 2a: Fix LLVM IR Codegen Bugs (2 days)
| # | Task | File | Bug |
|---|---|---|---|
| E2E-001 | Fix `main` return type (`void` → `i32`) | `src/codegen/llvm.rs` | C main must return i32 |
| E2E-002 | Fix string constant ABI (pass length with `i8*`) | `src/codegen/llvm.rs` | `axon_print_str` expects `(i8*, i64)` |
| E2E-003 | Fix `Rvalue::Len` (returns 0) | `src/codegen/llvm.rs` | Array/string length broken |
| E2E-004 | Fix `Rvalue::Aggregate` type_of_rvalue | `src/codegen/llvm.rs` | Always returns UNIT |
| E2E-005 | Fix `emit_place_store` projections | `src/codegen/llvm.rs` | Struct field writes are no-ops |
| E2E-006 | Fix comparison ops codegen | `src/codegen/llvm.rs` | Ensure all 6 comparison ops work |

### Sub-phase 2b: Complete C Runtime (1 day)
| # | Task | File | Description |
|---|---|---|---|
| E2E-007 | Real `axon_print_*` implementations | `src/codegen/runtime.rs` | All 6 print functions (i32, i64, f32, f64, bool, str) |
| E2E-008 | Real `axon_alloc`/`axon_dealloc` | `src/codegen/runtime.rs` | malloc/free wrappers |
| E2E-009 | Real `axon_panic` with message | `src/codegen/runtime.rs` | Print message and abort |
| E2E-010 | Compile + link runtime with IR | `src/codegen/llvm.rs` | `compile_ir_to_binary()` must compile `axon_runtime.c` and link it |

### Sub-phase 2c: E2E Test Harness (1 day)
| # | Task | File | Description |
|---|---|---|---|
| E2E-011 | Expand E2E test harness | `tests/e2e_tests.rs` | Compile → run → assert stdout/exit code |
| E2E-012 | Add 15+ test programs | `tests/e2e/` | hello, fibonacci, structs, enums, closures, recursion |
| E2E-013 | Cross-platform CI | `.github/workflows/` | Windows MSVC, Linux GCC, macOS Clang |

**Gate**: `axonc build hello.axon && ./hello` works. 15+ programs compile and run correctly. All E2E tests green.

---

## PHASE 3: CPU TENSOR RUNTIME (Week 2-3, ~5 days)

This is the existing Phase 10 spec. Replace every tensor stub with real math.

### Sub-phase 3a: Tensor Data Structure (1 day)
| # | Task | File | Description |
|---|---|---|---|
| T-001 | Implement `AxonTensor` C struct | `runtime/axon_tensor.c` | data, shape, strides, ndim, numel, dtype, refcount |
| T-002 | Tensor creation (`zeros`, `ones`, `rand`, `from_data`) | `runtime/axon_tensor.c` | Allocate + initialize |
| T-003 | Tensor refcounting (`retain`/`release`) | `runtime/axon_tensor.c` | Auto-free on refcount=0 |
| T-004 | Tensor views (`slice`, `reshape`, `transpose`) | `runtime/axon_tensor.c` | Non-owning views with strides |
| T-005 | Tensor printing | `runtime/axon_tensor.c` | Pretty-print with shape info |

### Sub-phase 3b: Element-wise Operations (1 day)
| # | Task | File | Description |
|---|---|---|---|
| T-006 | Add, Sub, Mul, Div (element-wise) | `runtime/axon_tensor_ops.c` | f32/f64 support |
| T-007 | Comparison ops (eq, ne, lt, gt, le, ge) | `runtime/axon_tensor_ops.c` | Returns bool tensor |
| T-008 | Unary ops (neg, abs, exp, log, sqrt) | `runtime/axon_tensor_ops.c` | Standard math |
| T-009 | Activation functions (relu, sigmoid, tanh, gelu) | `runtime/axon_tensor_ops.c` | In-place variants too |
| T-010 | Broadcasting engine | `runtime/axon_tensor_ops.c` | NumPy-style broadcast iteration |

### Sub-phase 3c: Reduction & Linear Algebra (1.5 days)
| # | Task | File | Description |
|---|---|---|---|
| T-011 | Sum, mean, max, min, argmax, argmin | `runtime/axon_tensor_reduce.c` | Along axis or all |
| T-012 | Matrix multiply (naive → blocked) | `runtime/axon_tensor_linalg.c` | Correctness first, optimize later |
| T-013 | Batch matmul | `runtime/axon_tensor_linalg.c` | [B, M, K] @ [B, K, N] → [B, M, N] |
| T-014 | Transpose, permute | `runtime/axon_tensor_linalg.c` | Stride manipulation |

### Sub-phase 3d: Compiler Integration (1.5 days)
| # | Task | File | Description |
|---|---|---|---|
| T-015 | LLVM codegen: tensor MIR ops → runtime calls | `src/codegen/llvm.rs` | Map `TensorOp::MatMul` to `axon_tensor_matmul()` etc. |
| T-016 | Tensor type layout in LLVM IR | `src/codegen/llvm.rs` | `%AxonTensor = type { i8*, i64*, i64*, i32, i64, i32, i32, i32, i1, i8* }` |
| T-017 | Tensor literal codegen | `src/codegen/llvm.rs` | `Tensor::from([1.0, 2.0, 3.0])` → runtime call |
| T-018 | Link tensor runtime with binary | `src/codegen/llvm.rs` | Compile all `runtime/*.c` and link |
| T-019 | E2E tensor tests | `tests/e2e/` | 10+ programs that create tensors, do math, print results |

**Gate**: This program compiles and runs:
```axon
val a = Tensor.rand([128, 784]);
val w = Tensor.rand([784, 256]);
val b = a @ w;
val c = b.relu();
println(c.sum());  // prints a number
```

---

## PHASE 4: AUTOGRAD ENGINE (Week 3-4, ~5 days)

This is the existing Phase 11 spec. Tape-based reverse-mode autodiff.

### Sub-phase 4a: Computation Graph (2 days)
| # | Task | File | Description |
|---|---|---|---|
| A-001 | `AxonGradFn` struct | `runtime/axon_autograd.c` | Operation node: backward function pointer, saved inputs |
| A-002 | `requires_grad` field on AxonTensor | `runtime/axon_tensor.c` | Enable gradient tracking |
| A-003 | `grad` field on AxonTensor | `runtime/axon_tensor.c` | Accumulated gradient storage |
| A-004 | Tape recording (forward ops append to tape) | `runtime/axon_autograd.c` | Thread-local tape |
| A-005 | `no_grad` context (disable recording) | `runtime/axon_autograd.c` | For inference |

### Sub-phase 4b: Backward Pass (2 days)
| # | Task | File | Description |
|---|---|---|---|
| A-006 | Topological sort of computation graph | `runtime/axon_autograd.c` | Kahn's algorithm |
| A-007 | `backward()` entry point | `runtime/axon_autograd.c` | Traverse reverse topo order |
| A-008 | Gradient rules: add, sub, mul, div | `runtime/axon_autograd.c` | d/dx(a+b)=1, d/dx(a*b)=b |
| A-009 | Gradient rules: matmul | `runtime/axon_autograd.c` | dL/dA = dL/dC @ B^T |
| A-010 | Gradient rules: relu, sigmoid, tanh | `runtime/axon_autograd.c` | Step function, σ(1-σ), 1-tanh² |
| A-011 | Gradient rules: sum, mean, reshape, transpose | `runtime/axon_autograd.c` | Broadcast grad, divide, permute strides |
| A-012 | Gradient accumulation (multi-use tensors) | `runtime/axon_autograd.c` | += not = for grad |

### Sub-phase 4c: Verification (1 day)
| # | Task | File | Description |
|---|---|---|---|
| A-013 | Numerical gradient checking utility | `tests/` | Finite differences: (f(x+ε)-f(x-ε))/(2ε) |
| A-014 | Autograd E2E tests | `tests/e2e/` | 10+ programs testing backward() |
| A-015 | Memory leak tests | `tests/` | Verify tape is cleaned after backward |

**Gate**: This runs:
```axon
val w = Tensor.rand([784, 10], requires_grad: true);
val x = Tensor.rand([32, 784]);
val y = (x @ w).relu().sum();
y.backward();
println(w.grad.shape());  // [784, 10]
```

---

## PHASE 5: GPU BACKEND (Week 4-6, ~8 days)

This is the existing Phase 12 spec. CUDA + ROCm support.

### Sub-phase 5a: GPU Abstraction Layer (2 days)
| # | Task | File | Description |
|---|---|---|---|
| GPU-001 | `axon_gpu.h` abstraction layer | `runtime/axon_gpu.h` | Thin wrapper over CUDA/HIP APIs |
| GPU-002 | GPU context initialization (lazy) | `runtime/axon_gpu.c` | Create context on first `to_gpu()` |
| GPU-003 | GPU memory allocator (caching) | `runtime/axon_gpu.c` | Reuse freed blocks |
| GPU-004 | `to_gpu()` / `to_cpu()` transfers | `runtime/axon_gpu.c` | cudaMemcpy / hipMemcpy |

### Sub-phase 5b: GPU Tensor Ops (3 days)
| # | Task | File | Description |
|---|---|---|---|
| GPU-005 | Element-wise ops (CUDA kernels) | `runtime/axon_gpu_kernels.cu` | add, mul, relu etc. — simple grid-stride loops |
| GPU-006 | Matmul via cuBLAS/rocBLAS | `runtime/axon_gpu_blas.c` | cublasSgemm / rocblas_sgemm |
| GPU-007 | Reduction ops (parallel sum, mean, max) | `runtime/axon_gpu_kernels.cu` | Tree reduction pattern |
| GPU-008 | Broadcast on GPU | `runtime/axon_gpu_kernels.cu` | Index mapping with strides |

### Sub-phase 5c: GPU Autograd (2 days)
| # | Task | File | Description |
|---|---|---|---|
| GPU-009 | Autograd tape works with GPU tensors | `runtime/axon_autograd.c` | Same graph, device-aware backward |
| GPU-010 | GPU gradient rules (same math, GPU memory) | `runtime/axon_gpu_autograd.c` | backward ops run on GPU |
| GPU-011 | Mixed CPU/GPU gradient flow | `runtime/axon_autograd.c` | to_gpu/to_cpu in graph |

### Sub-phase 5d: Testing (1 day)
| # | Task | File | Description |
|---|---|---|---|
| GPU-012 | GPU E2E tests (skip if no GPU) | `tests/e2e/` | Compile + run, compare CPU vs GPU results |
| GPU-013 | cuBLAS numerical accuracy tests | `tests/` | Compare against CPU matmul |
| GPU-014 | Memory leak tests (GPU allocator) | `tests/` | No CUDA memory leaks after program exit |

**Gate**: Matmul on GPU matches CPU result within 1e-5. Autograd backward works on GPU.

**Note**: GPU phase can be deprioritized for v1.0 if CUDA/ROCm toolchains are unavailable. CPU-only v1.0 is a valid release target.

---

## PHASE 6: TRAINING INFRASTRUCTURE (Week 5-7, ~6 days)

This is the existing Phase 13 spec. Make the stdlib stubs real.

### Sub-phase 6a: nn.Module System (2 days)
| # | Task | File | Description |
|---|---|---|---|
| NN-001 | Module trait → C runtime | `runtime/axon_nn.c` | `forward()`, `parameters()`, `train()/eval()` |
| NN-002 | Linear layer (real weights, bias) | `runtime/axon_nn.c` | `W @ x + b` with grad support |
| NN-003 | Conv2d (im2col + matmul approach) | `runtime/axon_nn.c` | Basic 2D convolution |
| NN-004 | BatchNorm, LayerNorm | `runtime/axon_nn.c` | Running mean/var tracking |
| NN-005 | Sequential container | `runtime/axon_nn.c` | Chain modules in order |

### Sub-phase 6b: Optimizers (1.5 days)
| # | Task | File | Description |
|---|---|---|---|
| OPT-001 | SGD (with momentum) | `runtime/axon_optim.c` | `w -= lr * grad` (+ momentum buffer) |
| OPT-002 | Adam / AdamW | `runtime/axon_optim.c` | First/second moment, bias correction |
| OPT-003 | Learning rate schedulers | `runtime/axon_optim.c` | StepLR, CosineAnnealing |
| OPT-004 | `zero_grad()` on all parameters | `runtime/axon_optim.c` | Reset .grad to zero |

### Sub-phase 6c: Loss Functions & Data Loading (1.5 days)
| # | Task | File | Description |
|---|---|---|---|
| LOSS-001 | CrossEntropyLoss | `runtime/axon_loss.c` | LogSoftmax + NNLLoss |
| LOSS-002 | MSELoss, L1Loss | `runtime/axon_loss.c` | Common regression losses |
| DATA-001 | DataLoader (batch iterator) | `runtime/axon_data.c` | Batching + shuffling |
| DATA-002 | MNIST dataset reader | `runtime/axon_data.c` | Parse IDX format |

### Sub-phase 6d: Checkpointing & Utilities (1 day)
| # | Task | File | Description |
|---|---|---|---|
| UTIL-001 | Model save/load (binary format) | `runtime/axon_checkpoint.c` | Serialize all parameter tensors |
| UTIL-002 | Accuracy, F1, confusion matrix | `runtime/axon_metrics.c` | Common metrics |
| UTIL-003 | ONNX export (basic) | `runtime/axon_onnx.c` | Trace graph → ONNX protobuf |

**Gate**: This program trains and achieves >95% accuracy:
```axon
val net = Sequential.new([Linear.new(784, 256), ReLU.new(), Linear.new(256, 10)]);
val opt = Adam.new(net.parameters(), lr: 0.001);
// ... training loop on MNIST
```

---

## PHASE 7: MODEL VALIDATION & DOGFOODING (Week 7-8, ~5 days)

This is the existing Phase 14 spec. Prove the system works end-to-end.

### Tasks
| # | Task | Description | Success Criteria |
|---|---|---|---|
| V-001 | Linear regression on synthetic data | Converges to correct weights | Loss < 0.01 |
| V-002 | MLP on MNIST | 3-layer network trains | > 98% accuracy |
| V-003 | CNN on CIFAR-10 | Conv + pool + fc | > 85% accuracy |
| V-004 | Character-level transformer | Self-attention model | Generates coherent text |
| V-005 | Benchmark vs PyTorch | Same models, measure time | Within 5x of PyTorch (CPU) |
| V-006 | Memory profiling | No leaks in 100-epoch training | valgrind/ASAN clean |
| V-007 | Create example programs | `examples/` directory | 5+ documented examples |
| V-008 | Fix all bugs found during validation | - | Zero known crashes |

**Gate**: All 4 models train successfully. Performance benchmarked and documented.

---

## PHASE 8: PRODUCTION POLISH (Week 8-10, ~7 days)

This is the existing Phase 15 spec. Make it ready for real users.

### Sub-phase 8a: Error Message Quality (1.5 days)
| # | Task | Description |
|---|---|---|
| P-001 | Audit all E0xxx-E9xxx error codes | Every error has: clear message, exact span, `help:` suggestion |
| P-002 | Audit all runtime panics | Every panic prints: file, line, what went wrong, what to do |
| P-003 | Add "did you mean X?" suggestions everywhere | Levenshtein distance on identifiers |

### Sub-phase 8b: Documentation Rewrite (2 days)
| # | Task | Description |
|---|---|---|
| P-004 | Rewrite Getting Started guide with real working examples | Every code snippet compiles and runs |
| P-005 | Rewrite Language Tour with compilation outputs | Show compile errors, not just code |
| P-006 | Rewrite all 6 tutorials with tested code | MNIST, linear regression, transformer etc. |
| P-007 | API reference for all stdlib modules | Generated from doc comments |

### Sub-phase 8c: Installer & Distribution (1.5 days)
| # | Task | Description |
|---|---|---|
| P-008 | Windows installer (MSI or standalone) | `axon-setup.exe` |
| P-009 | Linux install script + .deb/.rpm packages | `curl -sSf install.sh \| sh` |
| P-010 | macOS install script + brew formula | `brew install axon` |
| P-011 | GitHub Releases with prebuilt binaries | Linux x86_64/aarch64, macOS, Windows |

### Sub-phase 8d: VS Code Extension Polish (1 day)
| # | Task | Description |
|---|---|---|
| P-012 | TextMate grammar for syntax highlighting | All keywords, operators, types |
| P-013 | LSP client wired to axon-lsp binary | Diagnostics, hover, go-to-def, completions |
| P-014 | Extension published to VS Code Marketplace | Users can install with one click |

### Sub-phase 8e: Final Hardening (1 day)
| # | Task | Description |
|---|---|---|
| P-015 | 48-hour fuzz campaign on all compiler passes | Grammar-based + random fuzzing |
| P-016 | Security audit of all unsafe code | Every `unsafe` block has `// SAFETY:` comment |
| P-017 | Full FR/NFR compliance re-verification | All 72 FR + 28 NFR pass with real runtime |
| P-018 | CHANGELOG, version bump to 1.0.0, tag release | `git tag v1.0.0` |

**Gate**: v1.0.0 tagged. Installers work on 3 platforms. VS Code extension published.

---

## CRITICAL PATH & TIMELINE

```
Week 1:  [Phase 0: Fix Build] → [Phase 1: Type System Gaps] → [Phase 2a: IR Bugs]
Week 2:  [Phase 2b-c: Runtime + E2E Tests]
Week 3:  [Phase 3a-b: Tensor Struct + Element-wise Ops]
Week 4:  [Phase 3c-d: Linalg + Compiler Integration]
Week 4:  [Phase 4a: Computation Graph]
Week 5:  [Phase 4b-c: Backward Pass + Verification]
Week 5-6: [Phase 5: GPU Backend] ← CAN SKIP FOR CPU-ONLY v1.0
Week 6-7: [Phase 6: Training Infrastructure]
Week 7-8: [Phase 7: Model Validation]
Week 8-10: [Phase 8: Production Polish]
```

### Minimum Viable 1.0 (CPU-only, skip GPU):
**~8 weeks** if working full-time with AI assistance.

### Full 1.0 (with GPU):
**~10 weeks** if CUDA toolkit is available.

---

## DEPENDENCY GRAPH

```
Phase 0 (Fix Build)
    └─→ Phase 1 (Type System Gaps)
         └─→ Phase 2 (E2E Compilation)
              └─→ Phase 3 (CPU Tensor Runtime)
                   ├─→ Phase 4 (Autograd)
                   │    ├─→ Phase 5 (GPU) ←── OPTIONAL for v1.0
                   │    └─→ Phase 6 (Training Infra)
                   │         └─→ Phase 7 (Validation)
                   │              └─→ Phase 8 (Polish + Release)
                   └─────────────────┘
```

---

## TASK COUNT SUMMARY

| Phase | Name | Tasks | Estimated Days |
|---|---|---|---|
| 0 | Fix Build | 5 | 0.5 |
| 1 | Type System Gaps | 11 | 3 |
| 2 | E2E Compilation | 13 | 4 |
| 3 | CPU Tensor Runtime | 19 | 5 |
| 4 | Autograd Engine | 15 | 5 |
| 5 | GPU Backend (optional) | 14 | 8 |
| 6 | Training Infrastructure | 13 | 6 |
| 7 | Model Validation | 8 | 5 |
| 8 | Production Polish | 18 | 7 |
| **TOTAL** | | **116** (102 without GPU) | **43.5 days** (35.5 without GPU) |

---

## RISK REGISTER

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| C runtime tensor math is slow | HIGH | Performance gap vs PyTorch | Use BLAS library (OpenBLAS) for matmul instead of naive loops |
| CUDA toolkit not available on dev machine | MEDIUM | Can't test GPU backend | Ship CPU-only v1.0, add GPU in v1.1 |
| Autograd gradient bugs (silent wrong results) | HIGH | Models don't converge | Numerical gradient checking against finite differences for every op |
| Borrow checker false positives on tensor code | MEDIUM | User frustration | AST-walk is conservative — document escape hatches |
| LLVM IR memory management bugs | HIGH | Segfaults in compiled programs | ASAN + valgrind on all E2E tests |
| MSVC linker issues recur | LOW | Build breaks | Permanent fix in `.cargo/config.toml` |

---

## IMPLEMENTATION PRIORITIES (What to do first)

### Day 1: Get the build working
1. Fix linker (Phase 0)
2. Verify all existing tests pass

### Week 1: Make the compiler correct
3. Fix LLVM IR bugs (Phase 2a) — these are known and documented
4. Complete C runtime (Phase 2b) — make compiled programs actually link and run
5. Wire dead code in type/borrow checker (Phase 1a)

### Week 2-3: Make tensors real
6. Implement C tensor runtime (Phase 3) — this is the biggest single chunk of work
7. Hook tensor ops into LLVM codegen

### Week 4-5: Make training work
8. Implement autograd (Phase 4) — the heart of ML
9. Implement optimizers and loss functions (Phase 6a-c)

### Week 6-7: Prove it works
10. Train MNIST, validate results (Phase 7)
11. GPU backend if time allows (Phase 5)

### Week 8+: Ship it
12. Polish error messages, docs, installers (Phase 8)
13. Tag v1.0.0

---

## SESSION HANDOFF NOTES

### For the next AI session working on this:
1. **Read this file first** — it has the complete state assessment
2. **Fix the linker FIRST** — nothing works without it
3. The Phase 9-15 specs in `docs/specs/` have extremely detailed task breakdowns — use them
4. The stdlib `.axon` files are signatures only — they type-check but don't compute
5. The `src/codegen/runtime.rs` contains stub C code — these need real implementations
6. All 6 LLVM IR bugs are documented in `docs/specs/phase9-e2e-compilation.md`
7. The crashed session created `axon-phase3-prompt.xml` at `C:\Users\tahaa\.copilot\session-state\` — it has context anchors but is focused on Phase 3 specifically
8. The worktree at `axon.worktrees/` is stale (pre-Phase 3) — ignore it
