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
        description: "Print bool to stdout: (value: i8) -> void",
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
    // Tensor operations — runtime stubs for element-wise and shape ops
    RuntimeFunction {
        name: "tensor_matmul",
        llvm_name: "axon_tensor_matmul",
        description: "Tensor matrix multiply: (a: *u8, b: *u8) -> *u8",
    },
    RuntimeFunction {
        name: "tensor_add",
        llvm_name: "axon_tensor_add",
        description: "Tensor element-wise add: (a: *u8, b: *u8) -> *u8",
    },
    RuntimeFunction {
        name: "tensor_sub",
        llvm_name: "axon_tensor_sub",
        description: "Tensor element-wise sub: (a: *u8, b: *u8) -> *u8",
    },
    RuntimeFunction {
        name: "tensor_mul",
        llvm_name: "axon_tensor_mul",
        description: "Tensor element-wise mul: (a: *u8, b: *u8) -> *u8",
    },
    RuntimeFunction {
        name: "tensor_div",
        llvm_name: "axon_tensor_div",
        description: "Tensor element-wise div: (a: *u8, b: *u8) -> *u8",
    },
    RuntimeFunction {
        name: "tensor_reshape",
        llvm_name: "axon_tensor_reshape",
        description: "Tensor reshape: (src: *u8, new_shape: *i64, new_ndim: i32) -> *u8",
    },
    RuntimeFunction {
        name: "tensor_transpose",
        llvm_name: "axon_tensor_transpose",
        description: "Tensor transpose: (src: *u8) -> *u8",
    },
    RuntimeFunction {
        name: "tensor_broadcast",
        llvm_name: "axon_tensor_broadcast",
        description: "Tensor broadcast: (src: *u8) -> *u8",
    },
    RuntimeFunction {
        name: "tensor_relu",
        llvm_name: "axon_tensor_relu",
        description: "Tensor relu: (src: *u8) -> *u8",
    },
    RuntimeFunction {
        name: "tensor_sum",
        llvm_name: "axon_tensor_sum",
        description: "Tensor sum: (src: *u8) -> *u8",
    },
    RuntimeFunction {
        name: "tensor_mean",
        llvm_name: "axon_tensor_mean",
        description: "Tensor mean: (src: *u8) -> *u8",
    },
    RuntimeFunction {
        name: "tensor_zeros",
        llvm_name: "axon_tensor_zeros",
        description: "Tensor zeros: (shape: *i64, ndim: i32, dtype: i8) -> *u8",
    },
    RuntimeFunction {
        name: "tensor_ones",
        llvm_name: "axon_tensor_ones",
        description: "Tensor ones: (shape: *i64, ndim: i32, dtype: i8) -> *u8",
    },
    RuntimeFunction {
        name: "tensor_rand",
        llvm_name: "axon_tensor_rand",
        description: "Tensor rand: (shape: *i64, ndim: i32, dtype: i8) -> *u8",
    },
    RuntimeFunction {
        name: "tensor_retain",
        llvm_name: "axon_tensor_retain",
        description: "Tensor retain: (tensor: *u8) -> void",
    },
    RuntimeFunction {
        name: "tensor_release",
        llvm_name: "axon_tensor_release",
        description: "Tensor release: (tensor: *u8) -> void",
    },
    RuntimeFunction {
        name: "tensor_print",
        llvm_name: "axon_tensor_print",
        description: "Print tensor summary: (tensor: *u8) -> void",
    },
    RuntimeFunction {
        name: "tensor_item",
        llvm_name: "axon_tensor_item",
        description: "Get scalar from tensor: (tensor: *u8) -> f64",
    },
    RuntimeFunction {
        name: "tensor_grad",
        llvm_name: "axon_tensor_grad",
        description: "Get gradient tensor: (tensor: *u8) -> *u8",
    },
    RuntimeFunction {
        name: "sgd_step1",
        llvm_name: "axon_sgd_step1",
        description: "Single-tensor SGD: (weight: *u8, lr: f64) -> void",
    },
    RuntimeFunction {
        name: "tensor_set_requires_grad",
        llvm_name: "axon_tensor_set_requires_grad",
        description: "Set requires_grad flag: (tensor: *u8, flag: i32) -> void",
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
    ir.push_str("declare void @axon_print_bool(i8)\n");
    ir.push_str("declare void @axon_print_newline()\n");
    ir.push_str("declare void @axon_print_i32(i32)\n");
    ir.push_str("declare void @axon_print_f32(float)\n");
    ir.push_str("declare void @axon_print_char(i32)\n");

    // Reference counting
    ir.push_str("\n; Reference counting runtime\n");
    ir.push_str("declare void @axon_rc_inc(i8*)\n");
    ir.push_str("declare void @axon_rc_dec(i8*)\n");
    ir.push_str("declare i64 @axon_rc_count(i8*)\n");

    // Tensor operations runtime
    ir.push_str("\n; Tensor operations runtime\n");
    ir.push_str("declare i8* @axon_tensor_matmul(i8*, i8*)\n");
    ir.push_str("declare i8* @axon_tensor_add(i8*, i8*)\n");
    ir.push_str("declare i8* @axon_tensor_sub(i8*, i8*)\n");
    ir.push_str("declare i8* @axon_tensor_mul(i8*, i8*)\n");
    ir.push_str("declare i8* @axon_tensor_div(i8*, i8*)\n");
    ir.push_str("declare i8* @axon_tensor_relu(i8*)\n");
    ir.push_str("declare i8* @axon_tensor_sum(i8*)\n");
    ir.push_str("declare i8* @axon_tensor_mean(i8*)\n");
    ir.push_str("declare i8* @axon_tensor_reshape(i8*, i64*, i32)\n");
    ir.push_str("declare i8* @axon_tensor_transpose(i8*)\n");
    ir.push_str("declare i8* @axon_tensor_broadcast(i8*)\n");
    ir.push_str("declare i8* @axon_tensor_zeros(i64*, i32, i8)\n");
    ir.push_str("declare i8* @axon_tensor_ones(i64*, i32, i8)\n");
    ir.push_str("declare i8* @axon_tensor_rand(i64*, i32, i8)\n");
    ir.push_str("declare void @axon_tensor_retain(i8*)\n");
    ir.push_str("declare void @axon_tensor_release(i8*)\n");
    ir.push_str("declare void @axon_tensor_print(i8*)\n");
    ir.push_str("declare double @axon_tensor_item(i8*)\n");
    ir.push_str("declare i8* @axon_tensor_grad(i8*)\n");

    // Autograd runtime
    ir.push_str("\n; Autograd runtime\n");
    ir.push_str("declare void @axon_autograd_enable()\n");
    ir.push_str("declare void @axon_autograd_disable()\n");
    ir.push_str("declare i8* @axon_ag_add(i8*, i8*)\n");
    ir.push_str("declare i8* @axon_ag_sub(i8*, i8*)\n");
    ir.push_str("declare i8* @axon_ag_mul(i8*, i8*)\n");
    ir.push_str("declare i8* @axon_ag_div(i8*, i8*)\n");
    ir.push_str("declare i8* @axon_ag_matmul(i8*, i8*)\n");
    ir.push_str("declare i8* @axon_ag_relu(i8*)\n");
    ir.push_str("declare i8* @axon_ag_sum(i8*)\n");
    ir.push_str("declare i8* @axon_ag_mean(i8*)\n");
    ir.push_str("declare void @axon_backward(i8*)\n");
    ir.push_str("declare void @axon_zero_grad(i8*)\n");
    ir.push_str("declare void @axon_tape_clear()\n");

    // Training runtime
    ir.push_str("\n; Training runtime\n");
    ir.push_str("declare void @axon_sgd_step(i8**, i32, double)\n");
    ir.push_str("declare void @axon_sgd_step1(i8*, double)\n");
    ir.push_str("declare void @axon_tensor_set_requires_grad(i8*, i32)\n");
    ir.push_str("declare i8* @axon_cross_entropy_loss(i8*, i8*)\n");
    ir.push_str("declare i8* @axon_mse_loss(i8*, i8*)\n");

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
#include <math.h>

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

// ── Tensor Operations (real implementations) ────────────────
// AxonTensor: a reference-counted, strided, N-dimensional tensor.

typedef struct {
    void* data;            // raw data buffer (owned if refcount > 0)
    int64_t* shape;        // shape array [d0, d1, ..., dn-1]
    int64_t* strides;      // strides array in elements
    int32_t ndim;          // number of dimensions
    int64_t numel;         // total number of elements
    uint8_t dtype;         // 0=f32, 1=f64, 2=i32, 3=i64, 4=bool
    int32_t refcount;      // reference count
    int8_t requires_grad;  // whether to track gradients
    void* grad;            // gradient tensor (AxonTensor* or NULL)
    void* grad_fn;         // backward function pointer (NULL for leaf)
} AxonTensor;

static int64_t axon_tensor_compute_numel(int64_t* shape, int32_t ndim) {
    int64_t n = 1;
    for (int32_t i = 0; i < ndim; i++) n *= shape[i];
    return n;
}

static void axon_tensor_compute_strides(int64_t* strides, int64_t* shape, int32_t ndim) {
    if (ndim == 0) return;
    strides[ndim - 1] = 1;
    for (int32_t i = ndim - 2; i >= 0; i--) {
        strides[i] = strides[i + 1] * shape[i + 1];
    }
}

static AxonTensor* axon_tensor_new(int64_t* shape, int32_t ndim, uint8_t dtype) {
    AxonTensor* t = (AxonTensor*)malloc(sizeof(AxonTensor));
    t->ndim = ndim;
    t->dtype = dtype;
    t->numel = axon_tensor_compute_numel(shape, ndim);
    t->shape = (int64_t*)malloc(ndim * sizeof(int64_t));
    t->strides = (int64_t*)malloc(ndim * sizeof(int64_t));
    memcpy(t->shape, shape, ndim * sizeof(int64_t));
    axon_tensor_compute_strides(t->strides, shape, ndim);
    size_t elem_size = axon_dtype_size(dtype);
    t->data = calloc(t->numel, elem_size);
    t->refcount = 1;
    t->requires_grad = 0;
    t->grad = NULL;
    t->grad_fn = NULL;
    return t;
}

void axon_tensor_retain(void* ptr) {
    if (!ptr) return;
    AxonTensor* t = (AxonTensor*)ptr;
    t->refcount++;
}

void axon_tensor_release(void* ptr) {
    if (!ptr) return;
    AxonTensor* t = (AxonTensor*)ptr;
    if (--t->refcount <= 0) {
        free(t->data);
        free(t->shape);
        free(t->strides);
        if (t->grad) axon_tensor_release(t->grad);
        free(t);
    }
}

void* axon_tensor_zeros(int64_t* shape, int32_t ndim, uint8_t dtype) {
    return axon_tensor_new(shape, ndim, dtype);
}

void* axon_tensor_ones(int64_t* shape, int32_t ndim, uint8_t dtype) {
    AxonTensor* t = axon_tensor_new(shape, ndim, dtype);
    if (dtype == 0) {
        float* d = (float*)t->data;
        for (int64_t i = 0; i < t->numel; i++) d[i] = 1.0f;
    } else if (dtype == 1) {
        double* d = (double*)t->data;
        for (int64_t i = 0; i < t->numel; i++) d[i] = 1.0;
    }
    return t;
}

void* axon_tensor_rand(int64_t* shape, int32_t ndim, uint8_t dtype) {
    AxonTensor* t = axon_tensor_new(shape, ndim, dtype);
    if (dtype == 0) {
        float* d = (float*)t->data;
        for (int64_t i = 0; i < t->numel; i++) d[i] = (float)rand() / (float)RAND_MAX;
    } else if (dtype == 1) {
        double* d = (double*)t->data;
        for (int64_t i = 0; i < t->numel; i++) d[i] = (double)rand() / (double)RAND_MAX;
    }
    return t;
}

void* axon_tensor_matmul(void* va, void* vb) {
    AxonTensor* a = (AxonTensor*)va;
    AxonTensor* b = (AxonTensor*)vb;
    if (a->ndim < 2 || b->ndim < 2) {
        fprintf(stderr, "axon panic: matmul requires at least 2D tensors\n");
        exit(1);
    }
    int64_t M = a->shape[a->ndim - 2];
    int64_t K = a->shape[a->ndim - 1];
    int64_t K2 = b->shape[b->ndim - 2];
    int64_t N = b->shape[b->ndim - 1];
    if (K != K2) {
        fprintf(stderr, "axon panic: matmul inner dims don't match: %lld vs %lld\n",
                (long long)K, (long long)K2);
        exit(1);
    }
    int64_t out_shape[2] = {M, N};
    AxonTensor* c = axon_tensor_new(out_shape, 2, a->dtype);
    if (a->dtype == 0) {
        float* da = (float*)a->data;
        float* db = (float*)b->data;
        float* dc = (float*)c->data;
        for (int64_t i = 0; i < M; i++) {
            for (int64_t j = 0; j < N; j++) {
                float sum = 0.0f;
                for (int64_t k = 0; k < K; k++) {
                    sum += da[i * K + k] * db[k * N + j];
                }
                dc[i * N + j] = sum;
            }
        }
    } else if (a->dtype == 1) {
        double* da = (double*)a->data;
        double* db = (double*)b->data;
        double* dc = (double*)c->data;
        for (int64_t i = 0; i < M; i++) {
            for (int64_t j = 0; j < N; j++) {
                double sum = 0.0;
                for (int64_t k = 0; k < K; k++) {
                    sum += da[i * K + k] * db[k * N + j];
                }
                dc[i * N + j] = sum;
            }
        }
    }
    return c;
}

void* axon_tensor_add(void* va, void* vb) {
    AxonTensor* a = (AxonTensor*)va;
    AxonTensor* b = (AxonTensor*)vb;
    AxonTensor* c = axon_tensor_new(a->shape, a->ndim, a->dtype);
    int64_t n = a->numel < b->numel ? a->numel : b->numel;
    if (a->dtype == 0) {
        float *da=(float*)a->data, *db=(float*)b->data, *dc=(float*)c->data;
        for (int64_t i = 0; i < n; i++) dc[i] = da[i] + db[i % b->numel];
    } else if (a->dtype == 1) {
        double *da=(double*)a->data, *db=(double*)b->data, *dc=(double*)c->data;
        for (int64_t i = 0; i < n; i++) dc[i] = da[i] + db[i % b->numel];
    }
    return c;
}

void* axon_tensor_sub(void* va, void* vb) {
    AxonTensor* a = (AxonTensor*)va;
    AxonTensor* b = (AxonTensor*)vb;
    AxonTensor* c = axon_tensor_new(a->shape, a->ndim, a->dtype);
    int64_t n = a->numel;
    if (a->dtype == 0) {
        float *da=(float*)a->data, *db=(float*)b->data, *dc=(float*)c->data;
        for (int64_t i = 0; i < n; i++) dc[i] = da[i] - db[i % b->numel];
    } else if (a->dtype == 1) {
        double *da=(double*)a->data, *db=(double*)b->data, *dc=(double*)c->data;
        for (int64_t i = 0; i < n; i++) dc[i] = da[i] - db[i % b->numel];
    }
    return c;
}

void* axon_tensor_mul(void* va, void* vb) {
    AxonTensor* a = (AxonTensor*)va;
    AxonTensor* b = (AxonTensor*)vb;
    AxonTensor* c = axon_tensor_new(a->shape, a->ndim, a->dtype);
    int64_t n = a->numel;
    if (a->dtype == 0) {
        float *da=(float*)a->data, *db=(float*)b->data, *dc=(float*)c->data;
        for (int64_t i = 0; i < n; i++) dc[i] = da[i] * db[i % b->numel];
    } else if (a->dtype == 1) {
        double *da=(double*)a->data, *db=(double*)b->data, *dc=(double*)c->data;
        for (int64_t i = 0; i < n; i++) dc[i] = da[i] * db[i % b->numel];
    }
    return c;
}

void* axon_tensor_div(void* va, void* vb) {
    AxonTensor* a = (AxonTensor*)va;
    AxonTensor* b = (AxonTensor*)vb;
    AxonTensor* c = axon_tensor_new(a->shape, a->ndim, a->dtype);
    int64_t n = a->numel;
    if (a->dtype == 0) {
        float *da=(float*)a->data, *db=(float*)b->data, *dc=(float*)c->data;
        for (int64_t i = 0; i < n; i++) dc[i] = da[i] / db[i % b->numel];
    } else if (a->dtype == 1) {
        double *da=(double*)a->data, *db=(double*)b->data, *dc=(double*)c->data;
        for (int64_t i = 0; i < n; i++) dc[i] = da[i] / db[i % b->numel];
    }
    return c;
}

void* axon_tensor_relu(void* va) {
    AxonTensor* a = (AxonTensor*)va;
    AxonTensor* c = axon_tensor_new(a->shape, a->ndim, a->dtype);
    if (a->dtype == 0) {
        float *da=(float*)a->data, *dc=(float*)c->data;
        for (int64_t i = 0; i < a->numel; i++) dc[i] = da[i] > 0 ? da[i] : 0.0f;
    } else if (a->dtype == 1) {
        double *da=(double*)a->data, *dc=(double*)c->data;
        for (int64_t i = 0; i < a->numel; i++) dc[i] = da[i] > 0 ? da[i] : 0.0;
    }
    return c;
}

void* axon_tensor_sum(void* va) {
    AxonTensor* a = (AxonTensor*)va;
    int64_t scalar_shape[1] = {1};
    AxonTensor* c = axon_tensor_new(scalar_shape, 1, a->dtype);
    if (a->dtype == 0) {
        float *da=(float*)a->data;
        float sum = 0.0f;
        for (int64_t i = 0; i < a->numel; i++) sum += da[i];
        ((float*)c->data)[0] = sum;
    } else if (a->dtype == 1) {
        double *da=(double*)a->data;
        double sum = 0.0;
        for (int64_t i = 0; i < a->numel; i++) sum += da[i];
        ((double*)c->data)[0] = sum;
    }
    return c;
}

void* axon_tensor_mean(void* va) {
    AxonTensor* a = (AxonTensor*)va;
    int64_t scalar_shape[1] = {1};
    AxonTensor* c = axon_tensor_new(scalar_shape, 1, a->dtype);
    if (a->dtype == 0) {
        float *da=(float*)a->data;
        float sum = 0.0f;
        for (int64_t i = 0; i < a->numel; i++) sum += da[i];
        ((float*)c->data)[0] = sum / (float)a->numel;
    } else if (a->dtype == 1) {
        double *da=(double*)a->data;
        double sum = 0.0;
        for (int64_t i = 0; i < a->numel; i++) sum += da[i];
        ((double*)c->data)[0] = sum / (double)a->numel;
    }
    return c;
}

void* axon_tensor_reshape(void* va, int64_t* new_shape, int32_t new_ndim) {
    AxonTensor* a = (AxonTensor*)va;
    int64_t new_numel = axon_tensor_compute_numel(new_shape, new_ndim);
    if (new_numel != a->numel) {
        fprintf(stderr, "axon panic: reshape: cannot reshape %lld elements to %lld elements\n",
                (long long)a->numel, (long long)new_numel);
        exit(1);
    }
    AxonTensor* c = (AxonTensor*)malloc(sizeof(AxonTensor));
    c->data = a->data;
    c->ndim = new_ndim;
    c->numel = new_numel;
    c->dtype = a->dtype;
    c->shape = (int64_t*)malloc(new_ndim * sizeof(int64_t));
    c->strides = (int64_t*)malloc(new_ndim * sizeof(int64_t));
    memcpy(c->shape, new_shape, new_ndim * sizeof(int64_t));
    axon_tensor_compute_strides(c->strides, new_shape, new_ndim);
    c->refcount = 1;
    c->requires_grad = a->requires_grad;
    c->grad = NULL;
    c->grad_fn = NULL;
    axon_tensor_retain(va);  // shared data, increment refcount on source
    return c;
}

void* axon_tensor_transpose(void* va) {
    AxonTensor* a = (AxonTensor*)va;
    if (a->ndim < 2) {
        fprintf(stderr, "axon panic: transpose requires at least 2D tensor\n");
        exit(1);
    }
    int64_t* new_shape = (int64_t*)malloc(a->ndim * sizeof(int64_t));
    memcpy(new_shape, a->shape, a->ndim * sizeof(int64_t));
    // Swap last two dims
    int64_t tmp = new_shape[a->ndim - 1];
    new_shape[a->ndim - 1] = new_shape[a->ndim - 2];
    new_shape[a->ndim - 2] = tmp;
    AxonTensor* c = axon_tensor_new(new_shape, a->ndim, a->dtype);
    // Copy with transposed indices
    int64_t rows = a->shape[a->ndim - 2];
    int64_t cols = a->shape[a->ndim - 1];
    if (a->dtype == 0) {
        float *da=(float*)a->data, *dc=(float*)c->data;
        for (int64_t i = 0; i < rows; i++)
            for (int64_t j = 0; j < cols; j++)
                dc[j * rows + i] = da[i * cols + j];
    } else if (a->dtype == 1) {
        double *da=(double*)a->data, *dc=(double*)c->data;
        for (int64_t i = 0; i < rows; i++)
            for (int64_t j = 0; j < cols; j++)
                dc[j * rows + i] = da[i * cols + j];
    }
    free(new_shape);
    return c;
}

void* axon_tensor_broadcast(void* src) {
    return src;  // basic broadcast is a no-op view; real broadcast handled in element-wise ops
}

// Print tensor summary
void axon_tensor_print(void* va) {
    AxonTensor* t = (AxonTensor*)va;
    printf("Tensor(shape=[");
    for (int32_t i = 0; i < t->ndim; i++) {
        if (i > 0) printf(", ");
        printf("%lld", (long long)t->shape[i]);
    }
    printf("], dtype=%s, numel=%lld)\n",
        t->dtype == 0 ? "f32" : t->dtype == 1 ? "f64" : t->dtype == 2 ? "i32" : "i64",
        (long long)t->numel);
    fflush(stdout);
}

// Get scalar value from a 1-element tensor
double axon_tensor_item(void* va) {
    AxonTensor* t = (AxonTensor*)va;
    if (t->dtype == 0) return (double)((float*)t->data)[0];
    if (t->dtype == 1) return ((double*)t->data)[0];
    if (t->dtype == 2) return (double)((int32_t*)t->data)[0];
    if (t->dtype == 3) return (double)((int64_t*)t->data)[0];
    return 0.0;
}

// ══════════════════════════════════════════════════════════════
// AUTOGRAD ENGINE — Tape-based reverse-mode automatic differentiation
// ══════════════════════════════════════════════════════════════

// Operation types for the computation graph
typedef enum {
    OP_ADD = 0, OP_SUB = 1, OP_MUL = 2, OP_DIV = 3,
    OP_MATMUL = 4, OP_RELU = 5, OP_SIGMOID = 6, OP_TANH = 7,
    OP_SUM = 8, OP_MEAN = 9, OP_RESHAPE = 10, OP_TRANSPOSE = 11
} GradOp;

typedef struct TapeEntry {
    GradOp op;
    void* output;     // AxonTensor* — result of forward op
    void* input_a;    // AxonTensor* — first input
    void* input_b;    // AxonTensor* — second input (NULL for unary)
} TapeEntry;

// Global tape (thread-local in a full implementation)
static TapeEntry* g_tape = NULL;
static int32_t g_tape_len = 0;
static int32_t g_tape_cap = 0;
static int32_t g_tape_enabled = 1;

void axon_autograd_enable(void)  { g_tape_enabled = 1; }
void axon_autograd_disable(void) { g_tape_enabled = 0; }
int32_t axon_autograd_is_enabled(void) { return g_tape_enabled; }

static void tape_record(GradOp op, void* out, void* a, void* b) {
    if (!g_tape_enabled) return;
    AxonTensor* ta = (AxonTensor*)a;
    AxonTensor* tb = (AxonTensor*)b;
    // Only record if at least one input requires grad
    if (!ta->requires_grad && (!tb || !((AxonTensor*)tb)->requires_grad)) return;

    if (g_tape_len >= g_tape_cap) {
        g_tape_cap = g_tape_cap == 0 ? 64 : g_tape_cap * 2;
        g_tape = (TapeEntry*)realloc(g_tape, g_tape_cap * sizeof(TapeEntry));
    }
    TapeEntry* e = &g_tape[g_tape_len++];
    e->op = op;
    e->output = out;
    e->input_a = a;
    e->input_b = b;
    ((AxonTensor*)out)->requires_grad = 1;
}

// Autograd-aware forward ops (record on tape, then compute)
void* axon_ag_add(void* a, void* b) {
    void* c = axon_tensor_add(a, b);
    tape_record(OP_ADD, c, a, b);
    return c;
}
void* axon_ag_sub(void* a, void* b) {
    void* c = axon_tensor_sub(a, b);
    tape_record(OP_SUB, c, a, b);
    return c;
}
void* axon_ag_mul(void* a, void* b) {
    void* c = axon_tensor_mul(a, b);
    tape_record(OP_MUL, c, a, b);
    return c;
}
void* axon_ag_div(void* a, void* b) {
    void* c = axon_tensor_div(a, b);
    tape_record(OP_DIV, c, a, b);
    return c;
}
void* axon_ag_matmul(void* a, void* b) {
    void* c = axon_tensor_matmul(a, b);
    tape_record(OP_MATMUL, c, a, b);
    return c;
}
void* axon_ag_relu(void* a) {
    void* c = axon_tensor_relu(a);
    tape_record(OP_RELU, c, a, NULL);
    return c;
}
void* axon_ag_sum(void* a) {
    void* c = axon_tensor_sum(a);
    tape_record(OP_SUM, c, a, NULL);
    return c;
}
void* axon_ag_mean(void* a) {
    void* c = axon_tensor_mean(a);
    tape_record(OP_MEAN, c, a, NULL);
    return c;
}

// Accumulate gradient: t->grad += grad
static void accum_grad(AxonTensor* t, AxonTensor* grad) {
    if (!t->grad) {
        t->grad = axon_tensor_zeros(t->shape, t->ndim, t->dtype);
    }
    AxonTensor* g = (AxonTensor*)t->grad;
    if (t->dtype == 0) {
        float *dg=(float*)g->data, *dv=(float*)grad->data;
        int64_t n = g->numel < grad->numel ? g->numel : grad->numel;
        for (int64_t i = 0; i < n; i++) dg[i] += dv[i % grad->numel];
    } else if (t->dtype == 1) {
        double *dg=(double*)g->data, *dv=(double*)grad->data;
        int64_t n = g->numel < grad->numel ? g->numel : grad->numel;
        for (int64_t i = 0; i < n; i++) dg[i] += dv[i % grad->numel];
    }
}

// Create a tensor of ones with same shape as t
static AxonTensor* ones_like(AxonTensor* t) {
    return (AxonTensor*)axon_tensor_ones(t->shape, t->ndim, t->dtype);
}

// Backward pass: traverse tape in reverse, apply gradient rules
void axon_backward(void* va) {
    AxonTensor* root = (AxonTensor*)va;
    if (root->numel != 1) {
        fprintf(stderr, "axon panic: backward() requires scalar tensor (numel=1), got %lld\n",
                (long long)root->numel);
        exit(1);
    }
    // Seed gradient: dL/dL = 1.0
    root->grad = axon_tensor_ones(root->shape, root->ndim, root->dtype);

    // Reverse tape traversal
    for (int32_t i = g_tape_len - 1; i >= 0; i--) {
        TapeEntry* e = &g_tape[i];
        AxonTensor* out = (AxonTensor*)e->output;
        AxonTensor* a = (AxonTensor*)e->input_a;
        AxonTensor* b = (AxonTensor*)e->input_b;
        AxonTensor* grad_out = (AxonTensor*)out->grad;
        if (!grad_out) continue;

        switch (e->op) {
            case OP_ADD:
                // d/da(a+b) = 1, d/db(a+b) = 1
                if (a->requires_grad) accum_grad(a, grad_out);
                if (b && b->requires_grad) accum_grad(b, grad_out);
                break;
            case OP_SUB:
                // d/da(a-b) = 1, d/db(a-b) = -1
                if (a->requires_grad) accum_grad(a, grad_out);
                if (b && b->requires_grad) {
                    AxonTensor* neg = axon_tensor_new(grad_out->shape, grad_out->ndim, grad_out->dtype);
                    if (grad_out->dtype == 0) {
                        float *ds=(float*)grad_out->data, *dd=(float*)neg->data;
                        for (int64_t j=0; j<grad_out->numel; j++) dd[j] = -ds[j];
                    } else {
                        double *ds=(double*)grad_out->data, *dd=(double*)neg->data;
                        for (int64_t j=0; j<grad_out->numel; j++) dd[j] = -ds[j];
                    }
                    accum_grad(b, neg);
                    axon_tensor_release(neg);
                }
                break;
            case OP_MUL:
                // d/da(a*b) = b, d/db(a*b) = a
                if (a->requires_grad) {
                    AxonTensor* ga = (AxonTensor*)axon_tensor_mul(grad_out, b);
                    accum_grad(a, ga);
                    axon_tensor_release(ga);
                }
                if (b && b->requires_grad) {
                    AxonTensor* gb = (AxonTensor*)axon_tensor_mul(grad_out, a);
                    accum_grad(b, gb);
                    axon_tensor_release(gb);
                }
                break;
            case OP_DIV:
                // d/da(a/b) = 1/b, d/db(a/b) = -a/b^2
                if (a->requires_grad) {
                    AxonTensor* ga = (AxonTensor*)axon_tensor_div(grad_out, b);
                    accum_grad(a, ga);
                    axon_tensor_release(ga);
                }
                if (b && b->requires_grad) {
                    AxonTensor* b2 = (AxonTensor*)axon_tensor_mul(b, b);
                    AxonTensor* neg_a = axon_tensor_new(a->shape, a->ndim, a->dtype);
                    if (a->dtype == 0) { float *s=(float*)a->data,*d=(float*)neg_a->data; for(int64_t j=0;j<a->numel;j++) d[j]=-s[j]; }
                    else { double *s=(double*)a->data,*d=(double*)neg_a->data; for(int64_t j=0;j<a->numel;j++) d[j]=-s[j]; }
                    AxonTensor* gb = (AxonTensor*)axon_tensor_div(axon_tensor_mul(grad_out, neg_a), b2);
                    accum_grad(b, gb);
                    axon_tensor_release(b2);
                    axon_tensor_release(neg_a);
                    axon_tensor_release(gb);
                }
                break;
            case OP_MATMUL:
                // dL/dA = dL/dC @ B^T, dL/dB = A^T @ dL/dC
                if (a->requires_grad) {
                    AxonTensor* bt = (AxonTensor*)axon_tensor_transpose(b);
                    AxonTensor* ga = (AxonTensor*)axon_tensor_matmul(grad_out, bt);
                    accum_grad(a, ga);
                    axon_tensor_release(bt);
                    axon_tensor_release(ga);
                }
                if (b && b->requires_grad) {
                    AxonTensor* at = (AxonTensor*)axon_tensor_transpose(a);
                    AxonTensor* gb = (AxonTensor*)axon_tensor_matmul(at, grad_out);
                    accum_grad(b, gb);
                    axon_tensor_release(at);
                    axon_tensor_release(gb);
                }
                break;
            case OP_RELU:
                // d/da(relu(a)) = 1 if a > 0, else 0
                if (a->requires_grad) {
                    AxonTensor* ga = axon_tensor_new(a->shape, a->ndim, a->dtype);
                    if (a->dtype == 0) {
                        float *da=(float*)a->data, *dg=(float*)grad_out->data, *dd=(float*)ga->data;
                        for(int64_t j=0;j<a->numel;j++) dd[j] = da[j] > 0 ? dg[j] : 0.0f;
                    } else {
                        double *da=(double*)a->data, *dg=(double*)grad_out->data, *dd=(double*)ga->data;
                        for(int64_t j=0;j<a->numel;j++) dd[j] = da[j] > 0 ? dg[j] : 0.0;
                    }
                    accum_grad(a, ga);
                    axon_tensor_release(ga);
                }
                break;
            case OP_SUM:
                // d/da(sum(a)) = ones_like(a) * grad_out
                if (a->requires_grad) {
                    AxonTensor* ga = ones_like(a);
                    // Scale by grad_out scalar
                    if (a->dtype == 0) {
                        float s = ((float*)grad_out->data)[0];
                        float *d = (float*)ga->data;
                        for(int64_t j=0;j<ga->numel;j++) d[j] *= s;
                    } else {
                        double s = ((double*)grad_out->data)[0];
                        double *d = (double*)ga->data;
                        for(int64_t j=0;j<ga->numel;j++) d[j] *= s;
                    }
                    accum_grad(a, ga);
                    axon_tensor_release(ga);
                }
                break;
            case OP_MEAN:
                // d/da(mean(a)) = ones_like(a) * grad_out / numel
                if (a->requires_grad) {
                    AxonTensor* ga = ones_like(a);
                    if (a->dtype == 0) {
                        float s = ((float*)grad_out->data)[0] / (float)a->numel;
                        float *d = (float*)ga->data;
                        for(int64_t j=0;j<ga->numel;j++) d[j] *= s;
                    } else {
                        double s = ((double*)grad_out->data)[0] / (double)a->numel;
                        double *d = (double*)ga->data;
                        for(int64_t j=0;j<ga->numel;j++) d[j] *= s;
                    }
                    accum_grad(a, ga);
                    axon_tensor_release(ga);
                }
                break;
            case OP_RESHAPE:
            case OP_TRANSPOSE:
                // Just pass gradient through (reshape grad to input shape)
                if (a->requires_grad) accum_grad(a, grad_out);
                break;
        }
    }
}

// Get the gradient tensor (or NULL if none)
void* axon_tensor_grad(void* va) {
    AxonTensor* t = (AxonTensor*)va;
    return t->grad;
}

// Zero all gradients
void axon_zero_grad(void* va) {
    AxonTensor* t = (AxonTensor*)va;
    if (t->grad) {
        AxonTensor* g = (AxonTensor*)t->grad;
        memset(g->data, 0, g->numel * axon_dtype_size(g->dtype));
    }
}

// Clear the tape (call after backward + step)
void axon_tape_clear(void) {
    g_tape_len = 0;
}

// ══════════════════════════════════════════════════════════════
// TRAINING INFRASTRUCTURE — Optimizers, Loss, Data utilities
// ══════════════════════════════════════════════════════════════

// SGD optimizer step: w -= lr * w.grad
void axon_sgd_step(void** params, int32_t n_params, double lr) {
    for (int32_t i = 0; i < n_params; i++) {
        AxonTensor* w = (AxonTensor*)params[i];
        if (!w->grad) continue;
        AxonTensor* g = (AxonTensor*)w->grad;
        if (w->dtype == 0) {
            float *dw=(float*)w->data, *dg=(float*)g->data;
            float flr = (float)lr;
            for(int64_t j=0;j<w->numel;j++) dw[j] -= flr * dg[j];
        } else {
            double *dw=(double*)w->data, *dg=(double*)g->data;
            for(int64_t j=0;j<w->numel;j++) dw[j] -= lr * dg[j];
        }
    }
}

// Single-tensor SGD step for MVP: w -= lr * w.grad
void axon_sgd_step1(void* vw, double lr) {
    AxonTensor* w = (AxonTensor*)vw;
    if (!w->grad) return;
    AxonTensor* g = (AxonTensor*)w->grad;
    if (w->dtype == 0) {
        float *dw=(float*)w->data, *dg=(float*)g->data;
        float flr = (float)lr;
        for(int64_t j=0;j<w->numel;j++) dw[j] -= flr * dg[j];
    } else {
        double *dw=(double*)w->data, *dg=(double*)g->data;
        for(int64_t j=0;j<w->numel;j++) dw[j] -= lr * dg[j];
    }
}

// Set requires_grad flag on a tensor
void axon_tensor_set_requires_grad(void* va, int32_t flag) {
    AxonTensor* t = (AxonTensor*)va;
    t->requires_grad = (int8_t)flag;
}

// Cross-entropy loss: -sum(target * log(softmax(pred)))
void* axon_cross_entropy_loss(void* vpred, void* vtarget) {
    AxonTensor* pred = (AxonTensor*)vpred;
    AxonTensor* target = (AxonTensor*)vtarget;
    int64_t scalar_shape[1] = {1};
    AxonTensor* loss = axon_tensor_new(scalar_shape, 1, pred->dtype);

    if (pred->dtype == 0) {
        float *dp=(float*)pred->data, *dt=(float*)target->data;
        // Simple cross-entropy for 1D: -sum(t * log(softmax(p)))
        // First find max for numerical stability
        float maxv = dp[0];
        for(int64_t j=1;j<pred->numel;j++) if(dp[j]>maxv) maxv=dp[j];
        // Compute log-softmax
        float sum_exp = 0.0f;
        for(int64_t j=0;j<pred->numel;j++) sum_exp += expf(dp[j]-maxv);
        float log_sum_exp = logf(sum_exp) + maxv;
        float ce = 0.0f;
        for(int64_t j=0;j<pred->numel;j++) ce -= dt[j] * (dp[j] - log_sum_exp);
        ((float*)loss->data)[0] = ce;
    } else {
        double *dp=(double*)pred->data, *dt=(double*)target->data;
        double maxv = dp[0];
        for(int64_t j=1;j<pred->numel;j++) if(dp[j]>maxv) maxv=dp[j];
        double sum_exp = 0.0;
        for(int64_t j=0;j<pred->numel;j++) sum_exp += exp(dp[j]-maxv);
        double log_sum_exp = log(sum_exp) + maxv;
        double ce = 0.0;
        for(int64_t j=0;j<pred->numel;j++) ce -= dt[j] * (dp[j] - log_sum_exp);
        ((double*)loss->data)[0] = ce;
    }
    return loss;
}

// MSE loss: mean((pred - target)^2)
void* axon_mse_loss(void* vpred, void* vtarget) {
    AxonTensor* pred = (AxonTensor*)vpred;
    AxonTensor* target = (AxonTensor*)vtarget;
    int64_t scalar_shape[1] = {1};
    AxonTensor* loss = axon_tensor_new(scalar_shape, 1, pred->dtype);
    if (pred->dtype == 0) {
        float *dp=(float*)pred->data, *dt=(float*)target->data;
        float sum = 0.0f;
        for(int64_t j=0;j<pred->numel;j++) { float d=dp[j]-dt[j]; sum+=d*d; }
        ((float*)loss->data)[0] = sum / (float)pred->numel;
    } else {
        double *dp=(double*)pred->data, *dt=(double*)target->data;
        double sum = 0.0;
        for(int64_t j=0;j<pred->numel;j++) { double d=dp[j]-dt[j]; sum+=d*d; }
        ((double*)loss->data)[0] = sum / (double)pred->numel;
    }
    return loss;
}
"#.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runtime_functions_count() {
        assert_eq!(RUNTIME_FUNCTIONS.len(), 39);
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
        // Tensor operation stubs
        assert!(ir.contains("@axon_tensor_matmul"));
        assert!(ir.contains("@axon_tensor_add"));
        assert!(ir.contains("@axon_tensor_sub"));
        assert!(ir.contains("@axon_tensor_mul"));
        assert!(ir.contains("@axon_tensor_div"));
        assert!(ir.contains("@axon_tensor_relu"));
        assert!(ir.contains("@axon_tensor_sum"));
        assert!(ir.contains("@axon_tensor_mean"));
        assert!(ir.contains("@axon_tensor_reshape"));
        assert!(ir.contains("@axon_tensor_transpose"));
        assert!(ir.contains("@axon_tensor_broadcast"));
        assert!(ir.contains("@axon_tensor_zeros"));
        assert!(ir.contains("@axon_tensor_ones"));
        assert!(ir.contains("@axon_tensor_rand"));
        assert!(ir.contains("@axon_tensor_retain"));
        assert!(ir.contains("@axon_tensor_release"));
        assert!(ir.contains("@axon_tensor_print"));
        assert!(ir.contains("@axon_tensor_item"));
        assert!(ir.contains("@axon_tensor_grad"));
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
        // Tensor operation real implementations
        assert!(src.contains("void* axon_tensor_matmul("));
        assert!(src.contains("void* axon_tensor_add("));
        assert!(src.contains("void* axon_tensor_sub("));
        assert!(src.contains("void* axon_tensor_mul("));
        assert!(src.contains("void* axon_tensor_div("));
        assert!(src.contains("void* axon_tensor_reshape("));
        assert!(src.contains("void* axon_tensor_transpose("));
        assert!(src.contains("void* axon_tensor_broadcast("));
        assert!(src.contains("void* axon_tensor_relu("));
        assert!(src.contains("void* axon_tensor_sum("));
        assert!(src.contains("void* axon_tensor_mean("));
        assert!(src.contains("void* axon_tensor_zeros("));
        assert!(src.contains("void* axon_tensor_ones("));
        assert!(src.contains("void* axon_tensor_rand("));
        assert!(src.contains("void axon_tensor_retain("));
        assert!(src.contains("void axon_tensor_release("));
        assert!(src.contains("void axon_tensor_print("));
        assert!(src.contains("double axon_tensor_item("));
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
