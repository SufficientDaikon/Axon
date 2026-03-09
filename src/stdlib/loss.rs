// stdlib/loss.rs — Loss functions for training.

use crate::symbol::SymbolTable;
use crate::types::*;

use super::{def_fn, def_struct, def_trait};

/// Register loss function types and constructors.
pub fn register_loss(interner: &mut TypeInterner, symbols: &mut SymbolTable) {
    register_trait(interner, symbols);
    register_structs(interner, symbols);
    register_functions(interner, symbols);
}

// -- LossFn trait -------------------------------------------------------------

fn register_trait(interner: &mut TypeInterner, symbols: &mut SymbolTable) {
    let forward_ty = interner.intern(Type::Function {
        params: vec![TypeId::INT64, TypeId::INT64],
        ret: TypeId::FLOAT32,
    });
    def_trait(
        symbols,
        interner,
        "LossFn",
        vec![("forward".into(), forward_ty)],
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
        assert!(s.lookup("LossFn").is_some());
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
}
