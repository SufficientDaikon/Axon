// stdlib/autograd.rs — Automatic differentiation primitives.
//
// Gradient rules for each GradOp variant:
//   Add:       d/dx(a + b) = 1, d/dy(a + b) = 1
//   Sub:       d/dx(a - b) = 1, d/dy(a - b) = -1
//   Mul:       d/dx(a * b) = b, d/dy(a * b) = a
//   Div:       d/dx(a / b) = 1/b, d/dy(a / b) = -a/b²
//   MatMul:    d/dA(A @ B) = grad @ B^T, d/dB(A @ B) = A^T @ grad
//   Relu:      d/dx(relu(x)) = grad * (x > 0)
//   Sigmoid:   d/dx(σ(x)) = σ(x) * (1 - σ(x)) * grad
//   Tanh:      d/dx(tanh(x)) = (1 - tanh²(x)) * grad
//   Softmax:   d/dx(softmax(x))_i = softmax(x)_i * (δ_ij - softmax(x)_j) * grad
//   Sum:       d/dx(sum(x)) = ones_like(x) * grad
//   Mean:      d/dx(mean(x)) = ones_like(x) / numel(x) * grad
//   Reshape:   d/dx(reshape(x)) = reshape(grad, x.shape)
//   Transpose: d/dx(transpose(x)) = transpose(grad)
//   Conv2d:    d/dx(conv2d(x, w)) = conv2d_backward(grad, w), d/dw = conv2d_weight_grad(x, grad)
//   MaxPool2d: d/dx(maxpool2d(x)) = grad routed to max indices
//   BatchNorm: d/dx(batchnorm(x)) = batchnorm_backward(grad, running_mean, running_var)
//   Linear:    d/dx(Wx + b) = grad @ W^T, d/dW = x^T @ grad, d/db = sum(grad, dim=0)
//   Dropout:   d/dx(dropout(x)) = grad * mask / (1 - p)
//   Embedding: d/dx(embed(idx)) = scatter_add(grad, idx)
//   CrossEntropy: d/dx(CE(x, y)) = softmax(x) - one_hot(y)
//   Log:       d/dx(log(x)) = grad / x
//   Exp:       d/dx(exp(x)) = exp(x) * grad
//   Pow:       d/dx(x^n) = n * x^(n-1) * grad
//   Neg:       d/dx(-x) = -grad
//   Abs:       d/dx(|x|) = sign(x) * grad

use crate::symbol::SymbolTable;
use crate::types::*;

use super::{def_enum, def_fn, def_method, def_struct, def_trait};

/// Register autograd types and functions.
pub fn register_autograd(interner: &mut TypeInterner, symbols: &mut SymbolTable) {
    register_types(interner, symbols);
    register_functions(interner, symbols);
    register_tensor_autograd_methods(interner, symbols);
    register_autograd_context(interner, symbols);
}

// -- Types --------------------------------------------------------------------

