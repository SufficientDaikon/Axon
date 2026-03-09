# Phase 9: End-to-End Compilation — Implementation Specification

## 1. Overview

Phase 9 closes the gap between "the compiler generates LLVM IR" and "an Axon program actually runs." Today, `axonc build hello.axon` emits LLVM IR and invokes `clang`, but **no Axon program has ever been compiled, linked, and executed end-to-end**. The generated IR contains correctness bugs, the C runtime has stub functions, the linker invocation is incomplete (no runtime linkage), and there are zero tests that execute a compiled binary and check its output.

### Goal

```
axonc build hello.axon && ./hello
```

This must produce a correct, running native binary on Windows, Linux, and macOS.

### Deliverables

| Deliverable                        | Path                                | Purpose                                              |
| ---------------------------------- | ----------------------------------- | ---------------------------------------------------- |
| Fixed LLVM IR codegen              | `src/codegen/llvm.rs`               | Correct IR for all scalar MIR lowering paths         |
| Complete C runtime                 | `src/codegen/runtime.rs`            | Real implementations (not stubs) for all 12+ funcs   |
| Wired build pipeline               | `src/codegen/llvm.rs`, `src/main.rs`| Compile runtime.c → link with IR → produce binary    |
| E2E integration test harness       | `tests/e2e_tests.rs`               | Compile + run programs, assert stdout/exit code      |
| 15+ validated test programs        | `tests/e2e/`                        | Hello world through recursion, structs, enums        |
| Cross-platform support             | `src/codegen/llvm.rs`               | Windows (MSVC/MinGW), Linux (GCC), macOS (Clang)    |
| New error codes (E5xxx)            | `src/error.rs`, `src/codegen/`      | Structured errors for build/link/runtime failures    |

### Dependencies

- Phases 1–8 complete (863 tests passing).
- External: `clang` (or `gcc`/MSVC `cl.exe`) available on PATH for linking.
- No new Rust crate dependencies required (uses `std::process::Command`).

---

## 2. Current State Audit

### 2.1 Pipeline Flow (Today)

```
.axon source
  → lexer::Lexer::tokenize()          → Vec<Token>
  → parser::parse()                    → Program (AST)
  → typeck::check()                    → (TypedProgram, Vec<CompileError>)
  → mir::MirBuilder::build()           → MirProgram
  → codegen::llvm::LlvmCodegen::generate()  → String (LLVM IR text)
  → codegen::llvm::compile_ir_to_binary()   → invokes `clang <file>.ll -o <file>`
```

**Problem**: `compile_ir_to_binary()` passes only the `.ll` file to clang. The C runtime (`axon_runtime.c`) is never compiled or linked. Every `declare` in the IR (e.g., `@axon_print_i64`) is an unresolved external symbol at link time.

### 2.2 LLVM IR Codegen Bugs (Identified)

| # | Bug                                                      | Location                        | Impact                                |
|---|----------------------------------------------------------|---------------------------------|---------------------------------------|
| 1 | Runtime not linked: `compile_ir_to_binary` never compiles/links `axon_runtime.c` | `llvm.rs:1234-1262`       | **All programs fail to link**         |
| 2 | `main` returns `void` but C `main` must return `i32`    | `llvm.rs:emit_function`         | Undefined behavior / linker warning   |
| 3 | String constants emitted as `i8*` but `axon_print_str` expects `(i8*, i64)` — length never passed | `llvm.rs:emit_constant` (MirConstant::String) | Incorrect print_str calls |
| 4 | `Rvalue::Len` always returns 0                           | `llvm.rs:509-516`               | Array/string length broken            |
| 5 | `Rvalue::Aggregate` returns wrong type for `type_of_rvalue` (always `UNIT`) | `llvm.rs:1147`  | Struct/tuple assignments silently dropped |
| 6 | `emit_place_store` ignores projections (Field/Index)     | `llvm.rs:1029-1049`             | Struct field writes are no-ops        |
| 7 | `emit_place` ignores projections                         | `llvm.rs:959-966`               | GEP chains never generated for refs   |
| 8 | Enum aggregate only inserts tag, no payload              | `llvm.rs:885-894`               | Enum variants with data are broken    |
| 9 | Float constants use `%e` format instead of LLVM hex      | `llvm.rs:1054-1060`             | Precision loss for some float values  |
|10 | `Terminator::Call` emits callee as operand string, but function names need `@` prefix | `llvm.rs:411-452` | Calls to user functions fail |
|11 | `type_of_rvalue` for `Ref` returns `INT64` instead of pointer type | `llvm.rs:1146` | References have wrong type in IR |
|12 | `axon_print_bool` takes `i1` in IR decl but `int8_t` in C runtime | `runtime.rs:179` / `runtime.rs:97` | ABI mismatch |

### 2.3 C Runtime Stubs

| Function               | Status      | Notes                                      |
| ---------------------- | ----------- | ------------------------------------------ |
| `axon_alloc`           | ✅ Real     | Uses `_aligned_malloc`/`posix_memalign`    |
| `axon_dealloc`         | ✅ Real     | Uses `_aligned_free`/`free`                |
| `axon_tensor_alloc`    | ⚠️ Stub     | Ignores dtype size; no header struct       |
| `axon_tensor_free`     | ⚠️ Stub     | Just calls `free` — no shape array cleanup |
| `axon_tensor_shape_check` | ⚠️ Stub  | Always returns 1 (true)                    |
| `axon_device_transfer` | ⚠️ Stub     | Returns input unchanged                    |
| `axon_panic`           | ✅ Real     | Prints to stderr and exits                 |
| `axon_print_i64`       | ✅ Real     | `printf("%lld")`                           |
| `axon_print_f64`       | ✅ Real     | `printf("%g")`                             |
| `axon_print_str`       | ✅ Real     | `fwrite(ptr, 1, len, stdout)`              |
| `axon_print_bool`      | ✅ Real     | `printf("%s", value ? "true" : "false")`   |
| `axon_print_newline`   | ✅ Real     | `printf("\n")`                             |

