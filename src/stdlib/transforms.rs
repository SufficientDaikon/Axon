// stdlib/transforms.rs — Data transforms for vision, text, and preprocessing.

use crate::symbol::SymbolTable;
use crate::types::*;

use super::{def_fn, def_struct, def_trait};

/// Register data transform types and functions.
pub fn register_transforms(interner: &mut TypeInterner, symbols: &mut SymbolTable) {
    register_trait(interner, symbols);
    register_structs(interner, symbols);
    register_functions(interner, symbols);
}

// -- Transform trait ----------------------------------------------------------

fn register_trait(interner: &mut TypeInterner, symbols: &mut SymbolTable) {
    let apply_ty = interner.intern(Type::Function {
        params: vec![TypeId::INT64],
        ret: TypeId::INT64,
    });
    def_trait(
        symbols,
        interner,
        "Transform",
        vec![("apply".into(), apply_ty)],
        vec![],
    );
}

// -- Structs ------------------------------------------------------------------

fn register_structs(interner: &mut TypeInterner, symbols: &mut SymbolTable) {
    def_struct(symbols, interner, "Compose", vec![], vec![]);
    def_struct(
        symbols, interner, "Resize",
        vec![("width".into(), TypeId::INT64), ("height".into(), TypeId::INT64)],
        vec![],
    );
    def_struct(
        symbols, interner, "CenterCrop",
        vec![("width".into(), TypeId::INT64), ("height".into(), TypeId::INT64)],
        vec![],
    );
    def_struct(
        symbols, interner, "RandomCrop",
        vec![("width".into(), TypeId::INT64), ("height".into(), TypeId::INT64)],
        vec![],
    );
    def_struct(
        symbols, interner, "RandomHorizontalFlip",
        vec![("p".into(), TypeId::FLOAT32)],
        vec![],
    );
    def_struct(
        symbols, interner, "Normalize",
        vec![("mean".into(), TypeId::FLOAT64), ("std".into(), TypeId::FLOAT64)],
        vec![],
    );
    def_struct(symbols, interner, "ToTensor", vec![], vec![]);
    def_struct(symbols, interner, "Tokenizer", vec![], vec![]);
    def_struct(
        symbols, interner, "PadSequence",
        vec![("max_len".into(), TypeId::INT64)],
        vec![],
    );
}

// -- Functions ----------------------------------------------------------------

fn register_functions(interner: &mut TypeInterner, symbols: &mut SymbolTable) {
    def_fn(symbols, interner, "resize", vec![TypeId::INT64, TypeId::INT64], TypeId::INT64);
    def_fn(symbols, interner, "center_crop", vec![TypeId::INT64, TypeId::INT64], TypeId::INT64);
    def_fn(symbols, interner, "random_crop", vec![TypeId::INT64, TypeId::INT64], TypeId::INT64);
    def_fn(symbols, interner, "random_horizontal_flip", vec![TypeId::FLOAT32], TypeId::INT64);
    def_fn(symbols, interner, "normalize_transform", vec![TypeId::FLOAT64, TypeId::FLOAT64], TypeId::INT64);
    def_fn(symbols, interner, "to_tensor_transform", vec![], TypeId::INT64);
    def_fn(symbols, interner, "tokenize", vec![TypeId::STRING], TypeId::INT64);
    def_fn(symbols, interner, "pad_sequence", vec![TypeId::INT64], TypeId::INT64);
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
    fn transform_trait_registered() {
        let (mut i, mut s) = fresh();
        register_transforms(&mut i, &mut s);
        assert!(s.lookup("Transform").is_some());
    }

    #[test]
    fn transform_structs_registered() {
        let (mut i, mut s) = fresh();
        register_transforms(&mut i, &mut s);
        for name in &[
            "Compose", "Resize", "CenterCrop", "RandomCrop",
            "RandomHorizontalFlip", "Normalize", "ToTensor",
            "Tokenizer", "PadSequence",
        ] {
            assert!(s.lookup(name).is_some(), "{} should be registered", name);
        }
    }

    #[test]
    fn transform_fns_registered() {
        let (mut i, mut s) = fresh();
        register_transforms(&mut i, &mut s);
        for name in &[
            "resize", "center_crop", "random_crop", "random_horizontal_flip",
            "normalize_transform", "to_tensor_transform", "tokenize", "pad_sequence",
        ] {
            assert!(s.lookup(name).is_some(), "{} should be registered", name);
        }
    }

    #[test]
    fn tokenize_signature() {
        let (mut i, mut s) = fresh();
        register_transforms(&mut i, &mut s);
        let sym_id = s.lookup("tokenize").unwrap();
        let sym = s.get_symbol(sym_id);
        match i.resolve(sym.ty) {
            Type::Function { params, ret } => {
                assert_eq!(params.len(), 1);
                assert_eq!(params[0], TypeId::STRING);
                assert_eq!(*ret, TypeId::INT64);
            }
            _ => panic!("tokenize should be a function"),
        }
    }
}