fn register_types(interner: &mut TypeInterner, symbols: &mut SymbolTable) {
    def_struct(
        symbols, interner, "GradTensor",
        vec![("data".into(), TypeId::INT64), ("requires_grad".into(), TypeId::BOOL)],
        vec![],
    );
    def_struct(symbols, interner, "ComputationGraph", vec![], vec![]);
    def_struct(
        symbols, interner, "GraphNode",
        vec![("id".into(), TypeId::INT64), ("op".into(), TypeId::INT64)],
        vec![],
    );

    def_enum(
        symbols,
        interner,
        "GradOp",
        vec![
            // d/dx(a + b) = 1, d/dy(a + b) = 1
            ("Add".into(), EnumVariantType::Unit),
            // d/dx(a - b) = 1, d/dy(a - b) = -1
            ("Sub".into(), EnumVariantType::Unit),
            // d/dx(a * b) = b, d/dy(a * b) = a
            ("Mul".into(), EnumVariantType::Unit),
            // d/dx(a / b) = 1/b, d/dy(a / b) = -a/b²
            ("Div".into(), EnumVariantType::Unit),
            // d/dA(A @ B) = grad @ B^T, d/dB = A^T @ grad
            ("MatMul".into(), EnumVariantType::Unit),
            // d/dx(relu(x)) = grad * (x > 0)
            ("Relu".into(), EnumVariantType::Unit),
            // d/dx(σ(x)) = σ(x) * (1 - σ(x)) * grad
            ("Sigmoid".into(), EnumVariantType::Unit),
            // d/dx(tanh(x)) = (1 - tanh²(x)) * grad
            ("Tanh".into(), EnumVariantType::Unit),
            // d/dx(softmax(x))_i = softmax(x)_i * (δ_ij - softmax(x)_j) * grad
            ("Softmax".into(), EnumVariantType::Unit),
            // d/dx(sum(x)) = ones_like(x) * grad
            ("Sum".into(), EnumVariantType::Unit),
            // d/dx(mean(x)) = ones_like(x) / numel(x) * grad
            ("Mean".into(), EnumVariantType::Unit),
            // d/dx(reshape(x)) = reshape(grad, x.shape)
            ("Reshape".into(), EnumVariantType::Unit),
            // d/dx(transpose(x)) = transpose(grad)
            ("Transpose".into(), EnumVariantType::Unit),
            // d/dx(conv2d(x, w)) = conv2d_backward(grad, w)
            ("Conv2d".into(), EnumVariantType::Unit),
            // d/dx(maxpool2d(x)) = grad routed to max indices
            ("MaxPool2d".into(), EnumVariantType::Unit),
            // d/dx(batchnorm(x)) = batchnorm_backward(grad, running_mean, running_var)
            ("BatchNorm".into(), EnumVariantType::Unit),
            // d/dx(Wx + b) = grad @ W^T, d/dW = x^T @ grad, d/db = sum(grad, dim=0)
            ("Linear".into(), EnumVariantType::Unit),
            // d/dx(dropout(x)) = grad * mask / (1 - p)
            ("Dropout".into(), EnumVariantType::Unit),
            // d/dx(embed(idx)) = scatter_add(grad, idx)
            ("Embedding".into(), EnumVariantType::Unit),
            // d/dx(CE(x, y)) = softmax(x) - one_hot(y)
            ("CrossEntropy".into(), EnumVariantType::Unit),
            // d/dx(log(x)) = grad / x
            ("Log".into(), EnumVariantType::Unit),
            // d/dx(exp(x)) = exp(x) * grad
            ("Exp".into(), EnumVariantType::Unit),
            // d/dx(x^n) = n * x^(n-1) * grad
            ("Pow".into(), EnumVariantType::Unit),
            // d/dx(-x) = -grad
            ("Neg".into(), EnumVariantType::Unit),
            // d/dx(|x|) = sign(x) * grad
            ("Abs".into(), EnumVariantType::Unit),
        ],
        vec![],
    );
}

// -- Functions ----------------------------------------------------------------

