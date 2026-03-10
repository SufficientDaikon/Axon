# Phase 7 & 8 Fix Tasks

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
