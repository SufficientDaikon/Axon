// stdlib/math.rs — Math constants and functions.

use crate::symbol::SymbolTable;
use crate::types::*;

use super::{def_const, def_fn};

/// Register math constants and functions.
pub fn register_math(interner: &mut TypeInterner, symbols: &mut SymbolTable) {
    register_constants(symbols);
    register_trig(interner, symbols);
    register_exponential(interner, symbols);
    register_rounding(interner, symbols);
    register_misc(interner, symbols);
}

// -- Constants ----------------------------------------------------------------

fn register_constants(symbols: &mut SymbolTable) {
    def_const(symbols, "PI", TypeId::FLOAT64);
    def_const(symbols, "E", TypeId::FLOAT64);
    def_const(symbols, "TAU", TypeId::FLOAT64);
    def_const(symbols, "INF", TypeId::FLOAT64);
    def_const(symbols, "NEG_INF", TypeId::FLOAT64);
    def_const(symbols, "NAN", TypeId::FLOAT64);
    def_const(symbols, "MAX_INT", TypeId::INT64);
    def_const(symbols, "MIN_INT", TypeId::INT64);
}

// -- Trigonometric functions --------------------------------------------------

fn register_trig(interner: &mut TypeInterner, symbols: &mut SymbolTable) {
    def_fn(symbols, interner, "sin", vec![TypeId::FLOAT64], TypeId::FLOAT64);
    def_fn(symbols, interner, "cos", vec![TypeId::FLOAT64], TypeId::FLOAT64);
    def_fn(symbols, interner, "tan", vec![TypeId::FLOAT64], TypeId::FLOAT64);
    def_fn(symbols, interner, "asin", vec![TypeId::FLOAT64], TypeId::FLOAT64);
    def_fn(symbols, interner, "acos", vec![TypeId::FLOAT64], TypeId::FLOAT64);
    def_fn(symbols, interner, "atan", vec![TypeId::FLOAT64], TypeId::FLOAT64);
    def_fn(
        symbols,
        interner,
        "atan2",
        vec![TypeId::FLOAT64, TypeId::FLOAT64],
        TypeId::FLOAT64,
    );
    def_fn(symbols, interner, "sinh", vec![TypeId::FLOAT64], TypeId::FLOAT64);
    def_fn(symbols, interner, "cosh", vec![TypeId::FLOAT64], TypeId::FLOAT64);
    def_fn(symbols, interner, "tanh", vec![TypeId::FLOAT64], TypeId::FLOAT64);
}

// -- Exponential / logarithmic ------------------------------------------------

fn register_exponential(interner: &mut TypeInterner, symbols: &mut SymbolTable) {
    def_fn(symbols, interner, "exp", vec![TypeId::FLOAT64], TypeId::FLOAT64);
    def_fn(symbols, interner, "log", vec![TypeId::FLOAT64], TypeId::FLOAT64);
    def_fn(symbols, interner, "log2", vec![TypeId::FLOAT64], TypeId::FLOAT64);
    def_fn(symbols, interner, "log10", vec![TypeId::FLOAT64], TypeId::FLOAT64);
    def_fn(
        symbols,
        interner,
        "pow",
        vec![TypeId::FLOAT64, TypeId::FLOAT64],
        TypeId::FLOAT64,
    );
    def_fn(symbols, interner, "sqrt", vec![TypeId::FLOAT64], TypeId::FLOAT64);
    def_fn(symbols, interner, "cbrt", vec![TypeId::FLOAT64], TypeId::FLOAT64);
}

// -- Rounding / truncation ----------------------------------------------------

fn register_rounding(interner: &mut TypeInterner, symbols: &mut SymbolTable) {
    def_fn(symbols, interner, "floor", vec![TypeId::FLOAT64], TypeId::FLOAT64);
    def_fn(symbols, interner, "ceil", vec![TypeId::FLOAT64], TypeId::FLOAT64);
    def_fn(symbols, interner, "round", vec![TypeId::FLOAT64], TypeId::FLOAT64);
    def_fn(symbols, interner, "trunc", vec![TypeId::FLOAT64], TypeId::FLOAT64);
    def_fn(symbols, interner, "abs", vec![TypeId::FLOAT64], TypeId::FLOAT64);
}

// -- Misc math ----------------------------------------------------------------

fn register_misc(interner: &mut TypeInterner, symbols: &mut SymbolTable) {
    def_fn(
        symbols,
        interner,
        "min",
        vec![TypeId::FLOAT64, TypeId::FLOAT64],
        TypeId::FLOAT64,
    );
    def_fn(
        symbols,
        interner,
        "max",
        vec![TypeId::FLOAT64, TypeId::FLOAT64],
        TypeId::FLOAT64,
    );
    def_fn(
        symbols,
        interner,
        "clamp",
        vec![TypeId::FLOAT64, TypeId::FLOAT64, TypeId::FLOAT64],
        TypeId::FLOAT64,
    );
    def_fn(symbols, interner, "signum", vec![TypeId::FLOAT64], TypeId::FLOAT64);
    def_fn(
        symbols,
        interner,
        "hypot",
        vec![TypeId::FLOAT64, TypeId::FLOAT64],
        TypeId::FLOAT64,
    );
    def_fn(symbols, interner, "fract", vec![TypeId::FLOAT64], TypeId::FLOAT64);
    def_fn(symbols, interner, "is_nan", vec![TypeId::FLOAT64], TypeId::BOOL);
    def_fn(symbols, interner, "is_inf", vec![TypeId::FLOAT64], TypeId::BOOL);
    def_fn(symbols, interner, "is_finite", vec![TypeId::FLOAT64], TypeId::BOOL);
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
    fn constants_registered() {
        let (mut i, mut s) = fresh();
        register_math(&mut i, &mut s);
        for name in &["PI", "E", "TAU", "INF", "NEG_INF", "NAN"] {
            assert!(s.lookup(name).is_some(), "{} should be registered", name);
        }
    }

    #[test]
    fn trig_functions_registered() {
        let (mut i, mut s) = fresh();
        register_math(&mut i, &mut s);
        for name in &["sin", "cos", "tan", "asin", "acos", "atan", "atan2"] {
            assert!(s.lookup(name).is_some(), "{} should be registered", name);
        }
    }

    #[test]
    fn exp_log_functions_registered() {
        let (mut i, mut s) = fresh();
        register_math(&mut i, &mut s);
        for name in &["exp", "log", "log2", "log10", "pow", "sqrt", "cbrt"] {
            assert!(s.lookup(name).is_some(), "{} should be registered", name);
        }
    }

    #[test]
    fn rounding_functions_registered() {
        let (mut i, mut s) = fresh();
        register_math(&mut i, &mut s);
        for name in &["floor", "ceil", "round", "trunc", "abs"] {
            assert!(s.lookup(name).is_some(), "{} should be registered", name);
        }
    }

    #[test]
    fn sin_has_correct_signature() {
        let (mut i, mut s) = fresh();
        register_math(&mut i, &mut s);
        let sym_id = s.lookup("sin").unwrap();
        let sym = s.get_symbol(sym_id);
        let ty = i.resolve(sym.ty);
        match ty {
            Type::Function { params, ret } => {
                assert_eq!(params.len(), 1);
                assert_eq!(params[0], TypeId::FLOAT64);
                assert_eq!(*ret, TypeId::FLOAT64);
            }
            _ => panic!("sin should be a function"),
        }
    }
}
