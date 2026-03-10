# Phase 7 & 8 Fix Tasks

## Phase 4 — Code Generation Completeness (target: 90%+)

- [X] T200 [P] [P4-DOC] Add optimization levels documentation to docs/architecture/textual-ir-rationale.md
- [X] T201 [P] [P4-CODEGEN] Replace TensorOp placeholders with runtime call stubs in src/codegen/llvm.rs
- [X] T202 [P] [P4-RUNTIME] Add tensor op runtime declarations (matmul/add/sub/mul/div/reshape/transpose/broadcast) to src/codegen/runtime.rs
- [X] T203 [P] [P4-RUNTIME] Add C runtime stub implementations for tensor ops in src/codegen/runtime.rs
- [X] T204 [P] [P4-TEST] Update runtime function count in tests/codegen_tests.rs (18→26)

## Phase 5 — Standard Library Completeness (target: 90%+)

- [X] T210 [P] [P5-SYNC] Add Channel::bounded(capacity) to src/stdlib/sync.rs
- [X] T211 [P] [P5-SYNC] Add Channel::close, try_send, is_full, len, capacity to src/stdlib/sync.rs
- [X] T212 [P] [P5-SYNC] Add Mutex::unlock to src/stdlib/sync.rs
- [X] T213 [P] [P5-SYNC] Add AtomicBool, Condvar, Barrier types to src/stdlib/sync.rs
- [X] T214 [P] [P5-SYNC] Add AtomicI64::fetch_sub, compare_exchange, swap to src/stdlib/sync.rs
- [X] T220 [P] [P5-COLL] Add BTreeMap type with 14 methods to src/stdlib/collections.rs
- [X] T221 [P] [P5-COLL] Add Deque type with 17 methods to src/stdlib/collections.rs
- [X] T222 [P] [P5-COLL] Add 19 missing Vec methods (first, last, retain, dedup, windows, chunks, map, filter, etc.)
- [X] T223 [P] [P5-COLL] Add 4 missing HashMap methods (entry, or_insert, retain, get_or_insert_with)
- [X] T224 [P] [P5-COLL] Add 6 missing HashSet methods (symmetric_difference, is_subset, is_superset, is_disjoint, iter, retain)
- [X] T225 [P] [P5-COLL] Add 6 missing Option methods (filter, or_else, flatten, expect, zip, ok_or)
- [X] T226 [P] [P5-COLL] Add 7 missing Result methods (and_then, or_else, expect, expect_err, ok, err, flatten)
- [X] T230 [P] [P5-STRING] Add 17 missing String methods (split_at, splitn, is_ascii, pad_left, strip_prefix, count, etc.)
- [X] T231 [P] [P5-IO] Add 15 missing File/IO methods (seek, exists, copy, rename, path utils, etc.)
- [X] T232 [P] [P5-THREAD] Add ThreadPool type, sleep_ms, park/unpark, scope to src/stdlib/thread.rs
- [X] T233 [P] [P5-RANDOM] Add Rng::sample, next_bool, next_normal to src/stdlib/random.rs
- [X] T234 [P] [P5-TENSOR] Add 20 missing Tensor methods (ne, le, ge, slice, narrow, select, cat, stack, type casts, etc.)

## Phase 6 — AI Framework Stdlib Completeness (target: 90%+)

- [X] T100 [P] [P6-GRAD] Add 32 gradient rule functions to stdlib/autograd/ops.axon
- [X] T101 [P] [P6-GRAD] Register 32 gradient rule functions in src/stdlib/autograd.rs
- [X] T102 [P] [P6-NN] Add forward()/parameters() bodies to stdlib/nn/layers.axon (9 layers)
- [X] T103 [P] [P6-NN] Add forward() bodies to stdlib/nn/activation.axon (7 activations)
- [X] T104 [P] [P6-NN] Add forward() bodies to stdlib/nn/transformer.axon (3 types)
- [X] T105 [P] [P6-NN] Add forward() bodies to stdlib/nn/recurrent.axon (2 types)
- [X] T106 [P] [P6-NN] Add forward() bodies to stdlib/nn/sequential.axon, embedding.axon
- [X] T107 [P] [P6-LOSS] Add forward()/backward() bodies to stdlib/loss/mod.axon (11 types)
- [X] T108 [P] [P6-OPTIM] Add step()/zero_grad() bodies to stdlib/optim/adam.axon, sgd.axon
- [X] T109 [P] [P6-OPTIM] Add step()/get_lr() bodies to stdlib/optim/scheduler.axon
- [X] T110 [P] [P6-TRAIN] Add function bodies to stdlib/train/*.axon
- [X] T111 [P] [P6-METRICS] Add function bodies to stdlib/metrics/mod.axon
- [X] T112 [P] [P6-INIT] Add function bodies to stdlib/nn/init.axon
- [X] T113 [P] [P6-TEST] Add 39 new integration tests to tests/ai_framework_tests.rs

## Phase 7 — Tooling Fixes (target: 90%+)

- [X] T001 [P] [P7-LSP] Add `textDocument/references` handler — `src/lsp/handlers.rs`, `src/lsp/protocol.rs`, `src/lsp/server.rs`
- [X] T002 [P] [P7-LSP] Add `textDocument/rename` handler — `src/lsp/handlers.rs`, `src/lsp/protocol.rs`, `src/lsp/server.rs`
- [X] T003 [P] [P7-LSP] Add `textDocument/codeAction` handler — `src/lsp/handlers.rs`, `src/lsp/protocol.rs`, `src/lsp/server.rs`
- [X] T004 [P] [P7-LSP] Add references/rename/codeAction capabilities to initialization — `src/lsp/handlers.rs`
- [X] T005 [P] [P7-LSP] Add tests for references, rename, codeAction — `src/lsp/handlers.rs`
- [X] T006 [P] [P7-REPL] Add tab completion function for REPL — `src/repl.rs`
- [X] T007 [P] [P7-DOC] Add markdown generation to doc generator — `src/doc.rs`

## Phase 8 — Hardening Fixes (target: 90%+)

- [X] T008 [P] [P8-CI] Enhance CI workflow with clippy + fmt — `.github/workflows/ci.yml`
- [X] T009 [P] [P8-ERR] Add suggestions to E1002 and E2001 errors — `src/symbol.rs`, `src/typeck.rs`
- [X] T010 [P] [P8-DOC] Create compliance README index — `docs/compliance/README.md`
- [X] T011 [P] [P8-SEC] Create SECURITY.md — `SECURITY.md`
