// stdlib/autograd.rs — Automatic differentiation primitives.

use crate::symbol::SymbolTable;
use crate::types::*;

use super::{def_enum, def_fn, def_struct};

/// Register autograd types and functions.
pub fn register_autograd(interner: &mut TypeInterner, symbols: &mut SymbolTable) {
    register_types(interner, symbols);
    register_functions(interner, symbols);
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
            ("Add".into(), EnumVariantType::Unit),
            ("Sub".into(), EnumVariantType::Unit),
            ("Mul".into(), EnumVariantType::Unit),
            ("Div".into(), EnumVariantType::Unit),
            ("MatMul".into(), EnumVariantType::Unit),
            ("Relu".into(), EnumVariantType::Unit),
            ("Sigmoid".into(), EnumVariantType::Unit),
            ("Tanh".into(), EnumVariantType::Unit),
            ("Softmax".into(), EnumVariantType::Unit),
            ("Sum".into(), EnumVariantType::Unit),
            ("Mean".into(), EnumVariantType::Unit),
            ("Reshape".into(), EnumVariantType::Unit),
            ("Transpose".into(), EnumVariantType::Unit),
            ("Conv2d".into(), EnumVariantType::Unit),
            ("MaxPool2d".into(), EnumVariantType::Unit),
            ("BatchNorm".into(), EnumVariantType::Unit),
            ("Linear".into(), EnumVariantType::Unit),
            ("Dropout".into(), EnumVariantType::Unit),
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
        for name in &["GradTensor", "ComputationGraph", "GraphNode"] {
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
            }
            _ => panic!("GradOp should be an enum"),
        }
    }

    #[test]
    fn functions_registered() {
        let (mut i, mut s) = fresh();
        register_autograd(&mut i, &mut s);
        for name in &["grad_tensor_new", "backward", "zero_grad", "detach", "no_grad", "grad_checkpoint"] {
            assert!(s.lookup(name).is_some(), "{} should be registered", name);
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
}
