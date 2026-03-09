// stdlib/metrics.rs — Evaluation metrics (accuracy, precision, recall, etc.).

use crate::symbol::SymbolTable;
use crate::types::*;

use super::def_fn;

/// Register evaluation metric functions.
pub fn register_metrics(interner: &mut TypeInterner, symbols: &mut SymbolTable) {
    // Classification metrics
    def_fn(symbols, interner, "accuracy", vec![TypeId::INT64, TypeId::INT64], TypeId::FLOAT32);
    def_fn(symbols, interner, "precision", vec![TypeId::INT64, TypeId::INT64], TypeId::FLOAT32);
    def_fn(symbols, interner, "recall", vec![TypeId::INT64, TypeId::INT64], TypeId::FLOAT32);
    def_fn(symbols, interner, "f1_score", vec![TypeId::INT64, TypeId::INT64], TypeId::FLOAT32);
    def_fn(
        symbols, interner, "confusion_matrix",
        vec![TypeId::INT64, TypeId::INT64, TypeId::INT64],
        TypeId::INT64,
    );
    def_fn(symbols, interner, "roc_auc", vec![TypeId::INT64, TypeId::INT64], TypeId::FLOAT32);

    // Regression metrics
    def_fn(symbols, interner, "mean_squared_error", vec![TypeId::INT64, TypeId::INT64], TypeId::FLOAT32);
    def_fn(symbols, interner, "mean_absolute_error", vec![TypeId::INT64, TypeId::INT64], TypeId::FLOAT32);
    def_fn(symbols, interner, "r2_score", vec![TypeId::INT64, TypeId::INT64], TypeId::FLOAT32);
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
    fn classification_metrics_registered() {
        let (mut i, mut s) = fresh();
        register_metrics(&mut i, &mut s);
        for name in &["accuracy", "precision", "recall", "f1_score", "confusion_matrix", "roc_auc"] {
            assert!(s.lookup(name).is_some(), "{} should be registered", name);
        }
    }

    #[test]
    fn regression_metrics_registered() {
        let (mut i, mut s) = fresh();
        register_metrics(&mut i, &mut s);
        for name in &["mean_squared_error", "mean_absolute_error", "r2_score"] {
            assert!(s.lookup(name).is_some(), "{} should be registered", name);
        }
    }

    #[test]
    fn accuracy_signature() {
        let (mut i, mut s) = fresh();
        register_metrics(&mut i, &mut s);
        let sym_id = s.lookup("accuracy").unwrap();
        let sym = s.get_symbol(sym_id);
        match i.resolve(sym.ty) {
            Type::Function { params, ret } => {
                assert_eq!(params.len(), 2);
                assert_eq!(params[0], TypeId::INT64);
                assert_eq!(params[1], TypeId::INT64);
                assert_eq!(*ret, TypeId::FLOAT32);
            }
            _ => panic!("accuracy should be a function"),
        }
    }

    #[test]
    fn confusion_matrix_signature() {
        let (mut i, mut s) = fresh();
        register_metrics(&mut i, &mut s);
        let sym_id = s.lookup("confusion_matrix").unwrap();
        let sym = s.get_symbol(sym_id);
        match i.resolve(sym.ty) {
            Type::Function { params, ret } => {
                assert_eq!(params.len(), 3);
                assert_eq!(*ret, TypeId::INT64);
            }
            _ => panic!("confusion_matrix should be a function"),
        }
    }
}