### 2.4 MIR Builder Issues

| # | Issue                                                     | Location                    |
|---|-----------------------------------------------------------|-----------------------------|
| 1 | `lower_fn_call` lowers function operand as expression, but function identifiers resolve to `Unit` when not in `local_map` | `mir.rs:500-509` |
| 2 | `mangled_name` is always set to `decl.name` — no namespace mangling applied | `mir.rs:404` |
| 3 | Closures lowered as bare expression (captures not handled) | `mir.rs:658-662` |

---

## 3. Architecture & Design Decisions

### 3.1 Build Pipeline (Target)

```
.axon source
  → parse → typecheck → MIR → LLVM IR (.ll file)
  → generate_runtime_c_source() → write axon_runtime.c
  → clang -c axon_runtime.c -o axon_runtime.o          [compile runtime]
  → clang program.ll axon_runtime.o -o program          [link everything]
  → clean up temporaries (.ll, .c, .o)
```

All intermediate files are written to a temporary directory (`std::env::temp_dir()` / `<output>.build/`), then cleaned up on success. On `--emit-llvm`, only the `.ll` file is written and kept.

### 3.2 `main` Function Convention

Axon's `fn main()` is user-facing and returns `Unit` (void). The compiler must emit a wrapper:

```llvm
define i32 @main() {
entry:
  call void @_axon_main()     ; the user's fn main()
  ret i32 0
}
```

The user's `main` function is mangled to `_axon_main` (or `axon_user_main`) to avoid collision with C's `main`.

### 3.3 Calling Convention for `print`

Axon does not yet have a generic `print()` function. For Phase 9, we support calling runtime print functions directly. The MIR builder must recognize calls to `print_i64`, `print_f64`, `print_str`, `print_bool`, and `print_newline` and emit them as `@axon_print_*` calls. A minimal `print()` builtin that dispatches on argument type is introduced.

### 3.4 String Passing Convention

Strings in Axon are `{ i8*, i64, i64 }` (ptr, len, cap). String literals should be lowered to a global constant `[N x i8]` and the call to `axon_print_str` should pass `(ptr, len)` — not just `ptr`.

### 3.5 Cross-Platform Compilation

| Platform          | C Compiler        | Linker             | Runtime Compile Command                                     |
| ----------------- | ----------------- | ------------------ | ----------------------------------------------------------- |
| Linux (x86_64)    | `cc` or `gcc`     | system ld          | `cc -c axon_runtime.c -o axon_runtime.o`                    |
| macOS (arm64/x86) | `clang` (Xcode)   | Apple ld           | `clang -c axon_runtime.c -o axon_runtime.o`                 |
| Windows (MSVC)    | `cl.exe`          | `link.exe`         | `cl /c axon_runtime.c /Fo:axon_runtime.obj`                 |
| Windows (MinGW)   | `gcc`             | MinGW ld           | `gcc -c axon_runtime.c -o axon_runtime.o`                   |

The compiler attempts linkers in order: `clang` → `gcc` → `cc` → `cl.exe`. If none found, emit `E5001`.

### 3.6 Temporary File Management

```
<output_dir>/.axon_build/
  ├── program.ll           # LLVM IR
  ├── axon_runtime.c       # Generated C runtime
  ├── axon_runtime.o       # Compiled runtime object
  └── (cleaned on success)
```

On failure, the directory is preserved for debugging. `--keep-temps` flag preserves it always.

---

## 4. Error Codes

Phase 9 introduces error codes in the `E5xxx` range for build/link/runtime errors:

| Code   | Severity | Message                                              | Context                                       |
| ------ | -------- | ---------------------------------------------------- | --------------------------------------------- |
| E5001  | Error    | C compiler not found                                 | Neither clang, gcc, cc, nor cl.exe on PATH    |
| E5002  | Error    | Runtime compilation failed                           | `clang -c axon_runtime.c` returned non-zero   |
| E5003  | Error    | Linking failed                                       | Final link step returned non-zero             |
| E5004  | Error    | Failed to write intermediate file                    | I/O error writing .ll, .c, or .o              |
| E5005  | Error    | LLVM IR verification failed                          | IR rejected by clang (syntax/type errors)     |
| E5006  | Warning  | Optimization level not supported by compiler         | Fallback to -O0                               |
| E5007  | Error    | Unsupported target triple                            | Cross-compilation target not recognized       |
| E5008  | Error    | Runtime panic                                        | `axon_panic` invoked at runtime               |
| E5009  | Error    | Missing `main` function                              | No `fn main()` defined in source              |
| E5010  | Error    | `main` function has invalid signature                | `main` takes parameters or returns non-Unit   |

---

## 5. Detailed Task Breakdown

### Phase 9a: Build Pipeline Wiring (T278–T283)

#### T278 — Implement `compile_and_link()` orchestrator

**Description**: Replace the current `compile_ir_to_binary()` with a full pipeline that (1) writes the `.ll` file, (2) generates and writes `axon_runtime.c`, (3) compiles the runtime to an object file, (4) links the IR + runtime object into a final binary.

**Files**: `src/codegen/llvm.rs`

