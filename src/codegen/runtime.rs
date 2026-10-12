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
    // Fused tensor operations (E7)
    RuntimeFunction {
        name: "fused_linear_relu",
        llvm_name: "axon_fused_linear_relu",
        description: "Fused matmul+add+relu: (a: *u8, b: *u8, bias: *u8) -> *u8",
    },
    RuntimeFunction {
        name: "fused_mse",
        llvm_name: "axon_fused_mse",
        description: "Fused sub+square+mean (MSE): (pred: *u8, target: *u8) -> *u8",
    },
    RuntimeFunction {
        name: "fused_softmax_ce",
        llvm_name: "axon_fused_softmax_ce",
        description: "Fused softmax+cross_entropy: (logits: *u8, target: *u8) -> *u8",
    },
    RuntimeFunction {
        name: "fused_elementwise",
        llvm_name: "axon_fused_elementwise",
        description: "Fused element-wise chain: (tensors: **u8, n_tensors: i32, ops: *u8, n_ops: i32) -> *u8",
    },
    // Parallel execution runtime (E8)
    RuntimeFunction {
        name: "parallel_begin",
        llvm_name: "axon_parallel_begin",
        description: "Begin parallel region: (count: i32) -> void",
    },
    RuntimeFunction {
        name: "parallel_end",
        llvm_name: "axon_parallel_end",
        description: "End parallel region (join point): () -> void",
    },
    // Thread pool runtime (GAMMA/E10)
    RuntimeFunction {
        name: "threadpool_init",
        llvm_name: "axon_threadpool_init",
        description: "Initialize global thread pool: (num_threads: i32) -> void",
    },
    RuntimeFunction {
        name: "threadpool_shutdown",
        llvm_name: "axon_threadpool_shutdown",
        description: "Shutdown global thread pool: () -> void",
    },
    RuntimeFunction {
        name: "threadpool_num_threads",
        llvm_name: "axon_threadpool_num_threads",
        description: "Get thread pool size: () -> i32",
    },
    // Memory pool runtime (GAMMA/E11)
    RuntimeFunction {
        name: "mempool_init",
        llvm_name: "axon_mempool_init",
        description: "Initialize memory pool: () -> void",
    },
    RuntimeFunction {
        name: "mempool_alloc",
        llvm_name: "axon_mempool_alloc",
        description: "Allocate from memory pool: (size: i64) -> *u8",
    },
    RuntimeFunction {
        name: "mempool_free",
        llvm_name: "axon_mempool_free",
        description: "Return allocation to memory pool: (ptr: *u8, size: i64) -> void",
    },
    RuntimeFunction {
        name: "mempool_clear",
        llvm_name: "axon_mempool_clear",
        description: "Release all cached memory pool blocks: () -> void",
    },
    RuntimeFunction {
        name: "mempool_stats",
        llvm_name: "axon_mempool_stats",
        description: "Print memory pool statistics: () -> void",
    },
    // GPU dispatch stubs (GAMMA/E12)
    RuntimeFunction {
        name: "gpu_init",
        llvm_name: "axon_gpu_init",
        description: "Initialize GPU context: () -> i32 (0=success, -1=no GPU)",
    },
    RuntimeFunction {
        name: "gpu_shutdown",
        llvm_name: "axon_gpu_shutdown",
        description: "Shutdown GPU context: () -> void",
    },
    RuntimeFunction {
        name: "gpu_device_count",
        llvm_name: "axon_gpu_device_count",
        description: "Get number of GPUs: () -> i32",
    },
    RuntimeFunction {
        name: "gpu_alloc",
        llvm_name: "axon_gpu_alloc",
        description: "Allocate GPU memory: (size: i64) -> *u8",
    },
    RuntimeFunction {
        name: "gpu_free",
        llvm_name: "axon_gpu_free",
        description: "Free GPU memory: (ptr: *u8) -> void",
    },
    RuntimeFunction {
        name: "gpu_tensor_to_device",
        llvm_name: "axon_gpu_tensor_to_device",
        description: "Copy tensor to GPU: (tensor: *u8, device_id: i32) -> *u8",
    },
    RuntimeFunction {
        name: "gpu_tensor_to_host",
        llvm_name: "axon_gpu_tensor_to_host",
        description: "Copy tensor to CPU: (tensor: *u8) -> *u8",
    },
    // Autograd forward ops (DELTA/D1)
    RuntimeFunction {
        name: "ag_sigmoid",
        llvm_name: "axon_ag_sigmoid",
        description: "Autograd sigmoid: (a: *u8) -> *u8",
    },
    RuntimeFunction {
        name: "ag_tanh",
        llvm_name: "axon_ag_tanh",
        description: "Autograd tanh: (a: *u8) -> *u8",
    },
    RuntimeFunction {
        name: "ag_log",
        llvm_name: "axon_ag_log",
        description: "Autograd log: (a: *u8) -> *u8",
    },
    RuntimeFunction {
        name: "ag_exp",
        llvm_name: "axon_ag_exp",
        description: "Autograd exp: (a: *u8) -> *u8",
    },
    RuntimeFunction {
        name: "ag_neg",
        llvm_name: "axon_ag_neg",
        description: "Autograd negate: (a: *u8) -> *u8",
    },
    RuntimeFunction {
        name: "ag_abs",
        llvm_name: "axon_ag_abs",
        description: "Autograd abs: (a: *u8) -> *u8",
    },
    RuntimeFunction {
        name: "ag_sqrt",
        llvm_name: "axon_ag_sqrt",
        description: "Autograd sqrt: (a: *u8) -> *u8",
    },
    RuntimeFunction {
        name: "ag_sin",
        llvm_name: "axon_ag_sin",
        description: "Autograd sin: (a: *u8) -> *u8",
    },
    RuntimeFunction {
        name: "ag_cos",
        llvm_name: "axon_ag_cos",
        description: "Autograd cos: (a: *u8) -> *u8",
    },
    RuntimeFunction {
        name: "ag_pow",
        llvm_name: "axon_ag_pow",
        description: "Autograd pow: (a: *u8, b: *u8) -> *u8",
    },
    RuntimeFunction {
        name: "ag_gelu",
        llvm_name: "axon_ag_gelu",
        description: "Autograd GELU: (a: *u8) -> *u8",
    },
    RuntimeFunction {
        name: "ag_silu",
        llvm_name: "axon_ag_silu",
        description: "Autograd SiLU: (a: *u8) -> *u8",
    },
    RuntimeFunction {
        name: "ag_leaky_relu",
        llvm_name: "axon_ag_leaky_relu",
        description: "Autograd LeakyReLU: (a: *u8) -> *u8",
    },
    // Adam optimizer (DELTA/D1)
    RuntimeFunction {
        name: "adam_init",
        llvm_name: "axon_adam_init",
        description: "Initialize Adam state: (params: **u8, n: i32) -> void",
    },
    RuntimeFunction {
        name: "adam_step",
        llvm_name: "axon_adam_step",
        description: "Adam step: (params: **u8, n: i32, lr: f64, beta1: f64, beta2: f64, eps: f64) -> void",
    },
    // Quantization (DELTA/D3)
    RuntimeFunction {
        name: "tensor_quantize",
        llvm_name: "axon_tensor_quantize",
        description: "Quantize f32 tensor to int8/uint8: (tensor: *u8, target_dtype: u8) -> *u8",
    },
    RuntimeFunction {
        name: "tensor_dequantize",
        llvm_name: "axon_tensor_dequantize",
        description: "Dequantize low-precision tensor back to f32: (tensor: *u8) -> *u8",
    },
    RuntimeFunction {
        name: "tensor_qmatmul",
        llvm_name: "axon_tensor_qmatmul",
        description: "Quantized int8 matmul with f32 accumulation: (a: *u8, b: *u8) -> *u8",
    },
    RuntimeFunction {
        name: "tensor_quantize_symmetric",
        llvm_name: "axon_tensor_quantize_symmetric",
        description: "Symmetric quantization (scale only, no zero-point): (tensor: *u8, bits: i32) -> *u8",
    },
    RuntimeFunction {
        name: "tensor_dequantize_scale",
        llvm_name: "axon_tensor_dequantize_scale",
        description: "Dequantize using explicit scale: (tensor: *u8, scale: f64) -> *u8",
    },
    RuntimeFunction {
        name: "tensor_calibrate_range",
        llvm_name: "axon_tensor_calibrate_range",
        description: "Compute min/max calibration range: (tensor: *u8, out_min: *f64, out_max: *f64) -> void",
    },
    RuntimeFunction {
        name: "tensor_quantize_dynamic",
        llvm_name: "axon_tensor_quantize_dynamic",
        description: "Dynamic quantization (compute scale per-tensor at runtime): (tensor: *u8, bits: i32) -> *u8",
    },
    RuntimeFunction {
        name: "tensor_mixed_precision_cast",
        llvm_name: "axon_tensor_mixed_precision_cast",
        description: "Cast tensor dtype for mixed precision: (tensor: *u8, target_dtype: u8) -> *u8",
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

    // Fused tensor operations (E7)
    ir.push_str("\n; Fused tensor operations\n");
    ir.push_str("declare i8* @axon_fused_linear_relu(i8*, i8*, i8*)\n");
    ir.push_str("declare i8* @axon_fused_mse(i8*, i8*)\n");
    ir.push_str("declare i8* @axon_fused_softmax_ce(i8*, i8*)\n");
    ir.push_str("declare i8* @axon_fused_elementwise(i8**, i32, i8*, i32)\n");

    // Parallel execution runtime (E8)
    ir.push_str("\n; Parallel execution runtime\n");
    ir.push_str("declare void @axon_parallel_begin(i32)\n");
    ir.push_str("declare void @axon_parallel_end()\n");

    // Thread pool runtime (GAMMA/E10)
    ir.push_str("\n; Thread pool runtime\n");
    ir.push_str("declare void @axon_threadpool_init(i32)\n");
    ir.push_str("declare void @axon_threadpool_shutdown()\n");
    ir.push_str("declare i32 @axon_threadpool_num_threads()\n");

    // Memory pool runtime (GAMMA/E11)
    ir.push_str("\n; Memory pool runtime\n");
    ir.push_str("declare void @axon_mempool_init()\n");
    ir.push_str("declare i8* @axon_mempool_alloc(i64)\n");
    ir.push_str("declare void @axon_mempool_free(i8*, i64)\n");
    ir.push_str("declare void @axon_mempool_clear()\n");
    ir.push_str("declare void @axon_mempool_stats()\n");

    // GPU dispatch stubs (GAMMA/E12)
    ir.push_str("\n; GPU dispatch stubs\n");
    ir.push_str("declare i32 @axon_gpu_init()\n");
    ir.push_str("declare void @axon_gpu_shutdown()\n");
    ir.push_str("declare i32 @axon_gpu_device_count()\n");
    ir.push_str("declare i8* @axon_gpu_alloc(i64)\n");
    ir.push_str("declare void @axon_gpu_free(i8*)\n");
    ir.push_str("declare i8* @axon_gpu_tensor_to_device(i8*, i32)\n");
    ir.push_str("declare i8* @axon_gpu_tensor_to_host(i8*)\n");

    // Autograd forward ops (DELTA/D1)
    ir.push_str("\n; Autograd forward ops\n");
    ir.push_str("declare i8* @axon_ag_sigmoid(i8*)\n");
    ir.push_str("declare i8* @axon_ag_tanh(i8*)\n");
    ir.push_str("declare i8* @axon_ag_log(i8*)\n");
    ir.push_str("declare i8* @axon_ag_exp(i8*)\n");
    ir.push_str("declare i8* @axon_ag_neg(i8*)\n");
    ir.push_str("declare i8* @axon_ag_abs(i8*)\n");
    ir.push_str("declare i8* @axon_ag_sqrt(i8*)\n");
    ir.push_str("declare i8* @axon_ag_sin(i8*)\n");
    ir.push_str("declare i8* @axon_ag_cos(i8*)\n");
    ir.push_str("declare i8* @axon_ag_pow(i8*, i8*)\n");
    ir.push_str("declare i8* @axon_ag_gelu(i8*)\n");
    ir.push_str("declare i8* @axon_ag_silu(i8*)\n");
    ir.push_str("declare i8* @axon_ag_leaky_relu(i8*)\n");

    // Adam optimizer (DELTA/D1)
    ir.push_str("\n; Adam optimizer\n");
    ir.push_str("declare void @axon_adam_init(i8**, i32)\n");
    ir.push_str("declare void @axon_adam_step(i8**, i32, double, double, double, double)\n");

    // Quantization (DELTA/D3)
    ir.push_str("\n; Quantization runtime\n");
    ir.push_str("declare i8* @axon_tensor_quantize(i8*, i8)\n");
    ir.push_str("declare i8* @axon_tensor_dequantize(i8*)\n");
    ir.push_str("declare i8* @axon_tensor_qmatmul(i8*, i8*)\n");
    ir.push_str("declare i8* @axon_tensor_quantize_symmetric(i8*, i32)\n");
    ir.push_str("declare i8* @axon_tensor_dequantize_scale(i8*, double)\n");
    ir.push_str("declare void @axon_tensor_calibrate_range(i8*, double*, double*)\n");
    ir.push_str("declare i8* @axon_tensor_quantize_dynamic(i8*, i32)\n");
    ir.push_str("declare i8* @axon_tensor_mixed_precision_cast(i8*, i8)\n");

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
    OP_SUM = 8, OP_MEAN = 9, OP_RESHAPE = 10, OP_TRANSPOSE = 11,
    OP_LOG = 12, OP_EXP = 13, OP_POW = 14, OP_NEG = 15,
    OP_ABS = 16, OP_SQRT = 17, OP_SIN = 18, OP_COS = 19,
    OP_GELU = 20, OP_SILU = 21, OP_LEAKY_RELU = 22
} GradOp;

