// stdlib/tensor.rs — Tensor creation, shape ops, reductions, linalg, device.

use crate::symbol::SymbolTable;
use crate::types::*;

use super::{def_fn, def_method, def_struct};

/// Register tensor operations and creation functions.
pub fn register_tensor(interner: &mut TypeInterner, symbols: &mut SymbolTable) {
    let tensor_ty = def_struct(symbols, interner, "Tensor", vec![], vec![]);

    register_creation(interner, symbols, tensor_ty);
    register_shape_ops(interner, symbols, tensor_ty);
    register_reductions(interner, symbols, tensor_ty);
    register_element_math(interner, symbols, tensor_ty);
    register_linalg(interner, symbols, tensor_ty);
    register_device_ops(interner, symbols, tensor_ty);
}

// -- Creation functions -------------------------------------------------------

fn register_creation(interner: &mut TypeInterner, symbols: &mut SymbolTable, tensor_ty: TypeId) {
    // Free functions that return a Tensor
    def_fn(symbols, interner, "zeros", vec![TypeId::INT64], tensor_ty);
    def_fn(symbols, interner, "ones", vec![TypeId::INT64], tensor_ty);
    def_fn(symbols, interner, "full", vec![TypeId::INT64, TypeId::FLOAT64], tensor_ty);
    def_fn(symbols, interner, "rand", vec![TypeId::INT64], tensor_ty);
    def_fn(symbols, interner, "randn", vec![TypeId::INT64], tensor_ty);
    def_fn(symbols, interner, "from_vec", vec![TypeId::UNIT], tensor_ty);
    def_fn(symbols, interner, "arange", vec![TypeId::FLOAT64, TypeId::FLOAT64, TypeId::FLOAT64], tensor_ty);
    def_fn(symbols, interner, "linspace", vec![TypeId::FLOAT64, TypeId::FLOAT64, TypeId::INT64], tensor_ty);
    def_fn(symbols, interner, "eye", vec![TypeId::INT64], tensor_ty);
}

// -- Shape operations ---------------------------------------------------------

fn register_shape_ops(interner: &mut TypeInterner, symbols: &mut SymbolTable, tensor_ty: TypeId) {
    def_method(symbols, interner, "Tensor", "shape", vec![tensor_ty], TypeId::UNIT);
    def_method(symbols, interner, "Tensor", "ndim", vec![tensor_ty], TypeId::INT64);
    def_method(symbols, interner, "Tensor", "numel", vec![tensor_ty], TypeId::INT64);
    def_method(symbols, interner, "Tensor", "reshape", vec![tensor_ty, TypeId::UNIT], tensor_ty);
    def_method(symbols, interner, "Tensor", "transpose", vec![tensor_ty], tensor_ty);
    def_method(
        symbols,
        interner,
        "Tensor",
        "permute",
        vec![tensor_ty, TypeId::UNIT],
        tensor_ty,
    );
    def_method(symbols, interner, "Tensor", "squeeze", vec![tensor_ty], tensor_ty);
    def_method(
        symbols,
        interner,
        "Tensor",
        "unsqueeze",
        vec![tensor_ty, TypeId::INT64],
        tensor_ty,
    );
    def_method(symbols, interner, "Tensor", "flatten", vec![tensor_ty], tensor_ty);
    def_method(symbols, interner, "Tensor", "view", vec![tensor_ty, TypeId::UNIT], tensor_ty);
    def_method(
        symbols,
        interner,
        "Tensor",
        "expand",
        vec![tensor_ty, TypeId::UNIT],
        tensor_ty,
    );
    def_method(symbols, interner, "Tensor", "contiguous", vec![tensor_ty], tensor_ty);
    def_method(symbols, interner, "Tensor", "is_contiguous", vec![tensor_ty], TypeId::BOOL);
    def_method(symbols, interner, "Tensor", "dtype", vec![tensor_ty], TypeId::UNIT);
}

// -- Reductions ---------------------------------------------------------------

fn register_reductions(interner: &mut TypeInterner, symbols: &mut SymbolTable, tensor_ty: TypeId) {
    def_method(symbols, interner, "Tensor", "sum", vec![tensor_ty], tensor_ty);
    def_method(symbols, interner, "Tensor", "mean", vec![tensor_ty], tensor_ty);
    def_method(symbols, interner, "Tensor", "max", vec![tensor_ty], tensor_ty);
    def_method(symbols, interner, "Tensor", "min", vec![tensor_ty], tensor_ty);
    def_method(symbols, interner, "Tensor", "argmax", vec![tensor_ty], tensor_ty);
    def_method(symbols, interner, "Tensor", "argmin", vec![tensor_ty], tensor_ty);
    def_method(symbols, interner, "Tensor", "prod", vec![tensor_ty], tensor_ty);
    def_method(symbols, interner, "Tensor", "var", vec![tensor_ty], tensor_ty);
    def_method(symbols, interner, "Tensor", "std", vec![tensor_ty], tensor_ty);
}

// -- Element-wise math --------------------------------------------------------

