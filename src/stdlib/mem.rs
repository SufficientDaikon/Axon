// stdlib/mem.rs — Memory utilities (size_of, align_of, swap, drop).

use crate::symbol::SymbolTable;
use crate::types::*;

use super::def_fn;

/// Register memory utility functions.
pub fn register_mem(interner: &mut TypeInterner, symbols: &mut SymbolTable) {
    // size_of<T>() -> i64 (approximated as no-arg returning i64)
    def_fn(symbols, interner, "size_of", vec![], TypeId::INT64);

    // align_of<T>() -> i64
    def_fn(symbols, interner, "align_of", vec![], TypeId::INT64);

    // swap<T>(a: &mut T, b: &mut T) (approximated)
    def_fn(
        symbols,
        interner,
        "swap",
        vec![TypeId::UNIT, TypeId::UNIT],
        TypeId::UNIT,
    );

    // drop<T>(val: T)
    def_fn(symbols, interner, "drop", vec![TypeId::UNIT], TypeId::UNIT);

    // forget<T>(val: T) — prevent destructor
    def_fn(symbols, interner, "forget", vec![TypeId::UNIT], TypeId::UNIT);

    // transmute<T, U>(val: T) -> U (unsafe, approximated)
    def_fn(symbols, interner, "transmute", vec![TypeId::UNIT], TypeId::UNIT);

    // replace<T>(dest: &mut T, val: T) -> T
    def_fn(
        symbols,
        interner,
        "replace",
        vec![TypeId::UNIT, TypeId::UNIT],
        TypeId::UNIT,
    );

    // take<T>(dest: &mut T) -> T (replaces with Default)
    def_fn(symbols, interner, "take", vec![TypeId::UNIT], TypeId::UNIT);
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
    fn size_of_registered() {
        let (mut i, mut s) = fresh();
        register_mem(&mut i, &mut s);
        assert!(s.lookup("size_of").is_some());
    }

    #[test]
    fn align_of_registered() {
        let (mut i, mut s) = fresh();
        register_mem(&mut i, &mut s);
        assert!(s.lookup("align_of").is_some());
    }

    #[test]
    fn swap_registered() {
        let (mut i, mut s) = fresh();
        register_mem(&mut i, &mut s);
        assert!(s.lookup("swap").is_some());
    }

    #[test]
    fn drop_registered() {
        let (mut i, mut s) = fresh();
        register_mem(&mut i, &mut s);
        assert!(s.lookup("drop").is_some());
    }

    #[test]
    fn size_of_returns_int64() {
        let (mut i, mut s) = fresh();
        register_mem(&mut i, &mut s);
        let sym_id = s.lookup("size_of").unwrap();
        let sym = s.get_symbol(sym_id);
        let ty = i.resolve(sym.ty);
        match ty {
            Type::Function { ret, .. } => assert_eq!(*ret, TypeId::INT64),
            _ => panic!("size_of should be a function"),
        }
    }
}