typedef struct TapeEntry {
    GradOp op;
    void* output;     // AxonTensor* — result of forward op
    void* input_a;    // AxonTensor* — first input
    void* input_b;    // AxonTensor* — second input (NULL for unary)
} TapeEntry;

// Thread-local tape (using __declspec(thread) on Windows, __thread on Unix)
#ifdef _WIN32
static __declspec(thread) TapeEntry* g_tape = NULL;
static __declspec(thread) int32_t g_tape_len = 0;
static __declspec(thread) int32_t g_tape_cap = 0;
static __declspec(thread) int32_t g_tape_enabled = 1;
#else
static __thread TapeEntry* g_tape = NULL;
static __thread int32_t g_tape_len = 0;
static __thread int32_t g_tape_cap = 0;
static __thread int32_t g_tape_enabled = 1;
#endif

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

// Helper: element-wise unary op on tensor (creates new tensor)
static AxonTensor* tensor_unary_op_f(AxonTensor* a, float (*f32_fn)(float), double (*f64_fn)(double)) {
    AxonTensor* out = axon_tensor_new(a->shape, a->ndim, a->dtype);
    if (a->dtype == 0) {
        float *s=(float*)a->data, *d=(float*)out->data;
        for(int64_t i=0;i<a->numel;i++) d[i] = f32_fn(s[i]);
    } else {
        double *s=(double*)a->data, *d=(double*)out->data;
        for(int64_t i=0;i<a->numel;i++) d[i] = f64_fn(s[i]);
    }
    return out;
}

// Forward unary ops used by autograd
static float  _sigmoidf(float x)  { return 1.0f/(1.0f+expf(-x)); }
static double _sigmoid(double x)  { return 1.0/(1.0+exp(-x)); }
static float  _tanhf_(float x)    { return tanhf(x); }
static double _tanh_(double x)    { return tanh(x); }
static float  _negf(float x)      { return -x; }
static double _negd(double x)     { return -x; }
static float  _absf_(float x)     { return fabsf(x); }
static double _absd(double x)     { return fabs(x); }
static float  _sqrtf_(float x)    { return sqrtf(x); }
static double _sqrtd(double x)    { return sqrt(x); }
static float  _sinf_(float x)     { return sinf(x); }
static double _sind(double x)     { return sin(x); }
static float  _cosf_(float x)     { return cosf(x); }
static double _cosd(double x)     { return cos(x); }
static float  _logf_(float x)     { return logf(x > 1e-12f ? x : 1e-12f); }
static double _logd(double x)     { return log(x > 1e-12 ? x : 1e-12); }
static float  _expf_(float x)     { return expf(x); }
static double _expd(double x)     { return exp(x); }

// GELU: 0.5 * x * (1 + tanh(sqrt(2/pi) * (x + 0.044715 * x^3)))
static float  _geluf(float x) {
    float c = 0.7978845608f; // sqrt(2/pi)
    return 0.5f * x * (1.0f + tanhf(c * (x + 0.044715f * x * x * x)));
}
static double _gelud(double x) {
    double c = 0.7978845608028654;
    return 0.5 * x * (1.0 + tanh(c * (x + 0.044715 * x * x * x)));
}

// SiLU (Swish): x * sigmoid(x)
static float  _siluf(float x) { return x * _sigmoidf(x); }
static double _silud(double x) { return x * _sigmoid(x); }

