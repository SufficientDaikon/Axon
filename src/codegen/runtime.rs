use serde::Serialize;

/// A runtime function that the Axon compiler may emit calls to.
#[derive(Debug, Clone, Serialize)]
pub struct RuntimeFunction {
    pub name: &'static str,
    pub llvm_name: &'static str,
    pub description: &'static str,
}

/// All Axon runtime functions.
pub const RUNTIME_FUNCTIONS: &[RuntimeFunction] = &[
    RuntimeFunction {
        name: "alloc",
        llvm_name: "axon_alloc",
        description: "Heap allocation: (size: i64, align: i64) -> *u8",
    },
    RuntimeFunction {
        name: "dealloc",
        llvm_name: "axon_dealloc",
        description: "Heap deallocation: (ptr: *u8, size: i64, align: i64) -> void",
    },
    RuntimeFunction {
        name: "tensor_alloc",
        llvm_name: "axon_tensor_alloc",
        description: "Tensor allocation: (dtype: i32, shape_ptr: *i64, ndim: i64, device: i8) -> *TensorHeader",
    },
    RuntimeFunction {
        name: "tensor_free",
        llvm_name: "axon_tensor_free",
        description: "Tensor deallocation: (tensor: *TensorHeader) -> void",
    },
    RuntimeFunction {
        name: "tensor_shape_check",
        llvm_name: "axon_tensor_shape_check",
        description: "Runtime shape assertion: (a: *TensorHeader, b: *TensorHeader, op: i32) -> i1",
    },
    RuntimeFunction {
        name: "device_transfer",
        llvm_name: "axon_device_transfer",
        description: "CPU<->GPU transfer: (src: *TensorHeader, dst_device: i8) -> *TensorHeader",
    },
    RuntimeFunction {
        name: "panic",
        llvm_name: "axon_panic",
        description: "Panic handler: (msg: *i8, file: *i8, line: i32) -> void (noreturn)",
    },
    RuntimeFunction {
        name: "print_i64",
        llvm_name: "axon_print_i64",
        description: "Print i64 to stdout: (value: i64) -> void",
    },
    RuntimeFunction {
        name: "print_f64",
        llvm_name: "axon_print_f64",
        description: "Print f64 to stdout: (value: f64) -> void",
    },
    RuntimeFunction {
        name: "print_str",
        llvm_name: "axon_print_str",
        description: "Print string to stdout: (ptr: *i8, len: i64) -> void",
    },
    RuntimeFunction {
        name: "print_bool",
        llvm_name: "axon_print_bool",
        description: "Print bool to stdout: (value: i1) -> void",
    },
    RuntimeFunction {
        name: "print_newline",
        llvm_name: "axon_print_newline",
        description: "Print newline to stdout: () -> void",
    },
    RuntimeFunction {
        name: "print_i32",
        llvm_name: "axon_print_i32",
        description: "Print i32 to stdout: (value: i32) -> void",
    },
    RuntimeFunction {
        name: "print_f32",
        llvm_name: "axon_print_f32",
        description: "Print f32 to stdout: (value: f32) -> void",
    },
    RuntimeFunction {
        name: "print_char",
        llvm_name: "axon_print_char",
        description: "Print char to stdout: (codepoint: i32) -> void",
    },
    // Reference counting (for future borrow checker integration)
    RuntimeFunction {
        name: "rc_inc",
        llvm_name: "axon_rc_inc",
        description: "Increment reference count: (ptr: *u8) -> void",
    },
    RuntimeFunction {
        name: "rc_dec",
        llvm_name: "axon_rc_dec",
        description: "Decrement reference count (frees at zero): (ptr: *u8) -> void",
    },
    RuntimeFunction {
        name: "rc_count",
        llvm_name: "axon_rc_count",
        description: "Get current reference count: (ptr: *u8) -> i64",
    },
];

/// Generate LLVM IR declarations for all runtime functions.
pub fn emit_runtime_declarations() -> String {
    let mut ir = String::new();
    ir.push_str("; Axon Runtime Library Declarations\n");

    // Memory
    ir.push_str("declare i8* @axon_alloc(i64, i64)\n");
    ir.push_str("declare void @axon_dealloc(i8*, i64, i64)\n");

    // Tensor
    ir.push_str("declare i8* @axon_tensor_alloc(i32, i64*, i64, i8)\n");
    ir.push_str("declare void @axon_tensor_free(i8*)\n");
    ir.push_str("declare i1 @axon_tensor_shape_check(i8*, i8*, i32)\n");
    ir.push_str("declare i8* @axon_device_transfer(i8*, i8)\n");

    // Panic
    ir.push_str("declare void @axon_panic(i8*, i8*, i32) noreturn\n");

    // I/O (for basic programs)
    ir.push_str("declare void @axon_print_i64(i64)\n");
    ir.push_str("declare void @axon_print_f64(double)\n");
    ir.push_str("declare void @axon_print_str(i8*, i64)\n");
    ir.push_str("declare void @axon_print_bool(i1)\n");
    ir.push_str("declare void @axon_print_newline()\n");
    ir.push_str("declare void @axon_print_i32(i32)\n");
    ir.push_str("declare void @axon_print_f32(float)\n");
    ir.push_str("declare void @axon_print_char(i32)\n");

    // Reference counting
    ir.push_str("\n; Reference counting runtime\n");
    ir.push_str("declare void @axon_rc_inc(i8*)\n");
    ir.push_str("declare void @axon_rc_dec(i8*)\n");
    ir.push_str("declare i64 @axon_rc_count(i8*)\n");

    // C stdlib (for printf-based I/O)
    ir.push_str("\n; C standard library\n");
    ir.push_str("declare i32 @printf(i8*, ...)\n");
    ir.push_str("declare i32 @puts(i8*)\n");
    ir.push_str("declare void @exit(i32) noreturn\n");

    ir
}

