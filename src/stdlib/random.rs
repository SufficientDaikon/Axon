// stdlib/random.rs — Random number generation functions.

use crate::symbol::SymbolTable;
use crate::types::*;

use super::{def_fn, def_method, def_struct};

/// Register random number generation functions and types.
pub fn register_random(interner: &mut TypeInterner, symbols: &mut SymbolTable) {
    // Free functions
    def_fn(symbols, interner, "random", vec![], TypeId::FLOAT64);
    def_fn(
        symbols,
        interner,
        "random_range",
        vec![TypeId::FLOAT64, TypeId::FLOAT64],
        TypeId::FLOAT64,
    );
    def_fn(symbols, interner, "random_int", vec![TypeId::INT64, TypeId::INT64], TypeId::INT64);
    def_fn(symbols, interner, "seed", vec![TypeId::INT64], TypeId::UNIT);
    def_fn(symbols, interner, "random_normal", vec![TypeId::FLOAT64, TypeId::FLOAT64], TypeId::FLOAT64);
    def_fn(symbols, interner, "random_uniform", vec![TypeId::FLOAT64, TypeId::FLOAT64], TypeId::FLOAT64);
    def_fn(symbols, interner, "random_bool", vec![TypeId::FLOAT64], TypeId::BOOL);

    // Rng struct for stateful generation
    let rng_ty = def_struct(symbols, interner, "Rng", vec![], vec![]);
    def_method(symbols, interner, "Rng", "new", vec![TypeId::INT64], rng_ty);
    def_method(symbols, interner, "Rng", "next_float", vec![rng_ty], TypeId::FLOAT64);
    def_method(symbols, interner, "Rng", "next_int", vec![rng_ty, TypeId::INT64, TypeId::INT64], TypeId::INT64);
    def_method(symbols, interner, "Rng", "shuffle", vec![rng_ty, TypeId::UNIT], TypeId::UNIT);
    def_method(symbols, interner, "Rng", "choice", vec![rng_ty, TypeId::UNIT], TypeId::UNIT);
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
    fn random_registered() {
        let (mut i, mut s) = fresh();
        register_random(&mut i, &mut s);
        assert!(s.lookup("random").is_some());
    }

    #[test]
    fn random_range_registered() {
        let (mut i, mut s) = fresh();
        register_random(&mut i, &mut s);
        assert!(s.lookup("random_range").is_some());
    }

    #[test]
    fn seed_registered() {
        let (mut i, mut s) = fresh();
        register_random(&mut i, &mut s);
        assert!(s.lookup("seed").is_some());
    }

    #[test]
    fn rng_struct_registered() {
        let (mut i, mut s) = fresh();
        register_random(&mut i, &mut s);
        assert!(s.lookup("Rng").is_some());
        assert!(s.lookup("Rng::new").is_some());
        assert!(s.lookup("Rng::next_float").is_some());
    }

    #[test]
    fn random_returns_float64() {
        let (mut i, mut s) = fresh();
        register_random(&mut i, &mut s);
        let sym_id = s.lookup("random").unwrap();
        let sym = s.get_symbol(sym_id);
        let ty = i.resolve(sym.ty);
        match ty {
            Type::Function { params, ret } => {
                assert!(params.is_empty());
                assert_eq!(*ret, TypeId::FLOAT64);
            }
            _ => panic!("random should be a function"),
        }
    }
}
