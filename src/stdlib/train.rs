// stdlib/train.rs — Training utilities: trainer, checkpoints, mixed precision.

use crate::symbol::SymbolTable;
use crate::types::*;

use super::{def_fn, def_struct, def_trait};

/// Register training utility types and functions.
pub fn register_train(interner: &mut TypeInterner, symbols: &mut SymbolTable) {
    register_callback_trait(interner, symbols);
    register_structs(interner, symbols);
    register_functions(interner, symbols);
}

// -- Callback trait -----------------------------------------------------------

fn register_callback_trait(interner: &mut TypeInterner, symbols: &mut SymbolTable) {
    let on_epoch_start_ty = interner.intern(Type::Function {
        params: vec![TypeId::INT64],
        ret: TypeId::UNIT,
    });
    let on_epoch_end_ty = interner.intern(Type::Function {
        params: vec![TypeId::INT64, TypeId::FLOAT32],
        ret: TypeId::UNIT,
    });
    let on_batch_start_ty = interner.intern(Type::Function {
        params: vec![TypeId::INT64],
        ret: TypeId::UNIT,
    });
    let on_batch_end_ty = interner.intern(Type::Function {
        params: vec![TypeId::INT64, TypeId::FLOAT32],
        ret: TypeId::UNIT,
    });
    let on_train_end_ty = interner.intern(Type::Function {
        params: vec![],
        ret: TypeId::UNIT,
    });
    def_trait(
        symbols,
        interner,
        "Callback",
        vec![
            ("on_epoch_start".into(), on_epoch_start_ty),
            ("on_epoch_end".into(), on_epoch_end_ty),
            ("on_batch_start".into(), on_batch_start_ty),
            ("on_batch_end".into(), on_batch_end_ty),
            ("on_train_end".into(), on_train_end_ty),
        ],
        vec![],
    );
}

// -- Structs ------------------------------------------------------------------

fn register_structs(interner: &mut TypeInterner, symbols: &mut SymbolTable) {
    def_struct(symbols, interner, "Trainer", vec![], vec![]);
    def_struct(
        symbols, interner, "Checkpoint",
        vec![("path".into(), TypeId::STRING), ("epoch".into(), TypeId::INT64)],
        vec![],
    );
    def_struct(
        symbols, interner, "GradScaler",
        vec![("init_scale".into(), TypeId::FLOAT32)],
        vec![],
    );
    def_struct(
        symbols, interner, "EarlyStopping",
        vec![("patience".into(), TypeId::INT64)],
        vec![],
    );
    def_struct(
        symbols, interner, "ModelCheckpoint",
        vec![("path".into(), TypeId::STRING)],
        vec![],
    );
    def_struct(
        symbols, interner, "TensorBoard",
        vec![("log_dir".into(), TypeId::STRING)],
        vec![],
    );
}

// -- Functions ----------------------------------------------------------------

fn register_functions(interner: &mut TypeInterner, symbols: &mut SymbolTable) {
    def_fn(symbols, interner, "trainer_new", vec![], TypeId::INT64);
    def_fn(symbols, interner, "checkpoint_save", vec![TypeId::STRING, TypeId::INT64], TypeId::UNIT);
    def_fn(symbols, interner, "checkpoint_load", vec![TypeId::STRING], TypeId::INT64);
    def_fn(symbols, interner, "grad_scaler_new", vec![TypeId::FLOAT32], TypeId::INT64);
    def_fn(symbols, interner, "autocast", vec![], TypeId::UNIT);
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
    fn callback_trait_registered() {
        let (mut i, mut s) = fresh();
        register_train(&mut i, &mut s);
        assert!(s.lookup("Callback").is_some());
    }

    #[test]
    fn structs_registered() {
        let (mut i, mut s) = fresh();
        register_train(&mut i, &mut s);
        for name in &["Trainer", "Checkpoint", "GradScaler", "EarlyStopping", "ModelCheckpoint", "TensorBoard"] {
            assert!(s.lookup(name).is_some(), "{} should be registered", name);
        }
    }

    #[test]
    fn functions_registered() {
        let (mut i, mut s) = fresh();
        register_train(&mut i, &mut s);
        for name in &["trainer_new", "checkpoint_save", "checkpoint_load", "grad_scaler_new", "autocast"] {
            assert!(s.lookup(name).is_some(), "{} should be registered", name);
        }
    }

    #[test]
    fn checkpoint_save_signature() {
        let (mut i, mut s) = fresh();
        register_train(&mut i, &mut s);
        let sym_id = s.lookup("checkpoint_save").unwrap();
        let sym = s.get_symbol(sym_id);
        match i.resolve(sym.ty) {
            Type::Function { params, ret } => {
                assert_eq!(params.len(), 2);
                assert_eq!(params[0], TypeId::STRING);
                assert_eq!(params[1], TypeId::INT64);
                assert_eq!(*ret, TypeId::UNIT);
            }
            _ => panic!("checkpoint_save should be a function"),
        }
    }
}