/// Generate a minimal C runtime library source file that implements
/// the basic runtime functions for CPU execution.
pub fn generate_runtime_c_source() -> String {
    r#"// axon_runtime.c — Axon Compiler Runtime Library
// Compile with: clang -c axon_runtime.c -o axon_runtime.o

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdint.h>
#include <stdbool.h>

#ifdef _MSC_VER
#pragma section(".CRT$XCU", read)
static void __cdecl axon_init_io(void);
__declspec(allocate(".CRT$XCU")) static void (__cdecl *axon_init_io_ptr)(void) = axon_init_io;
static void __cdecl axon_init_io(void) {
    setvbuf(stdout, NULL, _IONBF, 0);
}
#else
__attribute__((constructor))
static void axon_init_io(void) {
    setvbuf(stdout, NULL, _IONBF, 0);
}
#endif

void* axon_alloc(int64_t size, int64_t align) {
    void* ptr;
#ifdef _WIN32
    ptr = _aligned_malloc(size, align);
#else
    if (posix_memalign(&ptr, align, size) != 0) ptr = NULL;
#endif
    return ptr;
}

void axon_dealloc(void* ptr, int64_t size, int64_t align) {
    (void)size; (void)align;
#ifdef _WIN32
    _aligned_free(ptr);
#else
    free(ptr);
#endif
}

size_t axon_dtype_size(uint8_t dtype) {
    switch(dtype) {
        case 0: return sizeof(float);     // f32
        case 1: return sizeof(double);    // f64
        case 2: return sizeof(int32_t);   // i32
        case 3: return sizeof(int64_t);   // i64
        case 4: return sizeof(uint8_t);   // bool/u8
        default: return sizeof(float);
    }
}

void* axon_tensor_alloc(int32_t dtype, int64_t* shape, int64_t ndim, int8_t device) {
    (void)device;
    // Calculate total number of elements
    int64_t total = 1;
    for (int64_t i = 0; i < ndim; i++) total *= shape[i];
    size_t elem_size = axon_dtype_size((uint8_t)dtype);
    return malloc((size_t)total * elem_size);
}

void axon_tensor_free(void* tensor) {
    free(tensor);
}

int8_t axon_tensor_shape_check(void* a, void* b, int32_t op) {
    (void)a; (void)b; (void)op;
    return 1;  // stub: always passes
}

void* axon_device_transfer(void* src, int8_t dst_device) {
    if (dst_device != 0) {
        fprintf(stderr, "axon panic: GPU device transfer requested but no GPU runtime linked\n");
        fflush(stderr);
        exit(1);
    }
    return src;  // CPU-to-CPU: no-op
}

void axon_panic(const char* msg, const char* file, int32_t line) {
    fprintf(stderr, "axon panic at %s:%d: %s\n", file, line, msg);
    fflush(stderr);
    exit(1);
}

void axon_print_i64(int64_t value) {
    printf("%lld", (long long)value);
    fflush(stdout);
}

void axon_print_f64(double value) {
    printf("%g", value);
    fflush(stdout);
}

void axon_print_str(const char* ptr, int64_t len) {
    fwrite(ptr, 1, len, stdout);
    fflush(stdout);
}

void axon_print_bool(int8_t value) {
    printf("%s", (value & 1) ? "true" : "false");
    fflush(stdout);
}

void axon_print_newline(void) {
    printf("\n");
    fflush(stdout);
}

void axon_print_i32(int32_t value) {
    printf("%d", value);
    fflush(stdout);
}

void axon_print_char(int32_t codepoint) {
    if (codepoint < 0x80) {
        putchar(codepoint);
    } else {
        printf("%c", (char)codepoint);
    }
    fflush(stdout);
}

void axon_print_f32(float value) {
    printf("%g", value);
    fflush(stdout);
}

// ── Reference Counting ──────────────────────────────────────
// Refcount is stored as an int64_t in the 8 bytes preceding
// the user-visible pointer (allocation header layout).

void axon_rc_inc(void* ptr) {
    if (!ptr) return;
    int64_t* rc = ((int64_t*)ptr) - 1;
    (*rc)++;
}

void axon_rc_dec(void* ptr) {
    if (!ptr) return;
    int64_t* rc = ((int64_t*)ptr) - 1;
    if (--(*rc) <= 0) {
        free(rc);  // free from header start
    }
}

int64_t axon_rc_count(void* ptr) {
    if (!ptr) return 0;
    int64_t* rc = ((int64_t*)ptr) - 1;
    return *rc;
}
"#.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runtime_functions_count() {
        assert_eq!(RUNTIME_FUNCTIONS.len(), 18);
    }

    #[test]
    fn runtime_functions_have_names() {
        for func in RUNTIME_FUNCTIONS {
            assert!(!func.name.is_empty(), "name should not be empty");
            assert!(!func.llvm_name.is_empty(), "llvm_name should not be empty");
            assert!(!func.description.is_empty(), "description should not be empty");
        }
    }

    #[test]
    fn runtime_llvm_names_prefixed() {
        for func in RUNTIME_FUNCTIONS {
            assert!(
                func.llvm_name.starts_with("axon_"),
                "llvm_name '{}' should start with 'axon_'",
                func.llvm_name
            );
        }
    }

    #[test]
    fn emit_declarations_contains_all_functions() {
        let ir = emit_runtime_declarations();
        assert!(ir.contains("@axon_alloc"));
        assert!(ir.contains("@axon_dealloc"));
        assert!(ir.contains("@axon_tensor_alloc"));
        assert!(ir.contains("@axon_tensor_free"));
        assert!(ir.contains("@axon_tensor_shape_check"));
        assert!(ir.contains("@axon_device_transfer"));
        assert!(ir.contains("@axon_panic"));
        assert!(ir.contains("@axon_print_i64"));
        assert!(ir.contains("@axon_print_f64"));
        assert!(ir.contains("@axon_print_str"));
        assert!(ir.contains("@axon_print_bool"));
        assert!(ir.contains("@axon_print_newline"));
        assert!(ir.contains("@axon_print_i32"));
        assert!(ir.contains("@axon_print_f32"));
        assert!(ir.contains("@axon_print_char"));
        assert!(ir.contains("@axon_rc_inc"));
        assert!(ir.contains("@axon_rc_dec"));
        assert!(ir.contains("@axon_rc_count"));
    }

    #[test]
    fn emit_declarations_contains_c_stdlib() {
        let ir = emit_runtime_declarations();
        assert!(ir.contains("@printf"));
        assert!(ir.contains("@puts"));
        assert!(ir.contains("@exit"));
    }

    #[test]
    fn emit_declarations_panic_noreturn() {
        let ir = emit_runtime_declarations();
        assert!(ir.contains("@axon_panic(i8*, i8*, i32) noreturn"));
    }

    #[test]
    fn generate_c_source_non_empty() {
        let src = generate_runtime_c_source();
        assert!(!src.is_empty());
    }

    #[test]
    fn generate_c_source_contains_functions() {
        let src = generate_runtime_c_source();
        assert!(src.contains("void* axon_alloc("));
        assert!(src.contains("void axon_dealloc("));
        assert!(src.contains("void axon_panic("));
        assert!(src.contains("void axon_print_i64("));
        assert!(src.contains("void axon_print_f64("));
        assert!(src.contains("void axon_print_str("));
        assert!(src.contains("void axon_print_bool("));
        assert!(src.contains("void axon_print_newline("));
        assert!(src.contains("void axon_print_i32("));
        assert!(src.contains("void axon_print_f32("));
        assert!(src.contains("void axon_print_char("));
        assert!(src.contains("void axon_rc_inc("));
        assert!(src.contains("void axon_rc_dec("));
        assert!(src.contains("int64_t axon_rc_count("));
    }

    #[test]
    fn generate_c_source_contains_dtype_size() {
        let src = generate_runtime_c_source();
        // Verify axon_dtype_size function exists with proper dtype handling
        assert!(src.contains("size_t axon_dtype_size(uint8_t dtype)"), "missing axon_dtype_size function");
        assert!(src.contains("sizeof(float)"), "missing f32 dtype size");
        assert!(src.contains("sizeof(double)"), "missing f64 dtype size");
        assert!(src.contains("sizeof(int32_t)"), "missing i32 dtype size");
        assert!(src.contains("sizeof(int64_t)"), "missing i64 dtype size");
        assert!(src.contains("sizeof(uint8_t)"), "missing bool/u8 dtype size");
        // Verify tensor_alloc uses axon_dtype_size
        assert!(src.contains("axon_dtype_size"), "tensor_alloc should use axon_dtype_size");
    }

    #[test]
    fn generate_c_source_panic_uses_location() {
        let src = generate_runtime_c_source();
        // Verify axon_panic uses file and line parameters (not hardcoded)
        assert!(src.contains("void axon_panic(const char* msg, const char* file, int32_t line)"));
        assert!(src.contains("file, line, msg"), "axon_panic should use file and line parameters");
    }
}
