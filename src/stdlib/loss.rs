// stdlib/loss.rs — Loss functions for training.

use crate::symbol::SymbolTable;
use crate::types::*;

use super::{def_fn, def_method, def_struct, def_trait};

/// Register loss function types and constructors.
pub fn register_loss(interner: &mut TypeInterner, symbols: &mut SymbolTable) {
    register_trait(interner, symbols);
    register_structs(interner, symbols);
    register_functions(interner, symbols);
    register_loss_methods(interner, symbols);
}

// -- LossFn trait -------------------------------------------------------------

fn register_trait(interner: &mut TypeInterner, symbols: &mut SymbolTable) {
    let forward_ty = interner.intern(Type::Function {
        params: vec![TypeId::INT64, TypeId::INT64],  // pred: Tensor, target: Tensor
        ret: TypeId::INT64,                          // -> Tensor (loss scalar)
    });
    let backward_ty = interner.intern(Type::Function {
        params: vec![],
        ret: TypeId::INT64,  // -> Tensor (gradient)
    });
    def_trait(
        symbols,
        interner,
        "LossFn",
        vec![
            ("forward".into(), forward_ty),
            ("backward".into(), backward_ty),
        ],
        vec![],
    );
}

// -- Loss structs -------------------------------------------------------------

fn register_structs(interner: &mut TypeInterner, symbols: &mut SymbolTable) {
    def_struct(symbols, interner, "MSELoss", vec![], vec![]);
    def_struct(symbols, interner, "L1Loss", vec![], vec![]);
    def_struct(symbols, interner, "CrossEntropyLoss", vec![], vec![]);
    def_struct(symbols, interner, "BCELoss", vec![], vec![]);
    def_struct(symbols, interner, "BCEWithLogitsLoss", vec![], vec![]);
    def_struct(symbols, interner, "NLLLoss", vec![], vec![]);
    def_struct(symbols, interner, "HuberLoss", vec![("delta".into(), TypeId::FLOAT32)], vec![]);
    def_struct(symbols, interner, "KLDivLoss", vec![], vec![]);
    def_struct(symbols, interner, "CosineEmbeddingLoss", vec![], vec![]);
    def_struct(symbols, interner, "TripletMarginLoss", vec![("margin".into(), TypeId::FLOAT32)], vec![]);
    def_struct(symbols, interner, "CTCLoss", vec![], vec![]);
}

// -- Constructor functions ----------------------------------------------------

fn register_functions(interner: &mut TypeInterner, symbols: &mut SymbolTable) {
    def_fn(symbols, interner, "mse_loss", vec![], TypeId::INT64);
    def_fn(symbols, interner, "l1_loss", vec![], TypeId::INT64);
    def_fn(symbols, interner, "cross_entropy_loss", vec![], TypeId::INT64);
    def_fn(symbols, interner, "bce_loss", vec![], TypeId::INT64);
    def_fn(symbols, interner, "bce_with_logits_loss", vec![], TypeId::INT64);
    def_fn(symbols, interner, "nll_loss", vec![], TypeId::INT64);
    def_fn(symbols, interner, "huber_loss", vec![TypeId::FLOAT32], TypeId::INT64);
    def_fn(symbols, interner, "kl_div_loss", vec![], TypeId::INT64);
    def_fn(symbols, interner, "cosine_embedding_loss", vec![], TypeId::INT64);
    def_fn(symbols, interner, "triplet_margin_loss", vec![TypeId::FLOAT32], TypeId::INT64);
    def_fn(symbols, interner, "ctc_loss", vec![], TypeId::INT64);
}

// -- Loss type methods --------------------------------------------------------
// Each loss type gets forward(self, pred: Tensor, target: Tensor) -> Tensor
// and backward(self) -> Tensor methods.

fn register_loss_methods_for(interner: &mut TypeInterner, symbols: &mut SymbolTable, type_name: &str) {
    // forward(self, pred: Tensor, target: Tensor) -> Tensor
    def_method(symbols, interner, type_name, "forward",
        vec![TypeId::INT64, TypeId::INT64, TypeId::INT64], TypeId::INT64);
    // backward(self) -> Tensor
    def_method(symbols, interner, type_name, "backward",
        vec![TypeId::INT64], TypeId::INT64);
}

fn register_loss_methods(interner: &mut TypeInterner, symbols: &mut SymbolTable) {
    let loss_types = [
        "MSELoss", "L1Loss", "CrossEntropyLoss", "BCELoss",
        "BCEWithLogitsLoss", "NLLLoss", "HuberLoss", "KLDivLoss",
        "CosineEmbeddingLoss", "TripletMarginLoss", "CTCLoss",
    ];
    for type_name in &loss_types {
        register_loss_methods_for(interner, symbols, type_name);
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
    fn loss_trait_registered() {
        let (mut i, mut s) = fresh();
        register_loss(&mut i, &mut s);
        let sym_id = s.lookup("LossFn").unwrap();
        let sym = s.get_symbol(sym_id);
        match i.resolve(sym.ty) {
            Type::Trait { methods, .. } => {
                let names: Vec<&str> = methods.iter().map(|(n, _)| n.as_str()).collect();
                assert!(names.contains(&"forward"), "LossFn should have forward");
                assert!(names.contains(&"backward"), "LossFn should have backward");
            }
            _ => panic!("LossFn should be a trait"),
        }
    }

    #[test]
    fn loss_structs_registered() {
        let (mut i, mut s) = fresh();
        register_loss(&mut i, &mut s);
        for name in &[
            "MSELoss", "L1Loss", "CrossEntropyLoss", "BCELoss",
            "BCEWithLogitsLoss", "NLLLoss", "HuberLoss", "KLDivLoss",
            "CosineEmbeddingLoss", "TripletMarginLoss", "CTCLoss",
        ] {
            assert!(s.lookup(name).is_some(), "{} should be registered", name);
        }
    }

    #[test]
    fn loss_fns_registered() {
        let (mut i, mut s) = fresh();
        register_loss(&mut i, &mut s);
        for name in &[
            "mse_loss", "l1_loss", "cross_entropy_loss", "bce_loss",
            "nll_loss", "huber_loss", "ctc_loss",
        ] {
            assert!(s.lookup(name).is_some(), "{} should be registered", name);
        }
    }

    #[test]
    fn huber_loss_signature() {
        let (mut i, mut s) = fresh();
        register_loss(&mut i, &mut s);
        let sym_id = s.lookup("huber_loss").unwrap();
        let sym = s.get_symbol(sym_id);
        match i.resolve(sym.ty) {
            Type::Function { params, ret } => {
                assert_eq!(params.len(), 1);
                assert_eq!(params[0], TypeId::FLOAT32);
                assert_eq!(*ret, TypeId::INT64);
            }
            _ => panic!("huber_loss should be a function"),
        }
    }

    #[test]
    fn loss_forward_methods_registered() {
        let (mut i, mut s) = fresh();
        register_loss(&mut i, &mut s);
        for type_name in &["MSELoss", "CrossEntropyLoss", "BCELoss", "L1Loss", "NLLLoss"] {
            let method = format!("{}::forward", type_name);
            assert!(s.lookup(&method).is_some(), "{} should be registered", method);
        }
    }

    #[test]
    fn loss_backward_methods_registered() {
        let (mut i, mut s) = fresh();
        register_loss(&mut i, &mut s);
        for type_name in &["MSELoss", "CrossEntropyLoss", "BCELoss", "L1Loss", "NLLLoss"] {
            let method = format!("{}::backward", type_name);
            assert!(s.lookup(&method).is_some(), "{} should be registered", method);
        }
    }
}