**Changes**:
- Add new public function `compile_and_link(ir: &str, output_path: &str, opt_level: OptLevel) -> Result<(), CompileError>`
- Create temp directory: `<output_path>.axon_build/`
- Write `<temp>/program.ll` and `<temp>/axon_runtime.c`
- Invoke: `clang -c <temp>/axon_runtime.c -o <temp>/axon_runtime.o`
- Invoke: `clang <temp>/program.ll <temp>/axon_runtime.o -o <output_path> -O<level>`
- On Windows: detect MSVC vs MinGW and adjust commands
- Clean temp dir on success
- Return `E5001`–`E5005` on failure

**Acceptance Criteria**:
- `compile_and_link()` produces a runnable binary when given valid IR
- Temp files cleaned on success, preserved on failure
- Error messages include stderr from clang on failure

---

#### T279 — Detect available C compiler cross-platform

**Description**: Implement a `detect_c_compiler() -> Result<CCompiler, CompileError>` function that probes for clang, gcc, cc, or cl.exe and returns the first available one along with its platform-specific flags.

**Files**: `src/codegen/llvm.rs`

**Changes**:
- Add `CCompiler` enum: `{ Clang, Gcc, Cc, Msvc }`
- Probe each by running `<compiler> --version` (or `cl.exe /?` for MSVC)
- Return platform-specific compile/link flag sets
- Emit `E5001` if none found

**Acceptance Criteria**:
- Returns correct compiler on each supported platform
- Unit test mocking disabled compiler (just test the detection logic)

---

#### T280 — Update `run_build` in `main.rs` to use new pipeline

**Description**: Replace the call to `compile_ir_to_binary()` in `run_build()` with `compile_and_link()`. Pass the runtime C source through the new orchestrator.

**Files**: `src/main.rs`

**Changes**:
- Replace `compile_ir_to_binary(&llvm_ir, &exe_output, opt_level)` call with `compile_and_link(&llvm_ir, &exe_output, opt_level)`
- Keep `--emit-llvm` and `--emit-obj` paths unchanged
- Add `--keep-temps` CLI flag

**Acceptance Criteria**:
- `axonc build hello.axon` invokes the full compile+link pipeline
- `--emit-llvm` still works as before
- `--keep-temps` preserves build directory

---

#### T281 — Add `--keep-temps` CLI flag

**Description**: Add a `--keep-temps` flag to the `Build` command that preserves intermediate `.ll`, `.c`, and `.o` files after compilation.

**Files**: `src/main.rs`

**Changes**:
- Add `#[arg(long)]` `keep_temps: bool` to `Commands::Build`
- Pass through to `compile_and_link()`

**Acceptance Criteria**:
- Without flag: temp dir deleted on success
- With flag: temp dir and all files preserved

---

#### T282 — Validate `main` function existence and signature

**Description**: Before codegen, check that the source contains exactly one `fn main()` with no parameters and `Unit` return type. Emit `E5009` / `E5010` otherwise.

**Files**: `src/codegen/llvm.rs` or `src/main.rs`

**Changes**:
- After type checking, scan `TypedProgram.items` for a function named `main`
- Verify signature: no params, return type is Unit (or Int64 for future compat)
- Emit structured `CompileError` with appropriate code

**Acceptance Criteria**:
- Missing main → `E5009` with helpful message
- `fn main(x: Int64)` → `E5010`
- `fn main() -> Int64 { ... }` → allowed (alternative convention)

---

#### T283 — Emit `main` wrapper for C ABI compatibility

**Description**: The user's `fn main()` returns void, but the C ABI requires `int main()`. Emit a C-ABI-compatible wrapper that calls the user's main and returns 0.

**Files**: `src/codegen/llvm.rs`

**Changes**:
- In `LlvmCodegen::generate()`, rename the user's `main` to `_axon_main` in the emitted IR
- Emit a new `@main` function:
  ```llvm
  define i32 @main() {
  entry:
    call void @_axon_main()
    ret i32 0
  }
  ```
- If user's main returns `Int64`, use: `%ret = call i64 @_axon_main(); %code = trunc i64 %ret to i32; ret i32 %code`

**Acceptance Criteria**:
- Emitted IR has a proper C-ABI `i32 @main()`
- User's logic is in `@_axon_main()`
- `./program` exits with code 0 for void main, user-provided code for Int64 main

---

### Phase 9b: LLVM IR Correctness Fixes (T284–T295)

#### T284 — Fix function call emission: callee name resolution

**Description**: `Terminator::Call` emits the callee operand as a raw string from `emit_operand()`. For user-defined functions, this resolves to a local place load instead of `@function_name`. Function calls must resolve to `@<mangled_name>`.

**Files**: `src/codegen/llvm.rs`, `src/mir.rs`

**Changes**:
- In `MirBuilder::lower_fn_call()`: when the function expression is an `Identifier` or `Path`, produce `Operand::Constant(MirConstant::String(mangled_name))` where `mangled_name` is the target function's mangled name
- In `LlvmCodegen::emit_terminator` for `Terminator::Call`: detect when callee is a `MirConstant::String` and emit as `@<name>` instead of loading from a local
- Add a function lookup table to `MirBuilder` mapping function names to their mangled names

**Acceptance Criteria**:
- `call void @some_function(...)` appears in IR for user-defined calls
- Recursive calls work
- Calls between multiple functions in same file work

---

#### T285 — Fix string literal emission for `print_str`

**Description**: Currently, `MirConstant::String` emits a GEP to the global's first byte (`i8*`), but `axon_print_str` requires `(i8*, i64)`. The length is never passed.