fn register_element_math(
    interner: &mut TypeInterner,
    symbols: &mut SymbolTable,
    tensor_ty: TypeId,
) {
    def_method(symbols, interner, "Tensor", "abs", vec![tensor_ty], tensor_ty);
    def_method(symbols, interner, "Tensor", "sqrt", vec![tensor_ty], tensor_ty);
    def_method(symbols, interner, "Tensor", "exp", vec![tensor_ty], tensor_ty);
    def_method(symbols, interner, "Tensor", "log", vec![tensor_ty], tensor_ty);
    def_method(
        symbols,
        interner,
        "Tensor",
        "pow",
        vec![tensor_ty, TypeId::FLOAT64],
        tensor_ty,
    );
    def_method(
        symbols,
        interner,
        "Tensor",
        "clamp",
        vec![tensor_ty, TypeId::FLOAT64, TypeId::FLOAT64],
        tensor_ty,
    );
    def_method(symbols, interner, "Tensor", "relu", vec![tensor_ty], tensor_ty);
    def_method(symbols, interner, "Tensor", "sigmoid", vec![tensor_ty], tensor_ty);
    def_method(symbols, interner, "Tensor", "tanh", vec![tensor_ty], tensor_ty);
    def_method(symbols, interner, "Tensor", "softmax", vec![tensor_ty, TypeId::INT64], tensor_ty);
    def_method(symbols, interner, "Tensor", "sin", vec![tensor_ty], tensor_ty);
    def_method(symbols, interner, "Tensor", "cos", vec![tensor_ty], tensor_ty);
    def_method(symbols, interner, "Tensor", "neg", vec![tensor_ty], tensor_ty);
    def_method(symbols, interner, "Tensor", "reciprocal", vec![tensor_ty], tensor_ty);
}

// -- Linear algebra -----------------------------------------------------------

fn register_linalg(interner: &mut TypeInterner, symbols: &mut SymbolTable, tensor_ty: TypeId) {
    def_method(symbols, interner, "Tensor", "matmul", vec![tensor_ty, tensor_ty], tensor_ty);
    def_method(symbols, interner, "Tensor", "dot", vec![tensor_ty, tensor_ty], tensor_ty);
    def_method(symbols, interner, "Tensor", "outer", vec![tensor_ty, tensor_ty], tensor_ty);
    def_method(symbols, interner, "Tensor", "norm", vec![tensor_ty], TypeId::FLOAT64);
    def_method(symbols, interner, "Tensor", "det", vec![tensor_ty], TypeId::FLOAT64);
    def_method(symbols, interner, "Tensor", "inv", vec![tensor_ty], tensor_ty);
    // SVD returns a tuple of tensors (approximated as Tensor)
    def_method(symbols, interner, "Tensor", "svd", vec![tensor_ty], TypeId::UNIT);
    def_method(symbols, interner, "Tensor", "eig", vec![tensor_ty], TypeId::UNIT);
    def_method(symbols, interner, "Tensor", "solve", vec![tensor_ty, tensor_ty], tensor_ty);
    def_method(symbols, interner, "Tensor", "cholesky", vec![tensor_ty], tensor_ty);
    def_method(symbols, interner, "Tensor", "qr", vec![tensor_ty], TypeId::UNIT);
    def_method(symbols, interner, "Tensor", "cross", vec![tensor_ty, tensor_ty], tensor_ty);
    def_method(symbols, interner, "Tensor", "trace", vec![tensor_ty], TypeId::FLOAT64);
}

// -- Device operations --------------------------------------------------------

fn register_device_ops(
    interner: &mut TypeInterner,
    symbols: &mut SymbolTable,
    tensor_ty: TypeId,
) {
    def_method(symbols, interner, "Tensor", "to_cpu", vec![tensor_ty], tensor_ty);
    def_method(symbols, interner, "Tensor", "to_gpu", vec![tensor_ty], tensor_ty);
    def_method(symbols, interner, "Tensor", "device", vec![tensor_ty], TypeId::UNIT);
    def_method(symbols, interner, "Tensor", "to_device", vec![tensor_ty, TypeId::UNIT], tensor_ty);
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
    fn tensor_struct_registered() {
        let (mut i, mut s) = fresh();
        register_tensor(&mut i, &mut s);
        assert!(s.lookup("Tensor").is_some());
    }

    #[test]
    fn tensor_creation_functions() {
        let (mut i, mut s) = fresh();
        register_tensor(&mut i, &mut s);
        for name in &["zeros", "ones", "full", "rand", "randn", "eye", "arange", "linspace"] {
            assert!(s.lookup(name).is_some(), "{} should be registered", name);
        }
    }

    #[test]
    fn tensor_shape_methods() {
        let (mut i, mut s) = fresh();
        register_tensor(&mut i, &mut s);
        for method in &["Tensor::shape", "Tensor::reshape", "Tensor::transpose", "Tensor::flatten"] {
            assert!(s.lookup(method).is_some(), "{} should be registered", method);
        }
    }

    #[test]
    fn tensor_reduction_methods() {
        let (mut i, mut s) = fresh();
        register_tensor(&mut i, &mut s);
        for method in &["Tensor::sum", "Tensor::mean", "Tensor::max", "Tensor::argmax"] {
            assert!(s.lookup(method).is_some(), "{} should be registered", method);
        }
    }

    #[test]
    fn tensor_linalg_methods() {
        let (mut i, mut s) = fresh();
        register_tensor(&mut i, &mut s);
        for method in &["Tensor::matmul", "Tensor::dot", "Tensor::inv", "Tensor::svd"] {
            assert!(s.lookup(method).is_some(), "{} should be registered", method);
        }
    }

    #[test]
    fn tensor_device_methods() {
        let (mut i, mut s) = fresh();
        register_tensor(&mut i, &mut s);
        assert!(s.lookup("Tensor::to_cpu").is_some());
        assert!(s.lookup("Tensor::to_gpu").is_some());
    }
}