fn register_functions(interner: &mut TypeInterner, symbols: &mut SymbolTable) {
    def_fn(symbols, interner, "grad_tensor_new", vec![TypeId::BOOL], TypeId::INT64);
    def_fn(symbols, interner, "backward", vec![], TypeId::UNIT);
    def_fn(symbols, interner, "zero_grad", vec![], TypeId::UNIT);
    def_fn(symbols, interner, "detach", vec![TypeId::INT64], TypeId::INT64);
    def_fn(symbols, interner, "no_grad", vec![], TypeId::UNIT);
    def_fn(symbols, interner, "grad_checkpoint", vec![TypeId::INT64], TypeId::INT64);
    def_fn(symbols, interner, "enable_grad", vec![], TypeId::UNIT);

    // ── Gradient rule functions ──────────────────────────────────
    // Each function computes the gradient for a specific operation.
    // Tensor values are represented as Int64 (stdlib convention).

    // Arithmetic gradient rules
    def_fn(symbols, interner, "grad_add", vec![TypeId::INT64], TypeId::INT64);
    def_fn(symbols, interner, "grad_sub", vec![TypeId::INT64], TypeId::INT64);
    def_fn(symbols, interner, "grad_mul", vec![TypeId::INT64, TypeId::INT64, TypeId::INT64], TypeId::INT64);
    def_fn(symbols, interner, "grad_div", vec![TypeId::INT64, TypeId::INT64, TypeId::INT64], TypeId::INT64);
    def_fn(symbols, interner, "grad_neg", vec![TypeId::INT64], TypeId::INT64);
    def_fn(symbols, interner, "grad_abs", vec![TypeId::INT64, TypeId::INT64], TypeId::INT64);

    // Matrix gradient rules
    def_fn(symbols, interner, "grad_matmul", vec![TypeId::INT64, TypeId::INT64, TypeId::INT64], TypeId::INT64);
    def_fn(symbols, interner, "grad_transpose", vec![TypeId::INT64], TypeId::INT64);
    def_fn(symbols, interner, "grad_reshape", vec![TypeId::INT64, TypeId::INT64], TypeId::INT64);

    // Activation gradient rules
    def_fn(symbols, interner, "grad_relu", vec![TypeId::INT64, TypeId::INT64], TypeId::INT64);
    def_fn(symbols, interner, "grad_sigmoid", vec![TypeId::INT64, TypeId::INT64], TypeId::INT64);
    def_fn(symbols, interner, "grad_tanh", vec![TypeId::INT64, TypeId::INT64], TypeId::INT64);
    def_fn(symbols, interner, "grad_softmax", vec![TypeId::INT64, TypeId::INT64], TypeId::INT64);
    def_fn(symbols, interner, "grad_gelu", vec![TypeId::INT64, TypeId::INT64], TypeId::INT64);
    def_fn(symbols, interner, "grad_silu", vec![TypeId::INT64, TypeId::INT64], TypeId::INT64);
    def_fn(symbols, interner, "grad_leaky_relu", vec![TypeId::INT64, TypeId::FLOAT64, TypeId::INT64], TypeId::INT64);

    // Reduction gradient rules
    def_fn(symbols, interner, "grad_sum", vec![TypeId::INT64, TypeId::INT64], TypeId::INT64);
    def_fn(symbols, interner, "grad_mean", vec![TypeId::INT64, TypeId::INT64, TypeId::INT64], TypeId::INT64);

    // Transcendental gradient rules
    def_fn(symbols, interner, "grad_log", vec![TypeId::INT64, TypeId::INT64], TypeId::INT64);
    def_fn(symbols, interner, "grad_exp", vec![TypeId::INT64, TypeId::INT64], TypeId::INT64);
    def_fn(symbols, interner, "grad_pow", vec![TypeId::INT64, TypeId::FLOAT64, TypeId::INT64], TypeId::INT64);
    def_fn(symbols, interner, "grad_sqrt", vec![TypeId::INT64, TypeId::INT64], TypeId::INT64);
    def_fn(symbols, interner, "grad_sin", vec![TypeId::INT64, TypeId::INT64], TypeId::INT64);
    def_fn(symbols, interner, "grad_cos", vec![TypeId::INT64, TypeId::INT64], TypeId::INT64);

    // Layer gradient rules
    def_fn(symbols, interner, "grad_linear", vec![TypeId::INT64, TypeId::INT64, TypeId::INT64], TypeId::INT64);
    def_fn(symbols, interner, "grad_conv2d", vec![TypeId::INT64, TypeId::INT64, TypeId::INT64], TypeId::INT64);
    def_fn(symbols, interner, "grad_batchnorm", vec![TypeId::INT64, TypeId::INT64, TypeId::INT64, TypeId::INT64], TypeId::INT64);
    def_fn(symbols, interner, "grad_layernorm", vec![TypeId::INT64, TypeId::INT64, TypeId::INT64, TypeId::INT64], TypeId::INT64);
    def_fn(symbols, interner, "grad_maxpool2d", vec![TypeId::INT64, TypeId::INT64], TypeId::INT64);
    def_fn(symbols, interner, "grad_dropout", vec![TypeId::INT64, TypeId::FLOAT64, TypeId::INT64], TypeId::INT64);
    def_fn(symbols, interner, "grad_embedding", vec![TypeId::INT64, TypeId::INT64, TypeId::INT64], TypeId::INT64);
    def_fn(symbols, interner, "grad_cross_entropy", vec![TypeId::INT64, TypeId::INT64, TypeId::INT64], TypeId::INT64);
}

// -- Tensor autograd methods --------------------------------------------------
// Register backward(), requires_grad(), grad(), zero_grad(), no_grad(), detach()
// as methods on the Tensor type so they can be called as `tensor.backward()`.
// NOTE: The Tensor struct is registered in tensor.rs; we add methods to it here.
// These use TypeId::INT64 as the Tensor proxy type (consistent with the rest of
// the stdlib where Tensor values are represented as Int64).