**Files**: `src/codegen/llvm.rs`

**Changes**:
- When lowering a call to `axon_print_str`, emit two arguments: the pointer from `intern_string()` GEP, and the length as `i64 <len>`
- Alternatively, modify `emit_constant` for `MirConstant::String` to return a `{ i8*, i64 }` struct and adjust call emission accordingly
- For the simpler approach: in the `Terminator::Call` emission, detect calls to `axon_print_str` and inject the string length argument

**Acceptance Criteria**:
- `call void @axon_print_str(i8* %ptr, i64 13)` appears in IR for `print_str("Hello, World!")`
- Strings with escape sequences have correct byte lengths
- Empty strings work (`i64 0`)

---

#### T286 — Fix `Rvalue::Aggregate` type inference

**Description**: `type_of_rvalue` returns `TypeId::UNIT` for all aggregates. This causes struct/tuple assignments to be silently dropped in `emit_assign` (the `ty_str == "{}" || ty_str == "void"` guard).

**Files**: `src/codegen/llvm.rs`

**Changes**:
- In `type_of_rvalue`, for `Rvalue::Aggregate`:
  - `AggregateKind::Tuple`: construct a `Type::Tuple` from field operand types
  - `AggregateKind::Struct(name)`: look up the struct type in the interner
  - `AggregateKind::Array`: construct `Type::Array` from first element type + field count
  - `AggregateKind::Enum(name, _)`: look up the enum type in the interner
- This requires the `LlvmCodegen` to either have access to the interner's type info or the aggregate rvalue to carry its result TypeId

**Acceptance Criteria**:
- `let p = Point { x: 1, y: 2 }` produces a non-void store in IR
- Tuple creation `(1, 2, 3)` stores to alloca correctly
- Array literal `[1, 2, 3]` stores to alloca correctly

---

#### T287 — Fix `emit_place_store` for projected places (field/index)

**Description**: `emit_place_store` currently ignores projections and always stores to the base local. Struct field writes and array element writes are no-ops.

**Files**: `src/codegen/llvm.rs`

**Changes**:
- When `place.projections` is non-empty, emit a GEP chain to compute the target address:
  ```llvm
  %field_ptr = getelementptr { i64, i64 }, { i64, i64 }* %_1, i32 0, i32 <field_idx>
  store i64 %value, i64* %field_ptr
  ```
- Handle `Projection::Field(idx)` → `getelementptr ... i32 0, i32 <idx>`
- Handle `Projection::Index(op)` → `getelementptr ... i64 <idx_val>`
- Handle `Projection::Deref` → load pointer then store through it

**Acceptance Criteria**:
- `point.x = 42` generates correct GEP + store
- `arr[0] = 10` generates correct GEP + store
- Chained projections work: `points[0].x = 1`

---

#### T288 — Fix `emit_place_load` projected type tracking

**Description**: In `emit_place_load`, after a `Projection::Field` GEP, `current_ty_str` is hardcoded to `"i8*"`. It should track the actual field type through the projection chain.

**Files**: `src/codegen/llvm.rs`

**Changes**:
- Track the current `TypeId` through projections:
  - `Projection::Field(idx)` on struct `{ A, B, C }` → set type to field `idx`'s type
  - `Projection::Index(_)` on array `[N x T]` → set type to `T`
  - `Projection::Deref` on `&T` → set type to `T`
- Requires a helper `fn type_after_projection(&self, base_ty: TypeId, proj: &Projection) -> TypeId`

**Acceptance Criteria**:
- `point.x` loads an `i64`, not an `i8*`
- `arr[0]` loads the correct element type
- Nested access `a.b.c` resolves correctly

---

#### T289 — Fix enum aggregate emission (tag + payload)

**Description**: `emit_aggregate` for `AggregateKind::Enum` only inserts the tag byte. Variant payloads are not stored.

**Files**: `src/codegen/llvm.rs`

**Changes**:
- Determine the full enum LLVM type: `{ i8, [<max_payload> x i8] }`
- Insert tag at index 0
- For payload: bitcast the payload area pointer and store fields
- Use `insertvalue` for simple payloads or store through GEP for complex ones

**Acceptance Criteria**:
- `Some(42)` produces: tag=1, payload=42
- `None` produces: tag=0
- Enum with struct variant stores all fields

---

#### T290 — Fix float constant format for LLVM IR

**Description**: Float constants use Rust's `{:.1}` or `{:e}` format, which can produce values like `1e1` that LLVM rejects. LLVM requires either decimal (`1.0`) or hex float (`0x3FF0000000000000`) format.

**Files**: `src/codegen/llvm.rs`

**Changes**:
- In `emit_constant` for `MirConstant::Float`, always emit in LLVM hex float format:
  ```rust
  let bits = v.to_bits();
  format!("0x{:016X}", bits)
  ```
- Alternatively, use `format!("{:.6e}", v)` which LLVM accepts, but hex is always exact

**Acceptance Criteria**:
- `3.14` emits as `0x40091EB851EB851F` (or acceptable LLVM decimal)
- `0.0` emits as `0.0` or `0x0000000000000000`
- `-1.5` emits correctly
- LLVM (clang) accepts all emitted float constants

---

#### T291 — Fix `Rvalue::Len` to compute real length

**Description**: `Rvalue::Len` always emits `add i64 0, 0` (returns 0). It should load the length from arrays (compile-time known) or strings (runtime `.len` field).

**Files**: `src/codegen/llvm.rs`