// LeakyReLU: max(0.01*x, x)
static float  _leaky_reluf(float x) { return x > 0 ? x : 0.01f * x; }
static double _leaky_relud(double x) { return x > 0 ? x : 0.01 * x; }

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
void* axon_ag_sigmoid(void* a) {
    AxonTensor* t = (AxonTensor*)a;
    AxonTensor* c = tensor_unary_op_f(t, _sigmoidf, _sigmoid);
    tape_record(OP_SIGMOID, c, a, NULL);
    return c;
}
void* axon_ag_tanh(void* a) {
    AxonTensor* t = (AxonTensor*)a;
    AxonTensor* c = tensor_unary_op_f(t, _tanhf_, _tanh_);
    tape_record(OP_TANH, c, a, NULL);
    return c;
}
void* axon_ag_log(void* a) {
    AxonTensor* t = (AxonTensor*)a;
    AxonTensor* c = tensor_unary_op_f(t, _logf_, _logd);
    tape_record(OP_LOG, c, a, NULL);
    return c;
}
void* axon_ag_exp(void* a) {
    AxonTensor* t = (AxonTensor*)a;
    AxonTensor* c = tensor_unary_op_f(t, _expf_, _expd);
    tape_record(OP_EXP, c, a, NULL);
    return c;
}
void* axon_ag_neg(void* a) {
    AxonTensor* t = (AxonTensor*)a;
    AxonTensor* c = tensor_unary_op_f(t, _negf, _negd);
    tape_record(OP_NEG, c, a, NULL);
    return c;
}
void* axon_ag_abs(void* a) {
    AxonTensor* t = (AxonTensor*)a;
    AxonTensor* c = tensor_unary_op_f(t, _absf_, _absd);
    tape_record(OP_ABS, c, a, NULL);
    return c;
}
void* axon_ag_sqrt(void* a) {
    AxonTensor* t = (AxonTensor*)a;
    AxonTensor* c = tensor_unary_op_f(t, _sqrtf_, _sqrtd);
    tape_record(OP_SQRT, c, a, NULL);
    return c;
}
void* axon_ag_sin(void* a) {
    AxonTensor* t = (AxonTensor*)a;
    AxonTensor* c = tensor_unary_op_f(t, _sinf_, _sind);
    tape_record(OP_SIN, c, a, NULL);
    return c;
}
void* axon_ag_cos(void* a) {
    AxonTensor* t = (AxonTensor*)a;
    AxonTensor* c = tensor_unary_op_f(t, _cosf_, _cosd);
    tape_record(OP_COS, c, a, NULL);
    return c;
}
void* axon_ag_pow(void* a, void* b) {
    AxonTensor* ta = (AxonTensor*)a;
    AxonTensor* tb = (AxonTensor*)b;
    AxonTensor* c = axon_tensor_new(ta->shape, ta->ndim, ta->dtype);
    if (ta->dtype == 0) {
        float *sa=(float*)ta->data, *sb=(float*)tb->data, *d=(float*)c->data;
        for(int64_t i=0;i<ta->numel;i++) d[i] = powf(sa[i], sb[i % tb->numel]);
    } else {
        double *sa=(double*)ta->data, *sb=(double*)tb->data, *d=(double*)c->data;
        for(int64_t i=0;i<ta->numel;i++) d[i] = pow(sa[i], sb[i % tb->numel]);
    }
    tape_record(OP_POW, c, a, b);
    return c;
}
void* axon_ag_gelu(void* a) {
    AxonTensor* t = (AxonTensor*)a;
    AxonTensor* c = tensor_unary_op_f(t, _geluf, _gelud);
    tape_record(OP_GELU, c, a, NULL);
    return c;
}
void* axon_ag_silu(void* a) {
    AxonTensor* t = (AxonTensor*)a;
    AxonTensor* c = tensor_unary_op_f(t, _siluf, _silud);
    tape_record(OP_SILU, c, a, NULL);
    return c;
}
void* axon_ag_leaky_relu(void* a) {
    AxonTensor* t = (AxonTensor*)a;
    AxonTensor* c = tensor_unary_op_f(t, _leaky_reluf, _leaky_relud);
    tape_record(OP_LEAKY_RELU, c, a, NULL);
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

// Helper: element-wise negate
static AxonTensor* tensor_negate(AxonTensor* t) {
    return tensor_unary_op_f(t, _negf, _negd);
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

    // Reverse tape traversal (topological order by construction)
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
                    AxonTensor* neg = tensor_negate(grad_out);
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
                    AxonTensor* neg_a = tensor_negate(a);
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
            case OP_SIGMOID: {
                // d/da(sigmoid(a)) = sigmoid(a) * (1 - sigmoid(a)) * grad_out
                if (a->requires_grad) {
                    AxonTensor* ga = axon_tensor_new(a->shape, a->ndim, a->dtype);
                    if (a->dtype == 0) {
                        float *dout=(float*)out->data, *dg=(float*)grad_out->data, *dd=(float*)ga->data;
                        for(int64_t j=0;j<a->numel;j++) dd[j] = dg[j] * dout[j] * (1.0f - dout[j]);
                    } else {
                        double *dout=(double*)out->data, *dg=(double*)grad_out->data, *dd=(double*)ga->data;
                        for(int64_t j=0;j<a->numel;j++) dd[j] = dg[j] * dout[j] * (1.0 - dout[j]);
                    }
                    accum_grad(a, ga);
                    axon_tensor_release(ga);
                }
                break;
            }
            case OP_TANH: {
                // d/da(tanh(a)) = (1 - tanh(a)^2) * grad_out
                if (a->requires_grad) {
                    AxonTensor* ga = axon_tensor_new(a->shape, a->ndim, a->dtype);
                    if (a->dtype == 0) {
                        float *dout=(float*)out->data, *dg=(float*)grad_out->data, *dd=(float*)ga->data;
                        for(int64_t j=0;j<a->numel;j++) dd[j] = dg[j] * (1.0f - dout[j]*dout[j]);
                    } else {
                        double *dout=(double*)out->data, *dg=(double*)grad_out->data, *dd=(double*)ga->data;
                        for(int64_t j=0;j<a->numel;j++) dd[j] = dg[j] * (1.0 - dout[j]*dout[j]);
                    }
                    accum_grad(a, ga);
                    axon_tensor_release(ga);
                }
                break;
            }
            case OP_LOG: {
                // d/da(log(a)) = 1/a * grad_out
                if (a->requires_grad) {
                    AxonTensor* ga = axon_tensor_new(a->shape, a->ndim, a->dtype);
                    if (a->dtype == 0) {
                        float *da=(float*)a->data, *dg=(float*)grad_out->data, *dd=(float*)ga->data;
                        for(int64_t j=0;j<a->numel;j++) dd[j] = dg[j] / (da[j] > 1e-12f ? da[j] : 1e-12f);
                    } else {
                        double *da=(double*)a->data, *dg=(double*)grad_out->data, *dd=(double*)ga->data;
                        for(int64_t j=0;j<a->numel;j++) dd[j] = dg[j] / (da[j] > 1e-12 ? da[j] : 1e-12);
                    }
                    accum_grad(a, ga);
                    axon_tensor_release(ga);
                }
                break;
            }
            case OP_EXP: {
                // d/da(exp(a)) = exp(a) * grad_out = out * grad_out
                if (a->requires_grad) {
                    AxonTensor* ga = (AxonTensor*)axon_tensor_mul(grad_out, out);
                    accum_grad(a, ga);
                    axon_tensor_release(ga);
                }
                break;
            }
            case OP_POW: {
                // d/da(a^b) = b * a^(b-1) * grad_out
                // d/db(a^b) = a^b * log(a) * grad_out
                if (a->requires_grad) {
                    AxonTensor* ga = axon_tensor_new(a->shape, a->ndim, a->dtype);
                    if (a->dtype == 0) {
                        float *da=(float*)a->data, *db=(float*)b->data, *dg=(float*)grad_out->data, *dd=(float*)ga->data;
                        for(int64_t j=0;j<a->numel;j++) dd[j] = dg[j] * db[j % b->numel] * powf(da[j], db[j % b->numel] - 1.0f);
                    } else {
                        double *da=(double*)a->data, *db=(double*)b->data, *dg=(double*)grad_out->data, *dd=(double*)ga->data;
                        for(int64_t j=0;j<a->numel;j++) dd[j] = dg[j] * db[j % b->numel] * pow(da[j], db[j % b->numel] - 1.0);
                    }
                    accum_grad(a, ga);
                    axon_tensor_release(ga);
                }
                if (b && b->requires_grad) {
                    AxonTensor* gb = axon_tensor_new(b->shape, b->ndim, b->dtype);
                    if (b->dtype == 0) {
                        float *da=(float*)a->data, *dout=(float*)out->data, *dg=(float*)grad_out->data, *dd=(float*)gb->data;
                        for(int64_t j=0;j<b->numel;j++) dd[j] = dg[j] * dout[j] * logf(da[j] > 1e-12f ? da[j] : 1e-12f);
                    } else {
                        double *da=(double*)a->data, *dout=(double*)out->data, *dg=(double*)grad_out->data, *dd=(double*)gb->data;
                        for(int64_t j=0;j<b->numel;j++) dd[j] = dg[j] * dout[j] * log(da[j] > 1e-12 ? da[j] : 1e-12);
                    }
                    accum_grad(b, gb);
                    axon_tensor_release(gb);
                }
                break;
            }
            case OP_NEG:
                // d/da(-a) = -grad_out
                if (a->requires_grad) {
                    AxonTensor* ga = tensor_negate(grad_out);
                    accum_grad(a, ga);
                    axon_tensor_release(ga);
                }
                break;
            case OP_ABS: {
                // d/da(|a|) = sign(a) * grad_out
                if (a->requires_grad) {
                    AxonTensor* ga = axon_tensor_new(a->shape, a->ndim, a->dtype);
                    if (a->dtype == 0) {
                        float *da=(float*)a->data, *dg=(float*)grad_out->data, *dd=(float*)ga->data;
                        for(int64_t j=0;j<a->numel;j++) dd[j] = dg[j] * (da[j] > 0 ? 1.0f : (da[j] < 0 ? -1.0f : 0.0f));
                    } else {
                        double *da=(double*)a->data, *dg=(double*)grad_out->data, *dd=(double*)ga->data;
                        for(int64_t j=0;j<a->numel;j++) dd[j] = dg[j] * (da[j] > 0 ? 1.0 : (da[j] < 0 ? -1.0 : 0.0));
                    }
                    accum_grad(a, ga);
                    axon_tensor_release(ga);
                }
                break;
            }
            case OP_SQRT: {
                // d/da(sqrt(a)) = 1/(2*sqrt(a)) * grad_out = grad_out / (2*out)
                if (a->requires_grad) {
                    AxonTensor* ga = axon_tensor_new(a->shape, a->ndim, a->dtype);
                    if (a->dtype == 0) {
                        float *dout=(float*)out->data, *dg=(float*)grad_out->data, *dd=(float*)ga->data;
                        for(int64_t j=0;j<a->numel;j++) dd[j] = dg[j] / (2.0f * (dout[j] > 1e-12f ? dout[j] : 1e-12f));
                    } else {
                        double *dout=(double*)out->data, *dg=(double*)grad_out->data, *dd=(double*)ga->data;
                        for(int64_t j=0;j<a->numel;j++) dd[j] = dg[j] / (2.0 * (dout[j] > 1e-12 ? dout[j] : 1e-12));
                    }
                    accum_grad(a, ga);
                    axon_tensor_release(ga);
                }
                break;
            }
            case OP_SIN: {
                // d/da(sin(a)) = cos(a) * grad_out
                if (a->requires_grad) {
                    AxonTensor* ga = axon_tensor_new(a->shape, a->ndim, a->dtype);
                    if (a->dtype == 0) {
                        float *da=(float*)a->data, *dg=(float*)grad_out->data, *dd=(float*)ga->data;
                        for(int64_t j=0;j<a->numel;j++) dd[j] = dg[j] * cosf(da[j]);
                    } else {
                        double *da=(double*)a->data, *dg=(double*)grad_out->data, *dd=(double*)ga->data;
                        for(int64_t j=0;j<a->numel;j++) dd[j] = dg[j] * cos(da[j]);
                    }
                    accum_grad(a, ga);
                    axon_tensor_release(ga);
                }
                break;
            }
            case OP_COS: {
                // d/da(cos(a)) = -sin(a) * grad_out
                if (a->requires_grad) {
                    AxonTensor* ga = axon_tensor_new(a->shape, a->ndim, a->dtype);
                    if (a->dtype == 0) {
                        float *da=(float*)a->data, *dg=(float*)grad_out->data, *dd=(float*)ga->data;
                        for(int64_t j=0;j<a->numel;j++) dd[j] = -dg[j] * sinf(da[j]);
                    } else {
                        double *da=(double*)a->data, *dg=(double*)grad_out->data, *dd=(double*)ga->data;
                        for(int64_t j=0;j<a->numel;j++) dd[j] = -dg[j] * sin(da[j]);
                    }
                    accum_grad(a, ga);
                    axon_tensor_release(ga);
                }
                break;
            }
            case OP_GELU: {
                // Approximate GELU gradient
                if (a->requires_grad) {
                    AxonTensor* ga = axon_tensor_new(a->shape, a->ndim, a->dtype);
                    if (a->dtype == 0) {
                        float c = 0.7978845608f;
                        float *da=(float*)a->data, *dg=(float*)grad_out->data, *dd=(float*)ga->data;
                        for(int64_t j=0;j<a->numel;j++) {
                            float x = da[j];
                            float inner = c * (x + 0.044715f * x*x*x);
                            float t = tanhf(inner);
                            float sech2 = 1.0f - t*t;
                            dd[j] = dg[j] * (0.5f * (1.0f + t) + 0.5f * x * sech2 * c * (1.0f + 3.0f*0.044715f*x*x));
                        }
                    } else {
                        double c = 0.7978845608028654;
                        double *da=(double*)a->data, *dg=(double*)grad_out->data, *dd=(double*)ga->data;
                        for(int64_t j=0;j<a->numel;j++) {
                            double x = da[j];
                            double inner = c * (x + 0.044715 * x*x*x);
                            double t = tanh(inner);
                            double sech2 = 1.0 - t*t;
                            dd[j] = dg[j] * (0.5 * (1.0 + t) + 0.5 * x * sech2 * c * (1.0 + 3.0*0.044715*x*x));
                        }
                    }
                    accum_grad(a, ga);
                    axon_tensor_release(ga);
                }
                break;
            }
            case OP_SILU: {
                // d/da(x*sigmoid(x)) = sigmoid(x) + x*sigmoid(x)*(1-sigmoid(x))
                //                    = sigmoid(x) * (1 + x*(1-sigmoid(x)))
                if (a->requires_grad) {
                    AxonTensor* ga = axon_tensor_new(a->shape, a->ndim, a->dtype);
                    if (a->dtype == 0) {
                        float *da=(float*)a->data, *dg=(float*)grad_out->data, *dd=(float*)ga->data;
                        for(int64_t j=0;j<a->numel;j++) {
                            float s = _sigmoidf(da[j]);
                            dd[j] = dg[j] * (s + da[j] * s * (1.0f - s));
                        }
                    } else {
                        double *da=(double*)a->data, *dg=(double*)grad_out->data, *dd=(double*)ga->data;
                        for(int64_t j=0;j<a->numel;j++) {
                            double s = _sigmoid(da[j]);
                            dd[j] = dg[j] * (s + da[j] * s * (1.0 - s));
                        }
                    }
                    accum_grad(a, ga);
                    axon_tensor_release(ga);
                }
                break;
            }
            case OP_LEAKY_RELU: {
                // d/da(leaky_relu(a)) = 1 if a > 0, else 0.01
                if (a->requires_grad) {
                    AxonTensor* ga = axon_tensor_new(a->shape, a->ndim, a->dtype);
                    if (a->dtype == 0) {
                        float *da=(float*)a->data, *dg=(float*)grad_out->data, *dd=(float*)ga->data;
                        for(int64_t j=0;j<a->numel;j++) dd[j] = da[j] > 0 ? dg[j] : 0.01f * dg[j];
                    } else {
                        double *da=(double*)a->data, *dg=(double*)grad_out->data, *dd=(double*)ga->data;
                        for(int64_t j=0;j<a->numel;j++) dd[j] = da[j] > 0 ? dg[j] : 0.01 * dg[j];
                    }
                    accum_grad(a, ga);
                    axon_tensor_release(ga);
                }
                break;
            }
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

// Adam optimizer state (per-parameter)
typedef struct {
    void* m;   // first moment (AxonTensor*)
    void* v;   // second moment (AxonTensor*)
    int32_t t; // timestep
} AxonAdamState;

static AxonAdamState* axon_adam_states = NULL;
static int32_t axon_adam_n_params = 0;

// Initialize Adam optimizer state for n parameters
void axon_adam_init(void** params, int32_t n_params) {
    if (axon_adam_states) free(axon_adam_states);
    axon_adam_states = (AxonAdamState*)calloc(n_params, sizeof(AxonAdamState));
    axon_adam_n_params = n_params;
    for (int32_t i = 0; i < n_params; i++) {
        AxonTensor* p = (AxonTensor*)params[i];
        axon_adam_states[i].m = axon_tensor_zeros(p->shape, p->ndim, p->dtype);
        axon_adam_states[i].v = axon_tensor_zeros(p->shape, p->ndim, p->dtype);
        axon_adam_states[i].t = 0;
    }
}

// Adam optimizer step: params updated in-place
void axon_adam_step(void** params, int32_t n_params, double lr, double beta1, double beta2, double eps) {
    if (!axon_adam_states || axon_adam_n_params != n_params) {
        axon_adam_init(params, n_params);
    }
    for (int32_t i = 0; i < n_params; i++) {
        AxonTensor* w = (AxonTensor*)params[i];
        if (!w->grad) continue;
        AxonTensor* g = (AxonTensor*)w->grad;
        AxonAdamState* s = &axon_adam_states[i];
        AxonTensor* m = (AxonTensor*)s->m;
        AxonTensor* v = (AxonTensor*)s->v;
        s->t++;

        if (w->dtype == 0) {
            float b1=(float)beta1, b2=(float)beta2, e=(float)eps, flr=(float)lr;
            float *dw=(float*)w->data, *dg=(float*)g->data, *dm=(float*)m->data, *dv=(float*)v->data;
            float bc1 = 1.0f - powf(b1, (float)s->t);
            float bc2 = 1.0f - powf(b2, (float)s->t);
            for(int64_t j=0;j<w->numel;j++) {
                dm[j] = b1*dm[j] + (1.0f-b1)*dg[j];
                dv[j] = b2*dv[j] + (1.0f-b2)*dg[j]*dg[j];
                float m_hat = dm[j] / bc1;
                float v_hat = dv[j] / bc2;
                dw[j] -= flr * m_hat / (sqrtf(v_hat) + e);
            }
        } else {
            double b1=beta1, b2=beta2, e=eps;
            double *dw=(double*)w->data, *dg=(double*)g->data, *dm=(double*)m->data, *dv=(double*)v->data;
            double bc1 = 1.0 - pow(b1, (double)s->t);
            double bc2 = 1.0 - pow(b2, (double)s->t);
            for(int64_t j=0;j<w->numel;j++) {
                dm[j] = b1*dm[j] + (1.0-b1)*dg[j];
                dv[j] = b2*dv[j] + (1.0-b2)*dg[j]*dg[j];
                double m_hat = dm[j] / bc1;
                double v_hat = dv[j] / bc2;
                dw[j] -= lr * m_hat / (sqrt(v_hat) + e);
            }
        }
    }
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

// ════════════════════════════════════════════════════════════════
// Fused Tensor Operations (E7)
// ════════════════════════════════════════════════════════════════

// Fused linear+relu: result = relu(matmul(a, b) + bias)
// Combines matmul, bias add, and relu into a single kernel,
// eliminating 2 intermediate tensor allocations.
void* axon_fused_linear_relu(void* va, void* vb, void* vbias) {
    AxonTensor* a = (AxonTensor*)va;
    AxonTensor* b = (AxonTensor*)vb;
    AxonTensor* bias = (AxonTensor*)vbias;
    // a is (M, K), b is (K, N) → result is (M, N)
    int64_t M = a->shape[0];
    int64_t K = a->shape[a->ndim > 1 ? 1 : 0];
    int64_t N = b->shape[b->ndim > 1 ? 1 : 0];
    int64_t out_shape[2] = {M, N};
    AxonTensor* out = axon_tensor_new(out_shape, 2, a->dtype);
    if (a->dtype == 0) {
        float *da=(float*)a->data, *db=(float*)b->data;
        float *dbias=(float*)bias->data, *dout=(float*)out->data;
        for(int64_t i=0;i<M;i++) {
            for(int64_t j=0;j<N;j++) {
                float sum = 0.0f;
                for(int64_t k=0;k<K;k++) sum += da[i*K+k] * db[k*N+j];
                sum += dbias[j % bias->numel];
                dout[i*N+j] = sum > 0.0f ? sum : 0.0f; // relu
            }
        }
    } else {
        double *da=(double*)a->data, *db=(double*)b->data;
        double *dbias=(double*)bias->data, *dout=(double*)out->data;
        for(int64_t i=0;i<M;i++) {
            for(int64_t j=0;j<N;j++) {
                double sum = 0.0;
                for(int64_t k=0;k<K;k++) sum += da[i*K+k] * db[k*N+j];
                sum += dbias[j % bias->numel];
                dout[i*N+j] = sum > 0.0 ? sum : 0.0; // relu
            }
        }
    }
    return out;
}

// Fused MSE loss: result = mean((pred - target)^2)
// Combines subtract, square, and mean into a single pass.
void* axon_fused_mse(void* vpred, void* vtarget) {
    AxonTensor* pred = (AxonTensor*)vpred;
    AxonTensor* target = (AxonTensor*)vtarget;
    int64_t scalar_shape[1] = {1};
    AxonTensor* loss = axon_tensor_new(scalar_shape, 1, pred->dtype);
    if (pred->dtype == 0) {
        float *dp=(float*)pred->data, *dt=(float*)target->data;
        float sum = 0.0f;
        for(int64_t i=0;i<pred->numel;i++) { float d=dp[i]-dt[i]; sum+=d*d; }
        ((float*)loss->data)[0] = sum / (float)pred->numel;
    } else {
        double *dp=(double*)pred->data, *dt=(double*)target->data;
        double sum = 0.0;
        for(int64_t i=0;i<pred->numel;i++) { double d=dp[i]-dt[i]; sum+=d*d; }
        ((double*)loss->data)[0] = sum / (double)pred->numel;
    }
    return loss;
}

// Fused softmax + cross-entropy loss
void* axon_fused_softmax_ce(void* vlogits, void* vtarget) {
    AxonTensor* logits = (AxonTensor*)vlogits;
    AxonTensor* target = (AxonTensor*)vtarget;
    int64_t scalar_shape[1] = {1};
    AxonTensor* loss = axon_tensor_new(scalar_shape, 1, logits->dtype);
    if (logits->dtype == 0) {
        float *dl=(float*)logits->data, *dt=(float*)target->data;
        // Find max for numerical stability
        float max_val = dl[0];
        for(int64_t i=1;i<logits->numel;i++) if(dl[i]>max_val) max_val=dl[i];
        // Compute log-sum-exp
        float sum_exp = 0.0f;
        for(int64_t i=0;i<logits->numel;i++) sum_exp += expf(dl[i]-max_val);
        float log_sum = logf(sum_exp) + max_val;
        // Cross-entropy: -sum(target * log(softmax))
        float ce = 0.0f;
        for(int64_t i=0;i<logits->numel;i++) ce -= dt[i] * (dl[i] - log_sum);
        ((float*)loss->data)[0] = ce;
    } else {
        double *dl=(double*)logits->data, *dt=(double*)target->data;
        double max_val = dl[0];
        for(int64_t i=1;i<logits->numel;i++) if(dl[i]>max_val) max_val=dl[i];
        double sum_exp = 0.0;
        for(int64_t i=0;i<logits->numel;i++) sum_exp += exp(dl[i]-max_val);
        double log_sum = log(sum_exp) + max_val;
        double ce = 0.0;
        for(int64_t i=0;i<logits->numel;i++) ce -= dt[i] * (dl[i] - log_sum);
        ((double*)loss->data)[0] = ce;
    }
    return loss;
}

// Fused element-wise chain: applies a sequence of element-wise ops in one pass.
// ops encoding: 0=add, 1=sub, 2=mul, 3=div, 4=relu(unary)
// tensors[0] and tensors[1] are the first op's inputs, tensors[2..] are
// the additional right-hand operands for subsequent binary ops.
void* axon_fused_elementwise(void** tensors, int32_t n_tensors, void* ops_buf, int32_t n_ops) {
    if (n_tensors < 2 || n_ops < 1) return NULL;
    AxonTensor* first = (AxonTensor*)tensors[0];
    AxonTensor* out = axon_tensor_new(first->shape, first->ndim, first->dtype);
    uint8_t* ops = (uint8_t*)ops_buf;
    int64_t n = first->numel;

    if (first->dtype == 0) {
        float* result = (float*)out->data;
        // Initialize with first op
        float* lhs = (float*)first->data;
        float* rhs = (float*)((AxonTensor*)tensors[1])->data;
        for(int64_t i=0;i<n;i++) {
            switch(ops[0]) {
                case 0: result[i]=lhs[i]+rhs[i]; break;
                case 1: result[i]=lhs[i]-rhs[i]; break;
                case 2: result[i]=lhs[i]*rhs[i]; break;
                case 3: result[i]=rhs[i]!=0?lhs[i]/rhs[i]:0; break;
                case 4: result[i]=lhs[i]>0?lhs[i]:0; break;
                default: result[i]=lhs[i]; break;
            }
        }
        // Apply subsequent ops
        int32_t tensor_idx = 2;
        for(int32_t op_idx=1; op_idx<n_ops; op_idx++) {
            if(ops[op_idx] == 4) { // relu is unary
                for(int64_t i=0;i<n;i++) result[i]=result[i]>0?result[i]:0;
            } else if(tensor_idx < n_tensors) {
                float* r = (float*)((AxonTensor*)tensors[tensor_idx])->data;
                for(int64_t i=0;i<n;i++) {
                    switch(ops[op_idx]) {
                        case 0: result[i]=result[i]+r[i]; break;
                        case 1: result[i]=result[i]-r[i]; break;
                        case 2: result[i]=result[i]*r[i]; break;
                        case 3: result[i]=r[i]!=0?result[i]/r[i]:0; break;
                        default: break;
                    }
                }
                tensor_idx++;
            }
        }
    } else {
        double* result = (double*)out->data;
        double* lhs = (double*)first->data;
        double* rhs = (double*)((AxonTensor*)tensors[1])->data;
        for(int64_t i=0;i<n;i++) {
            switch(ops[0]) {
                case 0: result[i]=lhs[i]+rhs[i]; break;
                case 1: result[i]=lhs[i]-rhs[i]; break;
                case 2: result[i]=lhs[i]*rhs[i]; break;
                case 3: result[i]=rhs[i]!=0?lhs[i]/rhs[i]:0; break;
                case 4: result[i]=lhs[i]>0?lhs[i]:0; break;
                default: result[i]=lhs[i]; break;
            }
        }
        int32_t tensor_idx = 2;
        for(int32_t op_idx=1; op_idx<n_ops; op_idx++) {
            if(ops[op_idx] == 4) {
                for(int64_t i=0;i<n;i++) result[i]=result[i]>0?result[i]:0;
            } else if(tensor_idx < n_tensors) {
                double* r = (double*)((AxonTensor*)tensors[tensor_idx])->data;
                for(int64_t i=0;i<n;i++) {
                    switch(ops[op_idx]) {
                        case 0: result[i]=result[i]+r[i]; break;
                        case 1: result[i]=result[i]-r[i]; break;
                        case 2: result[i]=result[i]*r[i]; break;
                        case 3: result[i]=r[i]!=0?result[i]/r[i]:0; break;
                        default: break;
                    }
                }
                tensor_idx++;
            }
        }
    }
    return out;
}

// ════════════════════════════════════════════════════════════════
// Thread Pool Runtime (GAMMA/E10)
// ════════════════════════════════════════════════════════════════
// Cross-platform thread pool using Win32 threads on Windows,
// pthreads on Unix. Supports work submission and barrier-based
// parallel regions.

#ifdef _WIN32
#include <windows.h>
#else
#include <pthread.h>
#include <unistd.h>
#endif

typedef struct AxonTask {
    void (*fn_ptr)(void*);
    void* arg;
    struct AxonTask* next;
} AxonTask;

typedef struct {
#ifdef _WIN32
    HANDLE* threads;
    HANDLE queue_mutex;
    HANDLE queue_cond;      // event for signaling
    HANDLE done_cond;       // event for barrier
    CRITICAL_SECTION cs;
#else
    pthread_t* threads;
    pthread_mutex_t queue_mutex;
    pthread_cond_t queue_cond;
    pthread_cond_t done_cond;
#endif
    AxonTask* head;
    AxonTask* tail;
    int32_t num_threads;
    int32_t active_tasks;       // tasks currently being executed
    int32_t pending_tasks;      // tasks in queue + being executed
    int32_t shutdown;
} AxonThreadPool;

static AxonThreadPool* axon_global_pool = NULL;

#ifdef _WIN32
static DWORD WINAPI axon_worker_thread(LPVOID arg) {
    AxonThreadPool* pool = (AxonThreadPool*)arg;
    for (;;) {
        EnterCriticalSection(&pool->cs);
        while (pool->head == NULL && !pool->shutdown) {
            LeaveCriticalSection(&pool->cs);
            WaitForSingleObject(pool->queue_cond, INFINITE);
            EnterCriticalSection(&pool->cs);
        }
        if (pool->shutdown && pool->head == NULL) {
            LeaveCriticalSection(&pool->cs);
            return 0;
        }
        AxonTask* task = pool->head;
        pool->head = task->next;
        if (pool->head == NULL) pool->tail = NULL;
        pool->active_tasks++;
        LeaveCriticalSection(&pool->cs);

        task->fn_ptr(task->arg);
        free(task);

        EnterCriticalSection(&pool->cs);
        pool->active_tasks--;
        pool->pending_tasks--;
        if (pool->pending_tasks == 0) SetEvent(pool->done_cond);
        LeaveCriticalSection(&pool->cs);
    }
    return 0;
}
#else
static void* axon_worker_thread(void* arg) {
    AxonThreadPool* pool = (AxonThreadPool*)arg;
    for (;;) {
        pthread_mutex_lock(&pool->queue_mutex);
        while (pool->head == NULL && !pool->shutdown) {
            pthread_cond_wait(&pool->queue_cond, &pool->queue_mutex);
        }
        if (pool->shutdown && pool->head == NULL) {
            pthread_mutex_unlock(&pool->queue_mutex);
            return NULL;
        }
        AxonTask* task = pool->head;
        pool->head = task->next;
        if (pool->head == NULL) pool->tail = NULL;
        pool->active_tasks++;
        pthread_mutex_unlock(&pool->queue_mutex);

        task->fn_ptr(task->arg);
        free(task);

        pthread_mutex_lock(&pool->queue_mutex);
        pool->active_tasks--;
        pool->pending_tasks--;
        if (pool->pending_tasks == 0) pthread_cond_broadcast(&pool->done_cond);
        pthread_mutex_unlock(&pool->queue_mutex);
    }
    return NULL;
}
#endif

void axon_threadpool_init(int32_t num_threads) {
    if (axon_global_pool != NULL) return; // already initialized
    if (num_threads <= 0) {
#ifdef _WIN32
        SYSTEM_INFO si;
        GetSystemInfo(&si);
        num_threads = (int32_t)si.dwNumberOfProcessors;
#else
        num_threads = (int32_t)sysconf(_SC_NPROCESSORS_ONLN);
#endif
        if (num_threads <= 0) num_threads = 4;
    }
    AxonThreadPool* pool = (AxonThreadPool*)calloc(1, sizeof(AxonThreadPool));
    pool->num_threads = num_threads;
    pool->head = NULL;
    pool->tail = NULL;
    pool->active_tasks = 0;
    pool->pending_tasks = 0;
    pool->shutdown = 0;
#ifdef _WIN32
    InitializeCriticalSection(&pool->cs);
    pool->queue_cond = CreateEvent(NULL, FALSE, FALSE, NULL);
    pool->done_cond = CreateEvent(NULL, TRUE, FALSE, NULL);  // manual reset
    pool->threads = (HANDLE*)malloc(num_threads * sizeof(HANDLE));
    for (int32_t i = 0; i < num_threads; i++) {
        pool->threads[i] = CreateThread(NULL, 0, axon_worker_thread, pool, 0, NULL);
    }
#else
    pthread_mutex_init(&pool->queue_mutex, NULL);
    pthread_cond_init(&pool->queue_cond, NULL);
    pthread_cond_init(&pool->done_cond, NULL);
    pool->threads = (pthread_t*)malloc(num_threads * sizeof(pthread_t));
    for (int32_t i = 0; i < num_threads; i++) {
        pthread_create(&pool->threads[i], NULL, axon_worker_thread, pool);
    }
#endif
    axon_global_pool = pool;
}

void axon_threadpool_shutdown(void) {
    AxonThreadPool* pool = axon_global_pool;
    if (pool == NULL) return;
#ifdef _WIN32
    EnterCriticalSection(&pool->cs);
    pool->shutdown = 1;
    LeaveCriticalSection(&pool->cs);
    // Wake all workers
    for (int32_t i = 0; i < pool->num_threads; i++) SetEvent(pool->queue_cond);
    WaitForMultipleObjects(pool->num_threads, pool->threads, TRUE, INFINITE);
    for (int32_t i = 0; i < pool->num_threads; i++) CloseHandle(pool->threads[i]);
    CloseHandle(pool->queue_cond);
    CloseHandle(pool->done_cond);
    DeleteCriticalSection(&pool->cs);
#else
    pthread_mutex_lock(&pool->queue_mutex);
    pool->shutdown = 1;
    pthread_cond_broadcast(&pool->queue_cond);
    pthread_mutex_unlock(&pool->queue_mutex);
    for (int32_t i = 0; i < pool->num_threads; i++) {
        pthread_join(pool->threads[i], NULL);
    }
    pthread_mutex_destroy(&pool->queue_mutex);
    pthread_cond_destroy(&pool->queue_cond);
    pthread_cond_destroy(&pool->done_cond);
#endif
    // Drain any remaining tasks
    AxonTask* t = pool->head;
    while (t) { AxonTask* n = t->next; free(t); t = n; }
    free(pool->threads);
    free(pool);
    axon_global_pool = NULL;
}

int32_t axon_threadpool_num_threads(void) {
    if (axon_global_pool == NULL) return 0;
    return axon_global_pool->num_threads;
}

static void axon_threadpool_submit(void (*fn)(void*), void* arg) {
    AxonThreadPool* pool = axon_global_pool;
    if (pool == NULL) { fn(arg); return; }  // fallback: run inline
    AxonTask* task = (AxonTask*)malloc(sizeof(AxonTask));
    task->fn_ptr = fn;
    task->arg = arg;
    task->next = NULL;
#ifdef _WIN32
    EnterCriticalSection(&pool->cs);
    if (pool->tail) pool->tail->next = task; else pool->head = task;
    pool->tail = task;
    pool->pending_tasks++;
    ResetEvent(pool->done_cond);
    LeaveCriticalSection(&pool->cs);
    SetEvent(pool->queue_cond);
#else
    pthread_mutex_lock(&pool->queue_mutex);
    if (pool->tail) pool->tail->next = task; else pool->head = task;
    pool->tail = task;
    pool->pending_tasks++;
    pthread_cond_signal(&pool->queue_cond);
    pthread_mutex_unlock(&pool->queue_mutex);
#endif
}

static void axon_threadpool_wait(void) {
    AxonThreadPool* pool = axon_global_pool;
    if (pool == NULL) return;
#ifdef _WIN32
    EnterCriticalSection(&pool->cs);
    while (pool->pending_tasks > 0) {
        LeaveCriticalSection(&pool->cs);
        WaitForSingleObject(pool->done_cond, INFINITE);
        EnterCriticalSection(&pool->cs);
    }
    LeaveCriticalSection(&pool->cs);
#else
    pthread_mutex_lock(&pool->queue_mutex);
    while (pool->pending_tasks > 0) {
        pthread_cond_wait(&pool->done_cond, &pool->queue_mutex);
    }
    pthread_mutex_unlock(&pool->queue_mutex);
#endif
}

// ── Parallel Region Implementation (replaces E8 no-op stubs) ──
// axon_parallel_begin/end now integrate with the thread pool.
// Each tensor op between begin/end is dispatched to the pool.
// The parallel dispatch state is tracked on a per-region stack.

#define AXON_MAX_PARALLEL_DEPTH 16

typedef struct {
    int32_t count;     // number of independent tasks in this region
    int32_t submitted; // tasks submitted so far
} AxonParallelRegion;

static AxonParallelRegion axon_parallel_stack[AXON_MAX_PARALLEL_DEPTH];
static int32_t axon_parallel_depth = 0;

void axon_parallel_begin(int32_t count) {
    // Lazily initialize thread pool on first parallel region
    if (axon_global_pool == NULL) axon_threadpool_init(0);
    if (axon_parallel_depth < AXON_MAX_PARALLEL_DEPTH) {
        axon_parallel_stack[axon_parallel_depth].count = count;
        axon_parallel_stack[axon_parallel_depth].submitted = 0;
    }
    axon_parallel_depth++;
}

void axon_parallel_end(void) {
    // Wait for all tasks in the current parallel region to finish
    axon_threadpool_wait();
    if (axon_parallel_depth > 0) axon_parallel_depth--;
}

// ════════════════════════════════════════════════════════════════
// Memory Pool Runtime (GAMMA/E11)
// ════════════════════════════════════════════════════════════════
// Slab-based memory pool that caches freed allocations by size
// bucket. Reduces malloc overhead in training loops where tensors
// of the same size are repeatedly allocated and freed.

#define AXON_MEMPOOL_NUM_BUCKETS 32
#define AXON_MEMPOOL_MAX_CACHED  64   // max cached blocks per bucket

typedef struct AxonMemBlock {
    struct AxonMemBlock* next;
    int64_t size;
} AxonMemBlock;

typedef struct {
    AxonMemBlock* free_list;
    int32_t count;
    int64_t bucket_size;  // size of allocations in this bucket
} AxonMemBucket;

typedef struct {
    AxonMemBucket buckets[AXON_MEMPOOL_NUM_BUCKETS];
    int64_t total_allocs;
    int64_t pool_hits;
    int64_t pool_misses;
#ifdef _WIN32
    CRITICAL_SECTION lock;
#else
    pthread_mutex_t lock;
#endif
} AxonMemPool;

static AxonMemPool* axon_global_mempool = NULL;

// Bucket sizes: 64, 128, 256, ... up to 64*2^31 (powers of 2 starting at 64)
static int32_t axon_mempool_bucket_index(int64_t size) {
    if (size <= 64) return 0;
    int32_t idx = 0;
    int64_t bucket = 64;
    while (bucket < size && idx < AXON_MEMPOOL_NUM_BUCKETS - 1) {
        bucket *= 2;
        idx++;
    }
    return idx;
}

static int64_t axon_mempool_bucket_size(int32_t idx) {
    int64_t size = 64;
    for (int32_t i = 0; i < idx; i++) size *= 2;
    return size;
}

void axon_mempool_init(void) {
    if (axon_global_mempool != NULL) return;
    AxonMemPool* pool = (AxonMemPool*)calloc(1, sizeof(AxonMemPool));
    for (int32_t i = 0; i < AXON_MEMPOOL_NUM_BUCKETS; i++) {
        pool->buckets[i].free_list = NULL;
        pool->buckets[i].count = 0;
        pool->buckets[i].bucket_size = axon_mempool_bucket_size(i);
    }
    pool->total_allocs = 0;
    pool->pool_hits = 0;
    pool->pool_misses = 0;
#ifdef _WIN32
    InitializeCriticalSection(&pool->lock);
#else
    pthread_mutex_init(&pool->lock, NULL);
#endif
    axon_global_mempool = pool;
}

void* axon_mempool_alloc(int64_t size) {
    if (size <= 0) return NULL;
    // Lazily initialize on first alloc
    if (axon_global_mempool == NULL) axon_mempool_init();
    AxonMemPool* pool = axon_global_mempool;
    int32_t idx = axon_mempool_bucket_index(size);

#ifdef _WIN32
    EnterCriticalSection(&pool->lock);
#else
    pthread_mutex_lock(&pool->lock);
#endif
    pool->total_allocs++;
    AxonMemBucket* bucket = &pool->buckets[idx];
    void* ptr = NULL;

    if (bucket->free_list != NULL) {
        // Reuse a cached block
        AxonMemBlock* block = bucket->free_list;
        bucket->free_list = block->next;
        bucket->count--;
        pool->pool_hits++;
        ptr = (void*)block;  // reuse the block memory itself
    } else {
        pool->pool_misses++;
    }

#ifdef _WIN32
    LeaveCriticalSection(&pool->lock);
#else
    pthread_mutex_unlock(&pool->lock);
#endif

    if (ptr == NULL) {
        // Allocate from system
        int64_t alloc_size = bucket->bucket_size;
        if (alloc_size < size) alloc_size = size;
        ptr = malloc((size_t)alloc_size);
    }
    return ptr;
}

void axon_mempool_free(void* ptr, int64_t size) {
    if (ptr == NULL || size <= 0) return;
    if (axon_global_mempool == NULL) { free(ptr); return; }
    AxonMemPool* pool = axon_global_mempool;
    int32_t idx = axon_mempool_bucket_index(size);

#ifdef _WIN32
    EnterCriticalSection(&pool->lock);
#else
    pthread_mutex_lock(&pool->lock);
#endif
    AxonMemBucket* bucket = &pool->buckets[idx];

    if (bucket->count < AXON_MEMPOOL_MAX_CACHED) {
        // Cache the block
        AxonMemBlock* block = (AxonMemBlock*)ptr;
        block->size = size;
        block->next = bucket->free_list;
        bucket->free_list = block;
        bucket->count++;
#ifdef _WIN32
        LeaveCriticalSection(&pool->lock);
#else
        pthread_mutex_unlock(&pool->lock);
#endif
    } else {
        // Bucket full, free to system
#ifdef _WIN32
        LeaveCriticalSection(&pool->lock);
#else
        pthread_mutex_unlock(&pool->lock);
#endif
        free(ptr);
    }
}

void axon_mempool_clear(void) {
    if (axon_global_mempool == NULL) return;
    AxonMemPool* pool = axon_global_mempool;
#ifdef _WIN32
    EnterCriticalSection(&pool->lock);
#else
    pthread_mutex_lock(&pool->lock);
#endif
    for (int32_t i = 0; i < AXON_MEMPOOL_NUM_BUCKETS; i++) {
        AxonMemBlock* block = pool->buckets[i].free_list;
        while (block) {
            AxonMemBlock* next = block->next;
            free(block);
            block = next;
        }
        pool->buckets[i].free_list = NULL;
        pool->buckets[i].count = 0;
    }
#ifdef _WIN32
    LeaveCriticalSection(&pool->lock);
#else
    pthread_mutex_unlock(&pool->lock);
#endif
}

void axon_mempool_stats(void) {
    if (axon_global_mempool == NULL) { printf("Memory pool: not initialized\n"); return; }
    AxonMemPool* pool = axon_global_mempool;
    printf("Memory pool stats:\n");
    printf("  Total allocations: %lld\n", (long long)pool->total_allocs);
    printf("  Pool hits:         %lld\n", (long long)pool->pool_hits);
    printf("  Pool misses:       %lld\n", (long long)pool->pool_misses);
    if (pool->total_allocs > 0) {
        printf("  Hit rate:          %.1f%%\n",
               100.0 * pool->pool_hits / pool->total_allocs);
    }
    int32_t total_cached = 0;
    for (int32_t i = 0; i < AXON_MEMPOOL_NUM_BUCKETS; i++) {
        total_cached += pool->buckets[i].count;
    }
    printf("  Cached blocks:     %d\n", total_cached);
}

// ════════════════════════════════════════════════════════════════
// GPU Dispatch Stubs (GAMMA/E12)
// ════════════════════════════════════════════════════════════════
// Compile-time GPU support detection. When AXON_USE_CUDA or
// AXON_USE_ROCM are defined, real GPU dispatch is enabled.
// Otherwise these are graceful CPU-only stubs.

static int32_t axon_gpu_initialized = 0;

int32_t axon_gpu_init(void) {
#if defined(AXON_USE_CUDA) || defined(AXON_USE_ROCM)
    // Placeholder for real GPU initialization
    axon_gpu_initialized = 1;
    return 0;
#else
    axon_gpu_initialized = 0;
    return -1;  // no GPU support compiled in
#endif
}

void axon_gpu_shutdown(void) {
#if defined(AXON_USE_CUDA) || defined(AXON_USE_ROCM)
    // Placeholder for real GPU shutdown
#endif
    axon_gpu_initialized = 0;
}

int32_t axon_gpu_device_count(void) {
#if defined(AXON_USE_CUDA) || defined(AXON_USE_ROCM)
    return 1;  // placeholder
#else
    return 0;
#endif
}

void* axon_gpu_alloc(int64_t size) {
    (void)size;
#if defined(AXON_USE_CUDA) || defined(AXON_USE_ROCM)
    return NULL;  // placeholder for cudaMalloc/hipMalloc
#else
    fprintf(stderr, "axon: GPU allocation not available (compiled without GPU support)\n");
    return NULL;
#endif
}

void axon_gpu_free(void* ptr) {
    (void)ptr;
#if defined(AXON_USE_CUDA) || defined(AXON_USE_ROCM)
    // placeholder for cudaFree/hipFree
#else
    // no-op on CPU-only builds
#endif
}

void* axon_gpu_tensor_to_device(void* tensor, int32_t device_id) {
    (void)device_id;
#if defined(AXON_USE_CUDA) || defined(AXON_USE_ROCM)
    return tensor;  // placeholder: would copy data to GPU
#else
    fprintf(stderr, "axon: GPU transfer not available (compiled without GPU support)\n");
    return tensor;  // return original tensor on CPU-only builds
#endif
}

void* axon_gpu_tensor_to_host(void* tensor) {
#if defined(AXON_USE_CUDA) || defined(AXON_USE_ROCM)
    return tensor;  // placeholder: would copy data to CPU
#else
    return tensor;  // already on host
#endif
}

// ══════════════════════════════════════════════════════════════
// QUANTIZATION — dtype casting, int8 compute, calibration
// ══════════════════════════════════════════════════════════════

// Dtype codes: 0=f32, 1=f64, 2=i32, 3=i64, 4=bool/u8, 5=i8, 6=f16, 7=bf16

// Quantize: f32 tensor → int8/uint8 (affine, per-tensor scale+zero_point)
void* axon_tensor_quantize(void* vtensor, uint8_t target_dtype) {
    AxonTensor* t = (AxonTensor*)vtensor;
    if (t->dtype != 0) return vtensor;  // only quantize f32 tensors

    AxonTensor* q = axon_tensor_new(t->shape, t->ndim, target_dtype == 5 ? 5 : 4);
    float* src = (float*)t->data;

    // Find min/max for affine quantization
    float vmin = src[0], vmax = src[0];
    for (int64_t i = 1; i < t->numel; i++) {
        if (src[i] < vmin) vmin = src[i];
        if (src[i] > vmax) vmax = src[i];
    }

    float scale = (vmax - vmin) / 255.0f;
    if (scale < 1e-10f) scale = 1e-10f;

    if (target_dtype == 5) {
        // Signed int8: range [-128, 127]
        int8_t* dst = (int8_t*)q->data;
        float zero_point = -128.0f - vmin / scale;
        for (int64_t i = 0; i < t->numel; i++) {
            float v = src[i] / scale + zero_point;
            if (v < -128.0f) v = -128.0f;
            if (v > 127.0f) v = 127.0f;
            dst[i] = (int8_t)(v + (v >= 0 ? 0.5f : -0.5f));
        }
    } else {
        // Unsigned uint8: range [0, 255]
        uint8_t* dst = (uint8_t*)q->data;
        for (int64_t i = 0; i < t->numel; i++) {
            float v = (src[i] - vmin) / scale;
            if (v < 0.0f) v = 0.0f;
            if (v > 255.0f) v = 255.0f;
            dst[i] = (uint8_t)(v + 0.5f);
        }
    }
    return q;
}

// Dequantize: int8/uint8 tensor → f32
void* axon_tensor_dequantize(void* vtensor) {
    AxonTensor* t = (AxonTensor*)vtensor;
    if (t->dtype == 0 || t->dtype == 1) return vtensor;  // already float

    AxonTensor* out = axon_tensor_new(t->shape, t->ndim, 0);
    float* dst = (float*)out->data;

    if (t->dtype == 5) {
        // int8 → f32 (assume scale=1, zero_point=0 for simple case)
        int8_t* src = (int8_t*)t->data;
        for (int64_t i = 0; i < t->numel; i++) {
            dst[i] = (float)src[i];
        }
    } else if (t->dtype == 4) {
        // uint8 → f32
        uint8_t* src = (uint8_t*)t->data;
        for (int64_t i = 0; i < t->numel; i++) {
            dst[i] = (float)src[i];
        }
    } else if (t->dtype == 2) {
        // i32 → f32
        int32_t* src = (int32_t*)t->data;
        for (int64_t i = 0; i < t->numel; i++) {
            dst[i] = (float)src[i];
        }
    }
    return out;
}

// Quantized int8 matmul: accumulate in int32, return f32 result
void* axon_tensor_qmatmul(void* va, void* vb) {
    AxonTensor* a = (AxonTensor*)va;
    AxonTensor* b = (AxonTensor*)vb;
    if (a->ndim < 2 || b->ndim < 2) return NULL;

    int64_t M = a->shape[a->ndim - 2];
    int64_t K = a->shape[a->ndim - 1];
    int64_t N = b->shape[b->ndim - 1];
    int64_t out_shape[2] = {M, N};
    AxonTensor* out = axon_tensor_new(out_shape, 2, 0);  // f32 output
    float* dst = (float*)out->data;

    if (a->dtype == 5 && b->dtype == 5) {
        // int8 x int8 → int32 accumulation → f32
        int8_t* da = (int8_t*)a->data;
        int8_t* db = (int8_t*)b->data;
        for (int64_t i = 0; i < M; i++) {
            for (int64_t j = 0; j < N; j++) {
                int32_t acc = 0;
                for (int64_t k = 0; k < K; k++) {
                    acc += (int32_t)da[i * K + k] * (int32_t)db[k * N + j];
                }
                dst[i * N + j] = (float)acc;
            }
        }
    } else {
        // Fallback: treat as uint8
        uint8_t* da = (uint8_t*)a->data;
        uint8_t* db = (uint8_t*)b->data;
        for (int64_t i = 0; i < M; i++) {
            for (int64_t j = 0; j < N; j++) {
                int32_t acc = 0;
                for (int64_t k = 0; k < K; k++) {
                    acc += (int32_t)da[i * K + k] * (int32_t)db[k * N + j];
                }
                dst[i * N + j] = (float)acc;
            }
        }
    }
    return out;
}

// Symmetric quantization: centered at zero, scale only (no zero-point)
void* axon_tensor_quantize_symmetric(void* vtensor, int32_t bits) {
    AxonTensor* t = (AxonTensor*)vtensor;
    if (t->dtype != 0) return vtensor;

    int32_t qmax = (1 << (bits - 1)) - 1;  // e.g., 127 for 8-bit
    AxonTensor* q = axon_tensor_new(t->shape, t->ndim, 5);  // int8 output
    float* src = (float*)t->data;
    int8_t* dst = (int8_t*)q->data;

    // Find absolute max
    float absmax = 0.0f;
    for (int64_t i = 0; i < t->numel; i++) {
        float v = src[i] < 0 ? -src[i] : src[i];
        if (v > absmax) absmax = v;
    }

    float scale = absmax / (float)qmax;
    if (scale < 1e-10f) scale = 1e-10f;

    for (int64_t i = 0; i < t->numel; i++) {
        float v = src[i] / scale;
        if (v > (float)qmax) v = (float)qmax;
        if (v < -(float)qmax) v = -(float)qmax;
        dst[i] = (int8_t)(v + (v >= 0 ? 0.5f : -0.5f));
    }
    return q;
}

// Dequantize with explicit scale factor
void* axon_tensor_dequantize_scale(void* vtensor, double scale) {
    AxonTensor* t = (AxonTensor*)vtensor;
    AxonTensor* out = axon_tensor_new(t->shape, t->ndim, 0);
    float* dst = (float*)out->data;
    float fscale = (float)scale;

    if (t->dtype == 5) {
        int8_t* src = (int8_t*)t->data;
        for (int64_t i = 0; i < t->numel; i++) {
            dst[i] = (float)src[i] * fscale;
        }
    } else if (t->dtype == 4) {
        uint8_t* src = (uint8_t*)t->data;
        for (int64_t i = 0; i < t->numel; i++) {
            dst[i] = (float)src[i] * fscale;
        }
    } else {
        // Already float — just scale
        float* src = (float*)t->data;
        for (int64_t i = 0; i < t->numel; i++) {
            dst[i] = src[i] * fscale;
        }
    }
    return out;
}

// Calibrate: compute per-tensor min and max for quantization range
void axon_tensor_calibrate_range(void* vtensor, double* out_min, double* out_max) {
    AxonTensor* t = (AxonTensor*)vtensor;
    if (t->dtype != 0 || t->numel == 0) {
        *out_min = 0.0;
        *out_max = 0.0;
        return;
    }
    float* src = (float*)t->data;
    float vmin = src[0], vmax = src[0];
    for (int64_t i = 1; i < t->numel; i++) {
        if (src[i] < vmin) vmin = src[i];
        if (src[i] > vmax) vmax = src[i];
    }
    *out_min = (double)vmin;
    *out_max = (double)vmax;
}

// Dynamic quantization: compute scale at runtime, then quantize
void* axon_tensor_quantize_dynamic(void* vtensor, int32_t bits) {
    AxonTensor* t = (AxonTensor*)vtensor;
    if (t->dtype != 0) return vtensor;

    // Compute symmetric scale on the fly
    return axon_tensor_quantize_symmetric(vtensor, bits);
}

// Mixed-precision dtype cast (e.g., f32→f16 or f32→bf16)
void* axon_tensor_mixed_precision_cast(void* vtensor, uint8_t target_dtype) {
    AxonTensor* t = (AxonTensor*)vtensor;

    // For CPU-only: truncate f32 to reduced precision via rounding
    // (Real f16/bf16 hardware would use native conversion)
    if (t->dtype == 0 && (target_dtype == 6 || target_dtype == 7)) {
        // f32 → f16/bf16: simulate reduced precision by rounding
        AxonTensor* out = axon_tensor_new(t->shape, t->ndim, 0);  // keep as f32 storage
        float* src = (float*)t->data;
        float* dst = (float*)out->data;
        for (int64_t i = 0; i < t->numel; i++) {
            if (target_dtype == 7) {
                // BFloat16: truncate lower 16 bits of mantissa
                uint32_t bits;
                memcpy(&bits, &src[i], 4);
                bits &= 0xFFFF0000u;  // zero lower 16 bits
                memcpy(&dst[i], &bits, 4);
            } else {
                // Float16: clamp + round (simplified)
                float v = src[i];
                if (v > 65504.0f) v = 65504.0f;
                if (v < -65504.0f) v = -65504.0f;
                dst[i] = v;
            }
        }
        return out;
    }
    // No-op for same dtype or unsupported cast
    return vtensor;
}
"#.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runtime_functions_count() {
        assert_eq!(RUNTIME_FUNCTIONS.len(), 83);
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

    #[test]
    fn generate_c_source_contains_threadpool() {
        let src = generate_runtime_c_source();
        assert!(src.contains("void axon_threadpool_init(int32_t num_threads)"));
        assert!(src.contains("void axon_threadpool_shutdown(void)"));
        assert!(src.contains("int32_t axon_threadpool_num_threads(void)"));
        assert!(src.contains("AxonThreadPool"));
        assert!(src.contains("axon_worker_thread"));
        assert!(src.contains("axon_threadpool_submit"));
        assert!(src.contains("axon_threadpool_wait"));
    }

    #[test]
    fn generate_c_source_contains_mempool() {
        let src = generate_runtime_c_source();
        assert!(src.contains("void axon_mempool_init(void)"));
        assert!(src.contains("void* axon_mempool_alloc(int64_t size)"));
        assert!(src.contains("void axon_mempool_free(void* ptr, int64_t size)"));
        assert!(src.contains("void axon_mempool_clear(void)"));
        assert!(src.contains("void axon_mempool_stats(void)"));
        assert!(src.contains("AxonMemPool"));
        assert!(src.contains("AxonMemBucket"));
    }

    #[test]
    fn generate_c_source_contains_gpu_stubs() {
        let src = generate_runtime_c_source();
        assert!(src.contains("int32_t axon_gpu_init(void)"));
        assert!(src.contains("void axon_gpu_shutdown(void)"));
        assert!(src.contains("int32_t axon_gpu_device_count(void)"));
        assert!(src.contains("void* axon_gpu_alloc(int64_t size)"));
        assert!(src.contains("void axon_gpu_free(void* ptr)"));
        assert!(src.contains("void* axon_gpu_tensor_to_device(void* tensor, int32_t device_id)"));
        assert!(src.contains("void* axon_gpu_tensor_to_host(void* tensor)"));
    }

    #[test]
    fn emit_declarations_contains_threadpool() {
        let ir = emit_runtime_declarations();
        assert!(ir.contains("@axon_threadpool_init(i32)"));
        assert!(ir.contains("@axon_threadpool_shutdown()"));
        assert!(ir.contains("@axon_threadpool_num_threads()"));
    }

    #[test]
    fn emit_declarations_contains_mempool() {
        let ir = emit_runtime_declarations();
        assert!(ir.contains("@axon_mempool_init()"));
        assert!(ir.contains("@axon_mempool_alloc(i64)"));
        assert!(ir.contains("@axon_mempool_free(i8*, i64)"));
        assert!(ir.contains("@axon_mempool_clear()"));
        assert!(ir.contains("@axon_mempool_stats()"));
    }

    #[test]
    fn emit_declarations_contains_gpu_stubs() {
        let ir = emit_runtime_declarations();
        assert!(ir.contains("@axon_gpu_init()"));
        assert!(ir.contains("@axon_gpu_shutdown()"));
        assert!(ir.contains("@axon_gpu_device_count()"));
        assert!(ir.contains("@axon_gpu_alloc(i64)"));
        assert!(ir.contains("@axon_gpu_free(i8*)"));
        assert!(ir.contains("@axon_gpu_tensor_to_device(i8*, i32)"));
        assert!(ir.contains("@axon_gpu_tensor_to_host(i8*)"));
    }

    #[test]
    fn generate_c_source_contains_autograd_ops() {
        let src = generate_runtime_c_source();
        assert!(src.contains("void* axon_ag_sigmoid(void* a)"));
        assert!(src.contains("void* axon_ag_tanh(void* a)"));
        assert!(src.contains("void* axon_ag_log(void* a)"));
        assert!(src.contains("void* axon_ag_exp(void* a)"));
        assert!(src.contains("void* axon_ag_neg(void* a)"));
        assert!(src.contains("void* axon_ag_abs(void* a)"));
        assert!(src.contains("void* axon_ag_sqrt(void* a)"));
        assert!(src.contains("void* axon_ag_sin(void* a)"));
        assert!(src.contains("void* axon_ag_cos(void* a)"));
        assert!(src.contains("void* axon_ag_pow(void* a, void* b)"));
        assert!(src.contains("void* axon_ag_gelu(void* a)"));
        assert!(src.contains("void* axon_ag_silu(void* a)"));
        assert!(src.contains("void* axon_ag_leaky_relu(void* a)"));
    }

    #[test]
    fn generate_c_source_contains_adam_optimizer() {
        let src = generate_runtime_c_source();
        assert!(src.contains("void axon_adam_init(void** params, int32_t n_params)"));
        assert!(src.contains("void axon_adam_step("));
        assert!(src.contains("AxonAdamState"));
    }

    #[test]
    fn generate_c_source_thread_local_tape() {
        let src = generate_runtime_c_source();
        // Verify tape is thread-local (not just static)
        assert!(src.contains("__declspec(thread)") || src.contains("__thread"),
            "tape should use thread-local storage");
    }

    #[test]
    fn generate_c_source_backward_has_all_gradops() {
        let src = generate_runtime_c_source();
        assert!(src.contains("case OP_SIGMOID:"));
        assert!(src.contains("case OP_TANH:"));
        assert!(src.contains("case OP_LOG:"));
        assert!(src.contains("case OP_EXP:"));
        assert!(src.contains("case OP_POW:"));
        assert!(src.contains("case OP_NEG:"));
        assert!(src.contains("case OP_ABS:"));
        assert!(src.contains("case OP_SQRT:"));
        assert!(src.contains("case OP_SIN:"));
        assert!(src.contains("case OP_COS:"));
        assert!(src.contains("case OP_GELU:"));
        assert!(src.contains("case OP_SILU:"));
        assert!(src.contains("case OP_LEAKY_RELU:"));
    }

    #[test]
    fn emit_declarations_contains_autograd_ops() {
        let ir = emit_runtime_declarations();
        assert!(ir.contains("@axon_ag_sigmoid(i8*)"));
        assert!(ir.contains("@axon_ag_tanh(i8*)"));
        assert!(ir.contains("@axon_ag_log(i8*)"));
        assert!(ir.contains("@axon_ag_exp(i8*)"));
        assert!(ir.contains("@axon_ag_pow(i8*, i8*)"));
        assert!(ir.contains("@axon_ag_gelu(i8*)"));
        assert!(ir.contains("@axon_ag_silu(i8*)"));
        assert!(ir.contains("@axon_adam_step("));
    }

    #[test]
    fn emit_declarations_contains_quantization_ops() {
        let ir = emit_runtime_declarations();
        assert!(ir.contains("@axon_tensor_quantize(i8*, i8)"));
        assert!(ir.contains("@axon_tensor_dequantize(i8*)"));
        assert!(ir.contains("@axon_tensor_qmatmul(i8*, i8*)"));
        assert!(ir.contains("@axon_tensor_quantize_symmetric(i8*, i32)"));
        assert!(ir.contains("@axon_tensor_dequantize_scale(i8*, double)"));
        assert!(ir.contains("@axon_tensor_calibrate_range(i8*, double*, double*)"));
        assert!(ir.contains("@axon_tensor_quantize_dynamic(i8*, i32)"));
        assert!(ir.contains("@axon_tensor_mixed_precision_cast(i8*, i8)"));
    }

    #[test]
    fn generate_c_source_has_quantization() {
        let src = generate_runtime_c_source();
        assert!(src.contains("void* axon_tensor_quantize("));
        assert!(src.contains("void* axon_tensor_dequantize("));
        assert!(src.contains("void* axon_tensor_qmatmul("));
        assert!(src.contains("void* axon_tensor_quantize_symmetric("));
        assert!(src.contains("void* axon_tensor_dequantize_scale("));
        assert!(src.contains("void axon_tensor_calibrate_range("));
        assert!(src.contains("void* axon_tensor_quantize_dynamic("));
        assert!(src.contains("void* axon_tensor_mixed_precision_cast("));
    }

    #[test]
    fn generate_c_source_qmatmul_int32_accum() {
        let src = generate_runtime_c_source();
        // Verify qmatmul uses int32 accumulation
        assert!(src.contains("int32_t acc = 0;"), "qmatmul should accumulate in int32");
    }
}
