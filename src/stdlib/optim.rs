// stdlib/optim.rs — Optimizers and learning-rate schedulers.

use crate::symbol::SymbolTable;
use crate::types::*;

use super::{def_fn, def_method, def_struct, def_trait};

/// Register optimizer types and functions.
pub fn register_optim(interner: &mut TypeInterner, symbols: &mut SymbolTable) {
    register_traits(interner, symbols);
    register_optimizer_structs(interner, symbols);
    register_scheduler_structs(interner, symbols);
    register_functions(interner, symbols);
    register_optimizer_methods(interner, symbols);
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
    let param_groups_ty = interner.intern(Type::Function {
        params: vec![],
        ret: TypeId::INT64, // Vec<ParamGroup> approximated as INT64
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
            ("param_groups".into(), param_groups_ty),
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
    def_struct(symbols, interner, "SGD", vec![
        ("lr".into(), TypeId::FLOAT64),
        ("momentum".into(), TypeId::FLOAT64),
        ("weight_decay".into(), TypeId::FLOAT64),
    ], vec![]);
    def_struct(symbols, interner, "Adam", vec![
        ("lr".into(), TypeId::FLOAT64),
        ("momentum".into(), TypeId::FLOAT64),
        ("weight_decay".into(), TypeId::FLOAT64),
    ], vec![]);
    def_struct(symbols, interner, "AdamW", vec![
        ("lr".into(), TypeId::FLOAT64),
        ("momentum".into(), TypeId::FLOAT64),
        ("weight_decay".into(), TypeId::FLOAT64),
    ], vec![]);
    def_struct(symbols, interner, "RMSprop", vec![
        ("lr".into(), TypeId::FLOAT64),
        ("momentum".into(), TypeId::FLOAT64),
        ("weight_decay".into(), TypeId::FLOAT64),
    ], vec![]);
    def_struct(symbols, interner, "Adagrad", vec![
        ("lr".into(), TypeId::FLOAT64),
        ("momentum".into(), TypeId::FLOAT64),
        ("weight_decay".into(), TypeId::FLOAT64),
    ], vec![]);
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
    // Optimizer constructors — all take (lr: Float64) for basic construction
    // (momentum and weight_decay default to 0.0; full constructors below)
    def_fn(symbols, interner, "sgd_new", vec![TypeId::FLOAT64], TypeId::INT64);
    def_fn(symbols, interner, "adam_new", vec![TypeId::FLOAT64], TypeId::INT64);
    def_fn(symbols, interner, "adamw_new", vec![TypeId::FLOAT64], TypeId::INT64);
    def_fn(symbols, interner, "rmsprop_new", vec![TypeId::FLOAT64], TypeId::INT64);
    def_fn(symbols, interner, "adagrad_new", vec![TypeId::FLOAT64], TypeId::INT64);

    // Full constructors with all hyperparams: (lr, momentum, weight_decay)
    def_fn(symbols, interner, "sgd_full_new",
        vec![TypeId::FLOAT64, TypeId::FLOAT64, TypeId::FLOAT64], TypeId::INT64);
    def_fn(symbols, interner, "adam_full_new",
        vec![TypeId::FLOAT64, TypeId::FLOAT64, TypeId::FLOAT64], TypeId::INT64);
    def_fn(symbols, interner, "adamw_full_new",
        vec![TypeId::FLOAT64, TypeId::FLOAT64, TypeId::FLOAT64], TypeId::INT64);
    def_fn(symbols, interner, "rmsprop_full_new",
        vec![TypeId::FLOAT64, TypeId::FLOAT64, TypeId::FLOAT64], TypeId::INT64);

    // Scheduler constructors
    def_fn(symbols, interner, "step_lr_new", vec![TypeId::INT64, TypeId::FLOAT32], TypeId::INT64);
    def_fn(symbols, interner, "cosine_annealing_lr_new", vec![TypeId::INT64], TypeId::INT64);
    def_fn(symbols, interner, "reduce_lr_on_plateau_new", vec![TypeId::INT64], TypeId::INT64);
    def_fn(symbols, interner, "one_cycle_lr_new", vec![TypeId::FLOAT32, TypeId::INT64], TypeId::INT64);
}

// -- Optimizer instance methods -----------------------------------------------
// Register step(), zero_grad(), param_groups() as methods on each optimizer type.

fn register_optimizer_methods_for(interner: &mut TypeInterner, symbols: &mut SymbolTable, type_name: &str) {
    // step(self) -> Unit
    def_method(symbols, interner, type_name, "step",
        vec![TypeId::INT64], TypeId::UNIT);
    // zero_grad(self) -> Unit
    def_method(symbols, interner, type_name, "zero_grad",
        vec![TypeId::INT64], TypeId::UNIT);
    // param_groups(self) -> Int64 (Vec<ParamGroup>)
    def_method(symbols, interner, type_name, "param_groups",
        vec![TypeId::INT64], TypeId::INT64);
    // state_dict(self) -> Int64
    def_method(symbols, interner, type_name, "state_dict",
        vec![TypeId::INT64], TypeId::INT64);
    // load_state_dict(self, state: Int64) -> Unit
    def_method(symbols, interner, type_name, "load_state_dict",
        vec![TypeId::INT64, TypeId::INT64], TypeId::UNIT);
}

fn register_optimizer_methods(interner: &mut TypeInterner, symbols: &mut SymbolTable) {
    for type_name in &["SGD", "Adam", "AdamW", "RMSprop", "Adagrad"] {
        register_optimizer_methods_for(interner, symbols, type_name);
    }
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
    fn optimizer_trait_has_param_groups() {
        let (mut i, mut s) = fresh();
        register_optim(&mut i, &mut s);
        let sym_id = s.lookup("Optimizer").unwrap();
        let sym = s.get_symbol(sym_id);
        match i.resolve(sym.ty) {
            Type::Trait { methods, .. } => {
                let names: Vec<&str> = methods.iter().map(|(n, _)| n.as_str()).collect();
                assert!(names.contains(&"step"), "Optimizer should have step");
                assert!(names.contains(&"zero_grad"), "Optimizer should have zero_grad");
                assert!(names.contains(&"param_groups"), "Optimizer should have param_groups");
            }
            _ => panic!("Optimizer should be a trait"),
        }
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
    fn optimizer_structs_have_fields() {
        let (mut i, mut s) = fresh();
        register_optim(&mut i, &mut s);
        let sym_id = s.lookup("SGD").unwrap();
        let sym = s.get_symbol(sym_id);
        match i.resolve(sym.ty) {
            Type::Struct { fields, .. } => {
                let names: Vec<&str> = fields.iter().map(|(n, _)| n.as_str()).collect();
                assert!(names.contains(&"lr"), "SGD should have lr");
                assert!(names.contains(&"momentum"), "SGD should have momentum");
                assert!(names.contains(&"weight_decay"), "SGD should have weight_decay");
            }
            _ => panic!("SGD should be a struct"),
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
    fn full_construction_fns_registered() {
        let (mut i, mut s) = fresh();
        register_optim(&mut i, &mut s);
        for name in &["sgd_full_new", "adam_full_new", "adamw_full_new", "rmsprop_full_new"] {
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
                assert_eq!(params[0], TypeId::FLOAT64);
                assert_eq!(*ret, TypeId::INT64);
            }
            _ => panic!("adam_new should be a function"),
        }
    }

    #[test]
    fn optimizer_methods_registered() {
        let (mut i, mut s) = fresh();
        register_optim(&mut i, &mut s);
        for type_name in &["SGD", "Adam", "AdamW", "RMSprop"] {
            for method in &["step", "zero_grad", "param_groups"] {
                let qualified = format!("{}::{}", type_name, method);
                assert!(s.lookup(&qualified).is_some(), "{} should be registered", qualified);
            }
        }
    }
}