**Changes**:
- For `Type::Array { size, .. }`: emit `i64 <size>` directly (compile-time constant)
- For `Type::Primitive(PrimKind::String)`: GEP into the string struct's `len` field (index 1): `getelementptr { i8*, i64, i64 }, ...* %str, i32 0, i32 1` then load
- For other types: emit 0 with a TODO comment

**Acceptance Criteria**:
- `len([1, 2, 3])` emits `i64 3`
- `len("hello")` emits a load from the string's length field

---

#### T292 — Fix `type_of_rvalue` for `Ref` return type

**Description**: `type_of_rvalue` returns `TypeId::INT64` for `Rvalue::Ref`. It should return a reference/pointer type.

**Files**: `src/codegen/llvm.rs`

**Changes**:
- For `Rvalue::Ref { place, mutable }`: look up the place's type and intern a `Type::Reference { inner: <place_ty>, mutable: <mutable> }` in the interner
- Since `LlvmCodegen` holds an immutable reference to the interner, consider caching or pre-computing reference types
- Simpler approach: use the place's type directly since references are lowered as pointers and the LLVM type is `<T>*` regardless

**Acceptance Criteria**:
- `&x` where `x: Int64` has type `i64*` in IR, not `i64`

---

#### T293 — Fix `axon_print_bool` ABI mismatch

**Description**: The LLVM IR declares `@axon_print_bool(i1)` but the C runtime defines `void axon_print_bool(int8_t value)`. LLVM `i1` is 1 bit; C `int8_t` is 8 bits. This is an ABI mismatch that can cause garbage values.

**Files**: `src/codegen/runtime.rs`

**Changes**:
- Option A: Change the C function to accept `_Bool` (C99) / `bool` (with `stdbool.h`):
  ```c
  #include <stdbool.h>
  void axon_print_bool(bool value) { ... }
  ```
- Option B: Change the LLVM IR declaration to `declare void @axon_print_bool(i8)` and zero-extend `i1` to `i8` before calling
- **Choose Option B** for maximum portability (no C99 requirement)

**Acceptance Criteria**:
- `print_bool(true)` prints `true`, not garbage
- `print_bool(false)` prints `false`
- E2E test validates both

---

#### T294 — Implement `print()` builtin function

**Description**: Add a `print()` builtin that accepts any printable type and dispatches to the appropriate `axon_print_*` runtime function. This is the primary way Axon programs produce output.

**Files**: `src/mir.rs`, `src/codegen/llvm.rs`, `src/typeck.rs`

**Changes**:
- In the type checker (`typeck.rs`): recognize `print(expr)` as a builtin call, accept any primitive type or string
- In MIR builder (`mir.rs`): lower `print(expr)` to a `Terminator::Call` with the appropriate `axon_print_*` function based on the argument type:
  - `Int8..Int64`, `UInt8..UInt64` → `axon_print_i64` (with cast to i64 if needed)
  - `Float16..Float64` → `axon_print_f64` (with cast to f64 if needed)
  - `Bool` → `axon_print_bool`
  - `String` → `axon_print_str`
- Add `println()` variant that appends `axon_print_newline()`

**Acceptance Criteria**:
- `print(42)` calls `axon_print_i64(42)`
- `print(3.14)` calls `axon_print_f64(3.14)`
- `print("hello")` calls `axon_print_str("hello", 5)`
- `println(42)` calls `axon_print_i64(42)` then `axon_print_newline()`

---

#### T295 — Fix MIR function name resolution for calls

**Description**: `MirBuilder::lower_fn_call` lowers the function expression via `lower_expr`, which for an `Identifier("foo")` that isn't in `local_map` produces `Operand::Constant(MirConstant::Unit)`. This means all function calls emit `call @undef`.

**Files**: `src/mir.rs`

**Changes**:
- Add a `function_map: HashMap<String, String>` to `MirBuilder` mapping source function names to mangled names
- In `build()`, do a first pass over `TypedProgram.items` to populate `function_map` with all function declarations
- In `lower_fn_call()`: check if the function expression is an `Identifier` or `Path` that matches a known function name; if so, produce `Operand::Constant(MirConstant::String(mangled_name))`
- For builtin functions (`print`, `println`, `print_i64`, etc.), map to their `@axon_*` names

**Acceptance Criteria**:
- `foo()` in MIR produces `Call { func: Constant(String("foo")), ... }`
- Recursive `fn fib(n) { ... fib(n-1) ... }` resolves correctly
- Builtin calls resolve to runtime function names

---

### Phase 9c: C Runtime Completion (T296–T299)

#### T296 — Complete tensor runtime with proper TensorHeader struct

**Description**: Replace tensor stubs with a real `TensorHeader` struct and proper allocation/deallocation.

**Files**: `src/codegen/runtime.rs`

**Changes**:
- Define `TensorHeader` in the C runtime:
  ```c
  typedef struct {
      void* data;
      int64_t* shape;
      int64_t ndim;
      int8_t device;    // 0=CPU, 1=CUDA, 2=ROCm
      int32_t dtype;    // enum: 0=f16, 1=f32, 2=f64, 3=i32, 4=i64, ...
      int64_t refcount;
  } TensorHeader;
  ```
- `axon_tensor_alloc`: allocate `TensorHeader`, copy shape array, allocate data buffer with correct element size
- `axon_tensor_free`: free data, free shape array, free header
- `axon_tensor_shape_check`: actually compare shapes for the given operation (elementwise needs same shape, matmul needs compatible inner dims)

**Acceptance Criteria**:
- `axon_tensor_alloc(FLOAT32, [2,3], 2, CPU)` allocates 24 bytes of data
- `axon_tensor_free` frees all three allocations (no leaks)
- `axon_tensor_shape_check` catches incompatible shapes

