// stdlib/export.rs — Model export and serialization.

use crate::symbol::SymbolTable;
use crate::types::*;

use super::def_fn;

/// Register model export / serialization functions.
pub fn register_export(interner: &mut TypeInterner, symbols: &mut SymbolTable) {
    def_fn(symbols, interner, "onnx_export", vec![TypeId::STRING, TypeId::INT32], TypeId::BOOL);
    def_fn(symbols, interner, "save_model", vec![TypeId::STRING], TypeId::BOOL);
    def_fn(symbols, interner, "load_model", vec![TypeId::STRING], TypeId::INT64);
    def_fn(symbols, interner, "save_tensor", vec![TypeId::STRING], TypeId::BOOL);
    def_fn(symbols, interner, "load_tensor", vec![TypeId::STRING], TypeId::INT64);
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
    fn export_fns_registered() {
        let (mut i, mut s) = fresh();
        register_export(&mut i, &mut s);
        for name in &["onnx_export", "save_model", "load_model", "save_tensor", "load_tensor"] {
            assert!(s.lookup(name).is_some(), "{} should be registered", name);
        }
    }

    #[test]
    fn onnx_export_signature() {
        let (mut i, mut s) = fresh();
        register_export(&mut i, &mut s);
        let sym_id = s.lookup("onnx_export").unwrap();
        let sym = s.get_symbol(sym_id);
        match i.resolve(sym.ty) {
            Type::Function { params, ret } => {
                assert_eq!(params.len(), 2);
                assert_eq!(params[0], TypeId::STRING);
                assert_eq!(params[1], TypeId::INT32);
                assert_eq!(*ret, TypeId::BOOL);
            }
            _ => panic!("onnx_export should be a function"),
        }
    }

    #[test]
    fn save_load_model_registered() {
        let (mut i, mut s) = fresh();
        register_export(&mut i, &mut s);
        assert!(s.lookup("save_model").is_some());
        assert!(s.lookup("load_model").is_some());
    }
}