fn register_tensor_autograd_methods(interner: &mut TypeInterner, symbols: &mut SymbolTable) {
    // Tensor::backward(self) -> Unit — run backpropagation from this tensor
    def_method(symbols, interner, "Tensor", "backward", vec![TypeId::INT64], TypeId::UNIT);

    // Tensor::requires_grad(self) -> Bool — check if gradient tracking is enabled
    def_method(symbols, interner, "Tensor", "requires_grad", vec![TypeId::INT64], TypeId::BOOL);

    // Tensor::grad(self) -> Tensor — access the accumulated gradient
    def_method(symbols, interner, "Tensor", "grad", vec![TypeId::INT64], TypeId::INT64);

    // Tensor::zero_grad(self) -> Unit — zero out accumulated gradients
    def_method(symbols, interner, "Tensor", "zero_grad", vec![TypeId::INT64], TypeId::UNIT);

    // Tensor::no_grad(self) -> Tensor — return detached copy with grad disabled
    def_method(symbols, interner, "Tensor", "no_grad", vec![TypeId::INT64], TypeId::INT64);

    // Tensor::detach(self) -> Tensor — detach from computation graph
    def_method(symbols, interner, "Tensor", "detach", vec![TypeId::INT64], TypeId::INT64);

    // Tensor::set_requires_grad(self, requires: Bool) -> Tensor — enable/disable grad tracking
    def_method(symbols, interner, "Tensor", "set_requires_grad", vec![TypeId::INT64, TypeId::BOOL], TypeId::INT64);

    // Tensor::retain_grad(self) -> Unit — retain gradient for non-leaf tensors
    def_method(symbols, interner, "Tensor", "retain_grad", vec![TypeId::INT64], TypeId::UNIT);

    // Tensor::grad_fn(self) -> Int64 — return the gradient function (GradOp) that created this tensor
    def_method(symbols, interner, "Tensor", "grad_fn", vec![TypeId::INT64], TypeId::INT64);

    // Tensor::is_leaf(self) -> Bool — check if this is a leaf tensor
    def_method(symbols, interner, "Tensor", "is_leaf", vec![TypeId::INT64], TypeId::BOOL);
}

// -- AutogradContext (GradientTape) -------------------------------------------
// Provides an explicit tape-based gradient recording context.
//   let ctx = autograd_context_new();
//   ctx.record(op, inputs, output);
//   ctx.backward(loss);
//   let grads = ctx.gradients(param);