---

#### T297 — Implement `axon_device_transfer` stub with error reporting

**Description**: Replace the no-op stub with a function that reports an error when GPU transfer is requested but no GPU runtime is available.

**Files**: `src/codegen/runtime.rs`

**Changes**:
- If `dst_device != 0` (not CPU) and no CUDA runtime is linked, call `axon_panic("GPU device transfer not available")`
- For CPU-to-CPU, return the input tensor unchanged

**Acceptance Criteria**:
- CPU-only programs work as before
- Attempting GPU transfer panics with clear message

---

#### T298 — Add `axon_print_char` and `axon_print_i32` to runtime

**Description**: Add print functions for `Char` (as Unicode codepoint) and `Int32` (common type).

**Files**: `src/codegen/runtime.rs`

**Changes**:
- Add `void axon_print_char(int32_t codepoint)` — prints the UTF-8 encoding of the codepoint
- Add `void axon_print_i32(int32_t value)` — prints as decimal
- Add LLVM IR declarations in `emit_runtime_declarations()`
- Update `RUNTIME_FUNCTIONS` array

**Acceptance Criteria**:
- `axon_print_char(65)` prints `A`
- `axon_print_char(0x1F600)` prints the 😀 emoji (or valid UTF-8)
- `axon_print_i32(42)` prints `42`

---

#### T299 — Add `fflush(stdout)` to runtime print functions

**Description**: On some platforms (especially Windows), stdout is line-buffered or fully buffered. Add `fflush(stdout)` after non-newline prints to ensure output appears immediately.

**Files**: `src/codegen/runtime.rs`

**Changes**:
- Add `fflush(stdout)` at the end of `axon_print_i64`, `axon_print_f64`, `axon_print_str`, `axon_print_bool`, `axon_print_newline`
- Alternatively, set stdout to unbuffered at runtime init: `setvbuf(stdout, NULL, _IONBF, 0)`

**Acceptance Criteria**:
- Output appears immediately when compiled program is piped or run in CI
- No output lost on program crash

---

### Phase 9d: E2E Test Harness (T300–T302)

#### T300 — Create E2E test harness

**Description**: Build a Rust test harness that compiles Axon source files to native binaries, runs them, and asserts on stdout output and exit code.

**Files**: `tests/e2e_tests.rs`

**Changes**:
- Create helper function:
  ```rust
  fn compile_and_run(source: &str) -> (String, i32) {
      // 1. Write source to temp .axon file
      // 2. Run full pipeline: parse → check → MIR → IR → compile+link
      // 3. Execute the resulting binary, capture stdout + exit code
      // 4. Clean up temp files
      // 5. Return (stdout, exit_code)
  }
  ```
- Use `std::process::Command` to run the compiled binary
- Set a timeout (5 seconds) for program execution to catch infinite loops
- Mark all E2E tests with `#[cfg_attr(not(feature = "e2e"), ignore)]` so they only run when clang is available

**Acceptance Criteria**:
- `compile_and_run("fn main() { print(42); }")` returns `("42", 0)`
- Tests are skipped gracefully when clang is not installed
- Temp files cleaned up after each test

---

#### T301 — Create E2E test source directory

**Description**: Create `tests/e2e/` directory with standalone `.axon` programs that serve as golden tests.

**Files**: `tests/e2e/*.axon`

**Changes**:
- Create the following test programs (each with expected output documented in a comment):

```
tests/e2e/
├── hello.axon              # print("Hello, World!\n") → "Hello, World!\n"
├── arithmetic.axon         # various int/float math   → "30\n15\n..."
├── if_else.axon            # conditional branching     → "yes\n"
├── while_loop.axon         # loop with counter         → "10\n"
├── functions.axon          # call user-defined fn      → "42\n"
├── recursion.axon          # fibonacci(10)             → "55\n"
├── structs.axon            # struct creation + access  → "3\n7\n"
├── enums.axon              # enum creation + match     → "red\n"
├── nested_if.axon          # if/else-if/else chain     → "medium\n"
├── unary_ops.axon          # negation, not             → "-5\ntrue\n"
├── comparisons.axon        # all comparison operators  → "true\nfalse\n..."
├── type_casts.axon         # int→float, float→int     → "42\n3\n"
├── multiple_fns.axon       # fn a() calls fn b()      → "hello from b\n"
├── tuples.axon             # tuple create + access     → "1\n2\n"
└── strings.axon            # string literals + print   → "hello world\n"
```

**Acceptance Criteria**:
- Each `.axon` file is syntactically valid and type-checks
- Expected output is documented in a header comment: `// expect: <output>`
- Programs cover all scalar MIR lowering paths that must work

---

#### T302 — Wire E2E tests to `cargo test`

**Description**: Create Rust tests in `tests/e2e_tests.rs` that iterate over `tests/e2e/*.axon`, compile each, run the binary, and assert stdout matches the `// expect:` comment.

**Files**: `tests/e2e_tests.rs`

**Changes**:
- Parse `// expect: <line>` comments from each `.axon` file to build expected output
- For each `.axon` file, call `compile_and_run()` and assert `stdout == expected`
- Also assert exit code == 0 (unless `// expect-exit: <code>` specified)
- Add individual `#[test]` functions for key programs (hello, arithmetic, recursion) for CI visibility
- Add a `#[test]` that scans the directory and runs all `.axon` files (catch-all)

**Acceptance Criteria**:
- `cargo test e2e` runs all E2E tests
- Each test reports which `.axon` file it's running
- Failures show expected vs actual output diff
- Tests skip with clear message when clang not available

