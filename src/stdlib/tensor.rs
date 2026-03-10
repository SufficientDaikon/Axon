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
    register_comparison_and_conversion(interner, symbols, tensor_ty);
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

// -- Comparison & scalar extraction ------------------------------------------

fn register_comparison_and_conversion(
    interner: &mut TypeInterner,
    symbols: &mut SymbolTable,
    tensor_ty: TypeId,
) {
    // item(self) -> Float64  — extract scalar from single-element tensor
    def_method(symbols, interner, "Tensor", "item", vec![tensor_ty], TypeId::FLOAT64);

    // to_vec(self) -> Vec<Float64>  — convert tensor to flat vector
    let vec_ty = TypeId::UNIT; // Vec<Float64> approximated as UNIT (consistent with stdlib)
    def_method(symbols, interner, "Tensor", "to_vec", vec![tensor_ty], vec_ty);

    // eq(self, other: Tensor) -> Tensor  — element-wise equality
    def_method(symbols, interner, "Tensor", "eq", vec![tensor_ty, tensor_ty], tensor_ty);

    // ne(self, other: Tensor) -> Tensor  — element-wise inequality
    def_method(symbols, interner, "Tensor", "ne", vec![tensor_ty, tensor_ty], tensor_ty);

    // lt(self, other: Tensor) -> Tensor  — element-wise less-than
    def_method(symbols, interner, "Tensor", "lt", vec![tensor_ty, tensor_ty], tensor_ty);

    // le(self, other: Tensor) -> Tensor  — element-wise less-or-equal
    def_method(symbols, interner, "Tensor", "le", vec![tensor_ty, tensor_ty], tensor_ty);

    // gt(self, other: Tensor) -> Tensor  — element-wise greater-than
    def_method(symbols, interner, "Tensor", "gt", vec![tensor_ty, tensor_ty], tensor_ty);

    // ge(self, other: Tensor) -> Tensor  — element-wise greater-or-equal
    def_method(symbols, interner, "Tensor", "ge", vec![tensor_ty, tensor_ty], tensor_ty);

    // sum_dim(self, dim: Int64) -> Tensor  — sum along a specific dimension
    def_method(symbols, interner, "Tensor", "sum_dim", vec![tensor_ty, TypeId::INT64], tensor_ty);

    // mean_dim(self, dim: Int64) -> Tensor  — mean along a specific dimension
    def_method(symbols, interner, "Tensor", "mean_dim", vec![tensor_ty, TypeId::INT64], tensor_ty);

    // Slicing operations
    def_method(symbols, interner, "Tensor", "slice", vec![tensor_ty, TypeId::INT64, TypeId::INT64, TypeId::INT64], tensor_ty);
    def_method(symbols, interner, "Tensor", "narrow", vec![tensor_ty, TypeId::INT64, TypeId::INT64, TypeId::INT64], tensor_ty);
    def_method(symbols, interner, "Tensor", "select", vec![tensor_ty, TypeId::INT64, TypeId::INT64], tensor_ty);
    def_method(symbols, interner, "Tensor", "index_select", vec![tensor_ty, TypeId::INT64, tensor_ty], tensor_ty);
    def_method(symbols, interner, "Tensor", "masked_select", vec![tensor_ty, tensor_ty], tensor_ty);
    def_method(symbols, interner, "Tensor", "where_cond", vec![tensor_ty, tensor_ty, tensor_ty], tensor_ty);

    // Concatenation / stacking
    def_fn(symbols, interner, "cat", vec![TypeId::UNIT, TypeId::INT64], tensor_ty);
    def_fn(symbols, interner, "stack", vec![TypeId::UNIT, TypeId::INT64], tensor_ty);

    // Type casting
    def_method(symbols, interner, "Tensor", "to_float32", vec![tensor_ty], tensor_ty);
    def_method(symbols, interner, "Tensor", "to_float64", vec![tensor_ty], tensor_ty);
    def_method(symbols, interner, "Tensor", "to_int64", vec![tensor_ty], tensor_ty);
    def_method(symbols, interner, "Tensor", "to_int32", vec![tensor_ty], tensor_ty);

    // Copying
    def_method(symbols, interner, "Tensor", "clone", vec![tensor_ty], tensor_ty);
    def_method(symbols, interner, "Tensor", "copy_from", vec![tensor_ty, tensor_ty], TypeId::UNIT);
    def_method(symbols, interner, "Tensor", "fill", vec![tensor_ty, TypeId::FLOAT64], TypeId::UNIT);
    def_method(symbols, interner, "Tensor", "zero_", vec![tensor_ty], TypeId::UNIT);
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

    #[test]
    fn tensor_comparison_and_conversion_methods() {
        let (mut i, mut s) = fresh();
        register_tensor(&mut i, &mut s);
        for method in &[
            "Tensor::item",
            "Tensor::to_vec",
            "Tensor::eq",
            "Tensor::ne",
            "Tensor::lt",
            "Tensor::le",
            "Tensor::gt",
            "Tensor::ge",
            "Tensor::sum_dim",
            "Tensor::mean_dim",
            "Tensor::slice",
            "Tensor::narrow",
            "Tensor::select",
            "Tensor::index_select",
            "Tensor::masked_select",
            "Tensor::where_cond",
            "Tensor::to_float32",
            "Tensor::to_float64",
            "Tensor::clone",
            "Tensor::fill",
            "Tensor::zero_",
        ] {
            assert!(s.lookup(method).is_some(), "{} should be registered", method);
        }
    }

    #[test]
    fn tensor_concat_stack_registered() {
        let (mut i, mut s) = fresh();
        register_tensor(&mut i, &mut s);
        assert!(s.lookup("cat").is_some());
        assert!(s.lookup("stack").is_some());
    }
}