fn register_autograd_context(interner: &mut TypeInterner, symbols: &mut SymbolTable) {
    // AutogradContext struct
    def_struct(symbols, interner, "AutogradContext", vec![], vec![]);

    // AutogradContext trait methods
    let record_ty = interner.intern(Type::Function {
        params: vec![TypeId::INT64, TypeId::INT64, TypeId::INT64], // op, inputs, output
        ret: TypeId::UNIT,
    });
    let backward_ty = interner.intern(Type::Function {
        params: vec![TypeId::INT64], // loss tensor
        ret: TypeId::UNIT,
    });
    let gradients_ty = interner.intern(Type::Function {
        params: vec![TypeId::INT64], // parameter tensor
        ret: TypeId::INT64,          // gradient tensor
    });
    let clear_ty = interner.intern(Type::Function {
        params: vec![],
        ret: TypeId::UNIT,
    });

    def_trait(
        symbols,
        interner,
        "GradientTape",
        vec![
            ("record".into(), record_ty),
            ("backward".into(), backward_ty),
            ("gradients".into(), gradients_ty),
            ("clear".into(), clear_ty),
        ],
        vec![],
    );

    // Constructor
    def_fn(symbols, interner, "autograd_context_new", vec![], TypeId::INT64);

    // Methods on AutogradContext
    // AutogradContext::record(self, op: Int64, inputs: Int64, output: Int64) -> Unit
    def_method(symbols, interner, "AutogradContext", "record",
        vec![TypeId::INT64, TypeId::INT64, TypeId::INT64, TypeId::INT64], TypeId::UNIT);

    // AutogradContext::backward(self, loss: Int64) -> Unit
    def_method(symbols, interner, "AutogradContext", "backward",
        vec![TypeId::INT64, TypeId::INT64], TypeId::UNIT);

    // AutogradContext::gradients(self, param: Int64) -> Int64 (Tensor)
    def_method(symbols, interner, "AutogradContext", "gradients",
        vec![TypeId::INT64, TypeId::INT64], TypeId::INT64);

    // AutogradContext::clear(self) -> Unit
    def_method(symbols, interner, "AutogradContext", "clear",
        vec![TypeId::INT64], TypeId::UNIT);
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::symbol::SymbolTable;

    fn fresh() -> (TypeInterner, SymbolTable) {
        (TypeInterner::new(), SymbolTable::new())
    }

    #[test]
    fn structs_registered() {
        let (mut i, mut s) = fresh();
        register_autograd(&mut i, &mut s);
        for name in &["GradTensor", "ComputationGraph", "GraphNode", "AutogradContext"] {
            assert!(s.lookup(name).is_some(), "{} should be registered", name);
        }
    }

    #[test]
    fn grad_op_enum_registered() {
        let (mut i, mut s) = fresh();
        register_autograd(&mut i, &mut s);
        let sym_id = s.lookup("GradOp").unwrap();
        let sym = s.get_symbol(sym_id);
        match i.resolve(sym.ty) {
            Type::Enum { variants, .. } => {
                let names: Vec<&str> = variants.iter().map(|(n, _)| n.as_str()).collect();
                assert!(names.contains(&"Add"));
                assert!(names.contains(&"MatMul"));
                assert!(names.contains(&"Dropout"));
                // New variants
                assert!(names.contains(&"Embedding"), "Embedding variant should exist");
                assert!(names.contains(&"CrossEntropy"), "CrossEntropy variant should exist");
                assert!(names.contains(&"Log"), "Log variant should exist");
                assert!(names.contains(&"Exp"), "Exp variant should exist");
            }
            _ => panic!("GradOp should be an enum"),
        }
    }

    #[test]
    fn functions_registered() {
        let (mut i, mut s) = fresh();
        register_autograd(&mut i, &mut s);
        for name in &["grad_tensor_new", "backward", "zero_grad", "detach", "no_grad",
                       "grad_checkpoint", "autograd_context_new", "enable_grad"] {
            assert!(s.lookup(name).is_some(), "{} should be registered", name);
        }
    }

    #[test]
    fn gradient_rule_functions_registered() {
        let (mut i, mut s) = fresh();
        register_autograd(&mut i, &mut s);
        let grad_fns = [
            // Arithmetic
            "grad_add", "grad_sub", "grad_mul", "grad_div", "grad_neg", "grad_abs",
            // Matrix
            "grad_matmul", "grad_transpose", "grad_reshape",
            // Activation
            "grad_relu", "grad_sigmoid", "grad_tanh", "grad_softmax",
            "grad_gelu", "grad_silu", "grad_leaky_relu",
            // Reduction
            "grad_sum", "grad_mean",
            // Transcendental
            "grad_log", "grad_exp", "grad_pow", "grad_sqrt", "grad_sin", "grad_cos",
            // Layer
            "grad_linear", "grad_conv2d", "grad_batchnorm", "grad_layernorm",
            "grad_maxpool2d", "grad_dropout", "grad_embedding", "grad_cross_entropy",
        ];
        assert!(grad_fns.len() >= 25, "Must have at least 25 gradient rule functions, got {}", grad_fns.len());
        for name in &grad_fns {
            assert!(s.lookup(name).is_some(), "gradient function {} should be registered", name);
        }
    }

    #[test]
    fn grad_tensor_new_signature() {
        let (mut i, mut s) = fresh();
        register_autograd(&mut i, &mut s);
        let sym_id = s.lookup("grad_tensor_new").unwrap();
        let sym = s.get_symbol(sym_id);
        match i.resolve(sym.ty) {
            Type::Function { params, ret } => {
                assert_eq!(params.len(), 1);
                assert_eq!(params[0], TypeId::BOOL);
                assert_eq!(*ret, TypeId::INT64);
            }
            _ => panic!("grad_tensor_new should be a function"),
        }
    }

    #[test]
    fn tensor_autograd_methods_registered() {
        let (mut i, mut s) = fresh();
        register_autograd(&mut i, &mut s);
        for method in &[
            "Tensor::backward", "Tensor::requires_grad", "Tensor::grad",
            "Tensor::zero_grad", "Tensor::no_grad", "Tensor::detach",
            "Tensor::set_requires_grad", "Tensor::retain_grad",
            "Tensor::grad_fn", "Tensor::is_leaf",
        ] {
            assert!(s.lookup(method).is_some(), "{} should be registered", method);
        }
    }

    #[test]
    fn autograd_context_methods_registered() {
        let (mut i, mut s) = fresh();
        register_autograd(&mut i, &mut s);
        for method in &[
            "AutogradContext::record", "AutogradContext::backward",
            "AutogradContext::gradients", "AutogradContext::clear",
        ] {
            assert!(s.lookup(method).is_some(), "{} should be registered", method);
        }
    }

    #[test]
    fn gradient_tape_trait_registered() {
        let (mut i, mut s) = fresh();
        register_autograd(&mut i, &mut s);
        let sym_id = s.lookup("GradientTape").unwrap();
        let sym = s.get_symbol(sym_id);
        match i.resolve(sym.ty) {
            Type::Trait { methods, .. } => {
                let names: Vec<&str> = methods.iter().map(|(n, _)| n.as_str()).collect();
                assert!(names.contains(&"record"), "GradientTape should have record");
                assert!(names.contains(&"backward"), "GradientTape should have backward");
                assert!(names.contains(&"gradients"), "GradientTape should have gradients");
                assert!(names.contains(&"clear"), "GradientTape should have clear");
            }
            _ => panic!("GradientTape should be a trait"),
        }
    }
}