---

### Phase 9e: Cross-Platform & Error Handling (T303–T305)

#### T303 — Windows MSVC linking support

**Description**: On Windows with MSVC toolchain, `clang` may not be available. Support compiling via `cl.exe` and linking via `link.exe`.

**Files**: `src/codegen/llvm.rs`

**Changes**:
- In `detect_c_compiler()`: on Windows, try `cl.exe` if clang/gcc not found
- For MSVC compilation:
  ```
  cl.exe /c axon_runtime.c /Fo:axon_runtime.obj
  clang -S -emit-llvm ... (or use llc if available)
  ```
- For MSVC linking:
  ```
  link.exe program.obj axon_runtime.obj /OUT:program.exe
  ```
- Set target triple to `x86_64-pc-windows-msvc` on Windows
- Handle both `.obj` (MSVC) and `.o` (MinGW) object file extensions

**Acceptance Criteria**:
- `axonc build hello.axon` works on Windows with only MSVC installed
- `axonc build hello.axon` works on Windows with MinGW installed
- Target triple matches the compilation environment

---

#### T304 — Error reporting with clang stderr forwarding

**Description**: When clang fails, the current error message is `"clang failed: <stderr>"`. Improve this to parse clang's output and produce structured `CompileError` messages.

**Files**: `src/codegen/llvm.rs`, `src/error.rs`

**Changes**:
- Parse clang stderr for common patterns:
  - `error: ` lines → extract message
  - `undefined reference to` → suggest missing runtime function
  - `cannot find -l` → suggest missing library
- Wrap in `CompileError` with code `E5003` (link error) or `E5005` (IR error)
- Include the raw clang output as a note on the error
- On "command not found", emit `E5001` with installation instructions per platform

**Acceptance Criteria**:
- Clang errors are wrapped in structured `CompileError`
- "undefined reference" errors mention which symbol is missing
- Users get actionable installation instructions for their platform

---

#### T305 — Runtime panic with source location

**Description**: Improve `axon_panic()` to include source file and line number. Wire the `Assert` terminator to pass the correct source location.

**Files**: `src/codegen/runtime.rs`, `src/codegen/llvm.rs`

**Changes**:
- `axon_panic` already accepts `(msg, file, line)` — ensure callers pass real values
- In `emit_terminator` for `Terminator::Assert`: pass the actual source file name (from the module being compiled) and the span's line number from the MIR
- Add `file: String` and `line: u32` fields to `Terminator::Assert` in MIR
- Update `MirBuilder` to propagate span info into Assert terminators

**Acceptance Criteria**:
- Runtime assertion failure prints: `axon panic at hello.axon:15: index out of bounds`
- Stack trace includes the file and line where the panic originated
- Panic calls `exit(1)` (non-zero exit code)

---

## 6. Test Plan

### 6.1 Unit Tests (in-module `#[cfg(test)]`)

| Module                 | Test                                                    | Validates                        |
| ---------------------- | ------------------------------------------------------- | -------------------------------- |
| `codegen/llvm.rs`      | `test_main_wrapper_emitted`                             | C-ABI `i32 @main()` wrapper     |
| `codegen/llvm.rs`      | `test_string_constant_with_length`                      | String lit → `(i8*, i64)`       |
| `codegen/llvm.rs`      | `test_aggregate_type_inference`                         | Struct/tuple type != Unit        |
| `codegen/llvm.rs`      | `test_projected_store_gep`                              | Field store emits GEP + store   |
| `codegen/llvm.rs`      | `test_projected_load_tracks_type`                       | Field load has correct type      |
| `codegen/llvm.rs`      | `test_enum_tag_and_payload`                             | Enum insertvalue has payload     |
| `codegen/llvm.rs`      | `test_float_hex_format`                                 | Float emitted as `0x...`        |
| `codegen/llvm.rs`      | `test_len_array_constant`                               | `Len` on array → compile-time N |
| `codegen/llvm.rs`      | `test_bool_print_zext`                                  | `i1` → `i8` before print call   |
| `codegen/llvm.rs`      | `test_function_call_uses_at_prefix`                     | `call ... @foo(...)` syntax     |
| `codegen/runtime.rs`   | `test_c_runtime_has_tensor_header`                      | TensorHeader struct in C source |
| `codegen/runtime.rs`   | `test_c_runtime_has_fflush`                             | fflush after prints             |
| `codegen/runtime.rs`   | `test_c_runtime_has_print_char`                         | axon_print_char declaration     |
| `mir.rs`               | `test_function_map_populated`                           | Functions resolvable by name    |
| `mir.rs`               | `test_print_builtin_lowered`                            | `print(42)` → `axon_print_i64` |
| `main.rs`              | (via CLI) `test_build_missing_main_error`               | E5009 emitted                    |

### 6.2 Integration Tests (compile IR, don't execute)

| Test File              | Test                                                    | Validates                        |
| ---------------------- | ------------------------------------------------------- | -------------------------------- |
| `codegen_tests.rs`     | `test_full_pipeline_hello_world_ir`                     | IR for hello world is valid      |
| `codegen_tests.rs`     | `test_full_pipeline_function_call_ir`                   | IR has `call @func(...)` syntax  |
| `codegen_tests.rs`     | `test_full_pipeline_struct_ir`                          | IR has struct alloca + GEP       |

### 6.3 End-to-End Tests (compile AND execute)

