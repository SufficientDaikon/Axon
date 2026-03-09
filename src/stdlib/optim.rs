// stdlib/optim.rs — Optimizers and learning-rate schedulers.

use crate::symbol::SymbolTable;
use crate::types::*;

use super::{def_fn, def_struct, def_trait};

/// Register optimizer types and functions.
pub fn register_optim(interner: &mut TypeInterner, symbols: &mut SymbolTable) {
    register_traits(interner, symbols);
    register_optimizer_structs(interner, symbols);
    register_scheduler_structs(interner, symbols);
    register_functions(interner, symbols);
}

// -- Traits -------------------------------------------------------------------

fn register_traits(interner: &mut TypeInterner, symbols: &mut SymbolTable) {
    let step_ty = interner.intern(Type::Function {
        params: vec![],
        ret: TypeId::UNIT,
    });
    let zero_grad_ty = interner.intern(Type::Function {
        params: vec![],
        ret: TypeId::UNIT,
    });
    let state_dict_ty = interner.intern(Type::Function {
        params: vec![],
        ret: TypeId::INT64,
    });
    let load_state_dict_ty = interner.intern(Type::Function {
        params: vec![TypeId::INT64],
        ret: TypeId::UNIT,
    });
    def_trait(
        symbols,
        interner,
        "Optimizer",
        vec![
            ("step".into(), step_ty),
            ("zero_grad".into(), zero_grad_ty),
            ("state_dict".into(), state_dict_ty),
            ("load_state_dict".into(), load_state_dict_ty),
        ],
        vec![],
    );

    let lr_step_ty = interner.intern(Type::Function {
        params: vec![],
        ret: TypeId::UNIT,
    });
    let get_lr_ty = interner.intern(Type::Function {
        params: vec![],
        ret: TypeId::FLOAT32,
    });
    def_trait(
        symbols,
        interner,
        "LrScheduler",
        vec![
            ("step".into(), lr_step_ty),
            ("get_lr".into(), get_lr_ty),
        ],
        vec![],
    );
}

// -- Optimizer structs --------------------------------------------------------

fn register_optimizer_structs(interner: &mut TypeInterner, symbols: &mut SymbolTable) {
    def_struct(symbols, interner, "SGD", vec![("lr".into(), TypeId::FLOAT32)], vec![]);
    def_struct(symbols, interner, "Adam", vec![("lr".into(), TypeId::FLOAT32)], vec![]);
    def_struct(symbols, interner, "AdamW", vec![("lr".into(), TypeId::FLOAT32)], vec![]);
    def_struct(symbols, interner, "RMSprop", vec![("lr".into(), TypeId::FLOAT32)], vec![]);
    def_struct(symbols, interner, "Adagrad", vec![("lr".into(), TypeId::FLOAT32)], vec![]);
}

// -- Scheduler structs --------------------------------------------------------

fn register_scheduler_structs(interner: &mut TypeInterner, symbols: &mut SymbolTable) {
    def_struct(
        symbols, interner, "StepLR",
        vec![("step_size".into(), TypeId::INT64), ("gamma".into(), TypeId::FLOAT32)],
        vec![],
    );
    def_struct(
        symbols, interner, "CosineAnnealingLR",
        vec![("t_max".into(), TypeId::INT64)],
        vec![],
    );
    def_struct(
        symbols, interner, "ReduceLROnPlateau",
        vec![("patience".into(), TypeId::INT64)],
        vec![],
    );
    def_struct(
        symbols, interner, "OneCycleLR",
        vec![("max_lr".into(), TypeId::FLOAT32), ("total_steps".into(), TypeId::INT64)],
        vec![],
    );
}

// -- Construction functions ---------------------------------------------------

fn register_functions(interner: &mut TypeInterner, symbols: &mut SymbolTable) {
    def_fn(symbols, interner, "sgd_new", vec![TypeId::FLOAT32], TypeId::INT64);
    def_fn(symbols, interner, "adam_new", vec![TypeId::FLOAT32], TypeId::INT64);
    def_fn(symbols, interner, "adamw_new", vec![TypeId::FLOAT32], TypeId::INT64);
    def_fn(symbols, interner, "rmsprop_new", vec![TypeId::FLOAT32], TypeId::INT64);
    def_fn(symbols, interner, "adagrad_new", vec![TypeId::FLOAT32], TypeId::INT64);
    def_fn(symbols, interner, "step_lr_new", vec![TypeId::INT64, TypeId::FLOAT32], TypeId::INT64);
    def_fn(symbols, interner, "cosine_annealing_lr_new", vec![TypeId::INT64], TypeId::INT64);
    def_fn(symbols, interner, "reduce_lr_on_plateau_new", vec![TypeId::INT64], TypeId::INT64);
    def_fn(symbols, interner, "one_cycle_lr_new", vec![TypeId::FLOAT32, TypeId::INT64], TypeId::INT64);
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
    fn traits_registered() {
        let (mut i, mut s) = fresh();
        register_optim(&mut i, &mut s);
        assert!(s.lookup("Optimizer").is_some());
        assert!(s.lookup("LrScheduler").is_some());
    }

    #[test]
    fn optimizer_structs_registered() {
        let (mut i, mut s) = fresh();
        register_optim(&mut i, &mut s);
        for name in &["SGD", "Adam", "AdamW", "RMSprop", "Adagrad"] {
            assert!(s.lookup(name).is_some(), "{} should be registered", name);
        }
    }

    #[test]
    fn scheduler_structs_registered() {
        let (mut i, mut s) = fresh();
        register_optim(&mut i, &mut s);
        for name in &["StepLR", "CosineAnnealingLR", "ReduceLROnPlateau", "OneCycleLR"] {
            assert!(s.lookup(name).is_some(), "{} should be registered", name);
        }
    }

    #[test]
    fn construction_fns_registered() {
        let (mut i, mut s) = fresh();
        register_optim(&mut i, &mut s);
        for name in &["sgd_new", "adam_new", "adamw_new", "rmsprop_new", "adagrad_new"] {
            assert!(s.lookup(name).is_some(), "{} should be registered", name);
        }
    }

    #[test]
    fn adam_new_signature() {
        let (mut i, mut s) = fresh();
        register_optim(&mut i, &mut s);
        let sym_id = s.lookup("adam_new").unwrap();
        let sym = s.get_symbol(sym_id);
        match i.resolve(sym.ty) {
            Type::Function { params, ret } => {
                assert_eq!(params.len(), 1);
                assert_eq!(params[0], TypeId::FLOAT32);
                assert_eq!(*ret, TypeId::INT64);
            }
            _ => panic!("adam_new should be a function"),
        }
    }
}
