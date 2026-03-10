# ADR-001: Textual LLVM IR over Programmatic IR Generation

**Status:** Accepted  
**Date:** 2024-12-01  
**Decision Makers:** Axon Core Team

## Context

The Axon compiler needs to emit LLVM IR for native code generation. Two approaches
were considered:

1. **Programmatic IR via inkwell/llvm-sys** — Rust bindings to the LLVM C++ API
   that construct IR in-memory through builder objects.
2. **Textual LLVM IR** — Emit `.ll` files as text, then invoke `clang` to compile
   and optimize them.

## Decision

**Use textual LLVM IR for code generation in Axon v1.0.**

The compiler emits LLVM IR as formatted text strings (`.ll` files), which are then
compiled to object code and executables by invoking `clang` as an external tool.

## Rationale

### Simplicity
Textual IR is plain text — easy to inspect, diff, and understand. There is no
impedance mismatch between what the compiler produces and what a developer reads
when debugging codegen issues. The emitted IR can be viewed in any text editor.

### No Native Dependency on LLVM C++ Libraries
Using `inkwell` or `llvm-sys` requires linking against LLVM's C++ libraries at
build time. This introduces:
- Massive build complexity (LLVM is 100M+ LOC of C++)
- Platform-specific configuration for LLVM discovery (`llvm-config`)
- Long compile times for the compiler itself
- Version coupling (must match exact LLVM version)

Textual IR avoids all of this. The only runtime dependency is `clang` on the PATH.

### clang Handles Optimization Better
By passing textual IR through `clang -O2`, we get LLVM's full optimization pipeline
(mem2reg, SROA, GVN, loop unrolling, inlining, etc.) without writing any pass
management code. This is strictly better than hand-rolling LLVM pass pipelines
through the C API, which is error-prone and requires deep LLVM expertise.

### Portability
Textual IR works on any platform where `clang` is available, which is essentially
every platform LLVM supports. There are no ABI compatibility concerns between
the compiler binary and the LLVM libraries.

## Trade-offs

### No DWARF Debug Info from IR Level
Textual IR does not directly support emitting DWARF metadata inline (source maps,
variable locations, etc.). However, this can be partially addressed by:
- Adding `!dbg` metadata annotations to the textual IR
- Passing `-g` flags to `clang` for debug builds
- This is a v2.0 concern — correctness comes first

### Performance Overhead
There is a minor overhead from serializing IR to text, writing to disk, and
invoking `clang` as a subprocess. For the scale of programs Axon targets in v1.0,
this overhead is negligible (typically <100ms for the clang invocation).

### No Incremental Compilation at IR Level
Each compilation produces a fresh `.ll` file. Incremental compilation would
require an in-memory IR representation. This is acceptable for v1.0.

## Alternatives Considered

| Approach | Pros | Cons |
|----------|------|------|
| inkwell | Type-safe Rust API, in-memory IR | LLVM build dependency, version coupling |
| llvm-sys | Low-level control, full LLVM access | Unsafe FFI, complex setup |
| Cranelift | Pure Rust, fast compilation | Less optimized output, smaller ecosystem |
| Textual IR | Simple, portable, debuggable | Subprocess overhead, no debug info (yet) |

## Consequences

- The `src/codegen/llvm.rs` module emits string-based LLVM IR
- `clang` must be installed on the user's system
- Debug info support is deferred to a later phase
- Migration to inkwell remains possible for v2.0 if performance demands it

## Future Plan

- **v1.0:** Keep textual IR — focus on correctness and feature completeness
- **v1.x:** Add `!dbg` metadata annotations for source-level debugging
- **v2.0:** Evaluate inkwell for in-memory IR if compilation speed or debug info
  quality becomes a bottleneck

## Optimization Passes

Axon **does not implement its own optimization passes** for v1.0. Instead, all
optimization is delegated to `clang` via its standard flag interface:

| Flag    | Description                                      |
|---------|--------------------------------------------------|
| `-O0`   | No optimization (fastest compilation, for debug) |
| `-O1`   | Basic optimizations (inlining, mem2reg)          |
| `-O2`   | Standard optimizations (GVN, loop unroll, SROA)  |
| `-O3`   | Aggressive optimizations (vectorization, etc.)   |
| `-Os`   | Size-optimized                                   |
| `-Oz`   | Minimum size                                     |

The Axon compiler passes the user-requested optimization level directly to `clang`
when compiling the generated `.ll` file:

```
clang -O2 output.ll -o output
```

This is sufficient for v1.0 because:

1. **LLVM's optimization pipeline is world-class** — it handles mem2reg, SROA, GVN,
   loop invariant code motion, inlining, dead code elimination, and hundreds of
   other passes automatically.
2. **No redundant work** — implementing custom MIR-level optimizations would
   duplicate what LLVM already does better.
3. **Correctness first** — custom optimization passes are a source of subtle bugs.
   Delegating to a battle-tested optimizer reduces our bug surface.

The only MIR-level "optimization" Axon performs is **constant folding** during
lowering (e.g., `3 + 4` → `7`), which is safe and trivial. All other optimization
is left to `clang`.

## Debug Information

The textual IR approach does **not** emit DWARF debug info directly. LLVM's debug
metadata (`!dbg`, `!DILocation`, etc.) can be included as text in the `.ll` file,
but the current Axon codegen does not generate these annotations.

### Current Workaround

Use `clang -g` on the generated `.ll` file to get basic source mapping. While this
does not map back to Axon source lines (it maps to the IR), it provides enough
information for tools like `gdb` and `lldb` to step through the generated code at
the IR level.

### Full Debug Info Path

Emitting proper Axon-source-to-DWARF debug info (so that `gdb` shows Axon source
lines, variable names, and types) requires one of:

1. **Inkwell/llvm-sys** — Use the LLVM `DIBuilder` API to emit proper debug
   metadata programmatically. This is the approach `rustc` uses.
2. **Custom DWARF emitter** — Generate DWARF sections directly in the object file.
   This is complex but avoids the LLVM library dependency.
3. **Textual `!dbg` metadata** — Emit `!dbg` annotations in the `.ll` file that
   reference `!DILocation` and `!DISubprogram` metadata entries. This is feasible
   but tedious and error-prone to generate as text.

### Decision

Full debug info support is **deferred to post-v1.0**. The priority for v1.0 is
correctness and feature completeness. Debug info will be addressed when either
inkwell is adopted (v2.0) or when textual `!dbg` metadata generation is implemented
(v1.x).