| Test                           | Source Program          | Expected stdout              | Exit code |
| ------------------------------ | ----------------------- | ---------------------------- | --------- |
| `e2e_hello_world`              | `hello.axon`            | `Hello, World!\n`            | 0         |
| `e2e_arithmetic`               | `arithmetic.axon`       | Integer math results         | 0         |
| `e2e_if_else`                  | `if_else.axon`          | Correct branch taken         | 0         |
| `e2e_while_loop`               | `while_loop.axon`       | Loop counter result          | 0         |
| `e2e_user_functions`           | `functions.axon`        | Function return values       | 0         |
| `e2e_recursion`                | `recursion.axon`        | `55` (fibonacci)             | 0         |
| `e2e_structs`                  | `structs.axon`          | Field access results         | 0         |
| `e2e_enums`                    | `enums.axon`            | Match arm results            | 0         |
| `e2e_nested_if`                | `nested_if.axon`        | Correct else-if branch       | 0         |
| `e2e_unary_ops`                | `unary_ops.axon`        | Negation and not results     | 0         |
| `e2e_comparisons`              | `comparisons.axon`      | All comparison results       | 0         |
| `e2e_type_casts`               | `type_casts.axon`       | Cast results                 | 0         |
| `e2e_multiple_functions`       | `multiple_fns.axon`     | Cross-function calls         | 0         |
| `e2e_tuples`                   | `tuples.axon`           | Tuple element access         | 0         |
| `e2e_strings`                  | `strings.axon`          | String output                | 0         |

### 6.4 Negative Tests

| Test                           | Input                         | Expected Error           |
| ------------------------------ | ----------------------------- | ------------------------ |
| `e2e_no_main`                  | `fn foo() {}`                 | E5009                    |
| `e2e_bad_main_sig`             | `fn main(x: Int64) {}`       | E5010                    |
| `e2e_no_clang`                 | (mock missing clang)          | E5001                    |

---

## 7. Implementation Order

Tasks should be implemented in this order to minimize blocked dependencies:

```
Phase 9a (Pipeline):
  T279 → T278 → T280 → T281 → T282 → T283

Phase 9b (IR Fixes):
  T295 → T284 → T285 → T290 → T293 → T294 → T286 → T287 → T288 → T289 → T291 → T292

Phase 9c (Runtime):
  T296 → T297 → T298 → T299

Phase 9d (Tests):
  T300 → T301 → T302

Phase 9e (Platform):
  T303 → T304 → T305
```

**Critical path**: T279 → T278 → T283 → T295 → T284 → T285 → T294 → T300 → T301 → T302

The earliest an E2E test can pass is after T283 (main wrapper) + T284 (function calls) + T285 (string printing) + T278 (linked runtime). A minimal `fn main() { }` should work after T278 + T283 alone.

---

## 8. Exit Criteria

Phase 9 is complete when **all** of the following are true:

- [ ] `axonc build hello.axon && ./hello` prints `Hello, World!` and exits 0
- [ ] `axonc build fib.axon && ./fib` prints `55` (fibonacci of 10) and exits 0
- [ ] All 15 E2E test programs compile, run, and produce expected output
- [ ] `cargo test` passes (all 863 existing tests + new unit tests + E2E tests)
- [ ] Build works on at least 2 of: Windows, Linux, macOS
- [ ] Missing clang produces helpful error `E5001` (not a panic)
- [ ] Runtime assertion failures print file + line + message to stderr
- [ ] No intermediate files left behind after successful build
- [ ] `--emit-llvm` still works correctly
- [ ] `--emit-mir` still works correctly
- [ ] `--emit-obj` still works correctly (object links with runtime)
- [ ] No regressions in existing codegen IR tests (`tests/codegen_tests.rs`)

---

## 9. Risks & Mitigations

| Risk                                              | Impact | Mitigation                                                       |
| ------------------------------------------------- | ------ | ---------------------------------------------------------------- |
| LLVM IR type mismatches cause clang crashes        | High   | Validate IR with `llc -verify-each` in test harness              |
| C ABI differences between platforms                | Medium | Use `i64`/`double`/`i8*` everywhere (C-compatible types)        |
| Existing tests break due to IR format changes      | Medium | Run `cargo test` after every change; fix codegen_tests.rs first |
| `clang` not available in CI                        | Medium | Use `#[ignore]` for E2E tests; detect in CI and install         |
| Float constant precision issues                   | Low    | Use hex float format (exact representation)                     |
| Windows path separators in temp file handling      | Low    | Use `std::path::PathBuf` exclusively, never raw string concat   |
| Struct field ordering mismatch between MIR and IR  | Medium | Verify field order matches declaration order in type interner    |

---

## 10. Files Modified Summary

| File                        | Changes                                                              |
| --------------------------- | -------------------------------------------------------------------- |
| `src/codegen/llvm.rs`       | Fix 12 bugs; add `compile_and_link()`, main wrapper, GEP chains     |
| `src/codegen/runtime.rs`    | Complete tensor runtime; add `print_char`/`print_i32`; add fflush   |
| `src/mir.rs`                | Add function_map; fix fn call lowering; add print builtin lowering  |
| `src/main.rs`               | Wire `compile_and_link()`; add `--keep-temps`; validate main fn     |
| `src/typeck.rs`             | Recognize `print()`/`println()` as builtins                         |
| `src/error.rs`              | (no changes — error codes are just strings)                          |
| `tests/e2e_tests.rs`        | New file: E2E test harness + 15 test cases                          |
| `tests/e2e/*.axon`          | New directory: 15 test programs                                      |
| `tests/codegen_tests.rs`    | Update existing tests if IR format changes break them                |
| `tasks.md`                  | Add Phase 9 task entries (T278–T305)                                 |
