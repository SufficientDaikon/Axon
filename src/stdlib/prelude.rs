// stdlib/prelude.rs — Core traits and ubiquitous functions.

use crate::symbol::SymbolTable;
use crate::types::*;

use super::{def_fn, def_method, def_trait};

/// Register prelude items: core traits and always-available functions.
pub fn register_prelude(interner: &mut TypeInterner, symbols: &mut SymbolTable) {
    // -- Core output functions ------------------------------------------------
    def_fn(symbols, interner, "println", vec![TypeId::STRING], TypeId::UNIT);
    def_fn(symbols, interner, "print", vec![TypeId::STRING], TypeId::UNIT);
    def_fn(symbols, interner, "eprintln", vec![TypeId::STRING], TypeId::UNIT);

    // -- Assertions / panics --------------------------------------------------
    def_fn(symbols, interner, "assert", vec![TypeId::BOOL], TypeId::UNIT);
    def_fn(symbols, interner, "assert_eq", vec![TypeId::INT32, TypeId::INT32], TypeId::UNIT);
    def_fn(symbols, interner, "panic", vec![TypeId::STRING], TypeId::NEVER);
    def_fn(symbols, interner, "unreachable", vec![], TypeId::NEVER);
    def_fn(symbols, interner, "todo", vec![], TypeId::NEVER);
    def_fn(symbols, interner, "unimplemented", vec![], TypeId::NEVER);
    def_fn(symbols, interner, "dbg", vec![TypeId::STRING], TypeId::STRING);

    // -- Core traits ----------------------------------------------------------
    // Each trait is registered as a named Trait type so user code can reference it.
    register_core_traits(interner, symbols);
}

fn register_core_traits(interner: &mut TypeInterner, symbols: &mut SymbolTable) {
    // Display: fn fmt(&self) -> String
    let fmt_ty = interner.intern(Type::Function {
        params: vec![],
        ret: TypeId::STRING,
    });
    def_trait(symbols, interner, "Display", vec![("fmt".into(), fmt_ty)], vec![]);

    // Debug: fn debug_fmt(&self) -> String
    let debug_fmt_ty = interner.intern(Type::Function {
        params: vec![],
        ret: TypeId::STRING,
    });
    def_trait(symbols, interner, "Debug", vec![("debug_fmt".into(), debug_fmt_ty)], vec![]);

    // Clone: fn clone(&self) -> Self  (approximated as Unit → Unit)
    let clone_ty = interner.intern(Type::Function {
        params: vec![],
        ret: TypeId::UNIT,
    });
    def_trait(symbols, interner, "Clone", vec![("clone".into(), clone_ty)], vec![]);

    // Copy (marker trait — no methods)
    def_trait(symbols, interner, "Copy", vec![], vec![]);

    // Default: fn default() -> Self
    let default_ty = interner.intern(Type::Function {
        params: vec![],
        ret: TypeId::UNIT,
    });
    def_trait(symbols, interner, "Default", vec![("default".into(), default_ty)], vec![]);

    // Drop: fn drop(&mut self)
    let drop_ty = interner.intern(Type::Function {
        params: vec![],
        ret: TypeId::UNIT,
    });
    def_trait(symbols, interner, "Drop", vec![("drop".into(), drop_ty)], vec![]);

    // PartialEq: fn eq(&self, other: &Self) -> Bool
    let eq_ty = interner.intern(Type::Function {
        params: vec![TypeId::UNIT],
        ret: TypeId::BOOL,
    });
    def_trait(symbols, interner, "PartialEq", vec![("eq".into(), eq_ty)], vec![]);

    // Eq (marker trait extending PartialEq)
    def_trait(symbols, interner, "Eq", vec![], vec![]);

    // PartialOrd: fn partial_cmp(&self, other: &Self) -> Option<i32>
    let partial_cmp_ty = interner.intern(Type::Function {
        params: vec![TypeId::UNIT],
        ret: TypeId::INT32,
    });
    def_trait(
        symbols,
        interner,
        "PartialOrd",
        vec![("partial_cmp".into(), partial_cmp_ty)],
        vec![],
    );

    // Ord: fn cmp(&self, other: &Self) -> i32
    let cmp_ty = interner.intern(Type::Function {
        params: vec![TypeId::UNIT],
        ret: TypeId::INT32,
    });
    def_trait(symbols, interner, "Ord", vec![("cmp".into(), cmp_ty)], vec![]);

    // Hash: fn hash(&self) -> i64
    let hash_ty = interner.intern(Type::Function {
        params: vec![],
        ret: TypeId::INT64,
    });
    def_trait(symbols, interner, "Hash", vec![("hash".into(), hash_ty)], vec![]);

    // Iterator: fn next(&mut self) -> Option<T>  (approximated)
    let next_ty = interner.intern(Type::Function {
        params: vec![],
        ret: TypeId::UNIT,
    });
    def_trait(symbols, interner, "Iterator", vec![("next".into(), next_ty)], vec![]);

    // Register Iterator default methods as type stubs.
    // Generic types are approximated: T → UNIT, U → UNIT, Fn → UNIT,
    // Iterator<T> → UNIT, Option<T> → UNIT, Vec<T> → UNIT, (T, U) → UNIT.
    // This matches the fidelity level used throughout the stdlib.

    let iter_self = TypeId::UNIT; // self: Iterator<T> (approximated)

    // next(self) -> Option<T>  (also registered as a method for uniform lookup)
    def_method(symbols, interner, "Iterator", "next", vec![iter_self], TypeId::UNIT);

    // map(self, f: Fn(T) -> U) -> Iterator<U>
    def_method(symbols, interner, "Iterator", "map", vec![iter_self, TypeId::UNIT], TypeId::UNIT);

    // filter(self, f: Fn(T) -> Bool) -> Iterator<T>
    def_method(symbols, interner, "Iterator", "filter", vec![iter_self, TypeId::UNIT], TypeId::UNIT);

    // fold(self, init: U, f: Fn(U, T) -> U) -> U
    def_method(symbols, interner, "Iterator", "fold", vec![iter_self, TypeId::UNIT, TypeId::UNIT], TypeId::UNIT);

    // collect(self) -> Vec<T>
    def_method(symbols, interner, "Iterator", "collect", vec![iter_self], TypeId::UNIT);

    // zip(self, other: Iterator<U>) -> Iterator<(T, U)>
    def_method(symbols, interner, "Iterator", "zip", vec![iter_self, TypeId::UNIT], TypeId::UNIT);

    // enumerate(self) -> Iterator<(Int64, T)>
    def_method(symbols, interner, "Iterator", "enumerate", vec![iter_self], TypeId::UNIT);

    // take(self, n: Int64) -> Iterator<T>
    def_method(symbols, interner, "Iterator", "take", vec![iter_self, TypeId::INT64], TypeId::UNIT);

    // skip(self, n: Int64) -> Iterator<T>
    def_method(symbols, interner, "Iterator", "skip", vec![iter_self, TypeId::INT64], TypeId::UNIT);

    // count(self) -> Int64
    def_method(symbols, interner, "Iterator", "count", vec![iter_self], TypeId::INT64);

    // any(self, f: Fn(T) -> Bool) -> Bool
    def_method(symbols, interner, "Iterator", "any", vec![iter_self, TypeId::UNIT], TypeId::BOOL);

    // all(self, f: Fn(T) -> Bool) -> Bool
    def_method(symbols, interner, "Iterator", "all", vec![iter_self, TypeId::UNIT], TypeId::BOOL);

    // find(self, f: Fn(T) -> Bool) -> Option<T>
    def_method(symbols, interner, "Iterator", "find", vec![iter_self, TypeId::UNIT], TypeId::UNIT);

    // chain(self, other: Iterator<T>) -> Iterator<T>
    def_method(symbols, interner, "Iterator", "chain", vec![iter_self, TypeId::UNIT], TypeId::UNIT);

    // flat_map(self, f: Fn(T) -> Iterator<U>) -> Iterator<U>
    def_method(symbols, interner, "Iterator", "flat_map", vec![iter_self, TypeId::UNIT], TypeId::UNIT);
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
    fn println_registered() {
        let (mut i, mut s) = fresh();
        register_prelude(&mut i, &mut s);
        assert!(s.lookup("println").is_some());
    }

    #[test]
    fn print_registered() {
        let (mut i, mut s) = fresh();
        register_prelude(&mut i, &mut s);
        assert!(s.lookup("print").is_some());
    }

    #[test]
    fn eprintln_registered() {
        let (mut i, mut s) = fresh();
        register_prelude(&mut i, &mut s);
        assert!(s.lookup("eprintln").is_some());
    }

    #[test]
    fn assert_registered() {
        let (mut i, mut s) = fresh();
        register_prelude(&mut i, &mut s);
        assert!(s.lookup("assert").is_some());
    }

    #[test]
    fn panic_registered() {
        let (mut i, mut s) = fresh();
        register_prelude(&mut i, &mut s);
        let sym_id = s.lookup("panic").unwrap();
        let sym = s.get_symbol(sym_id);
        let ty = i.resolve(sym.ty);
        match ty {
            Type::Function { ret, .. } => assert_eq!(*ret, TypeId::NEVER),
            _ => panic!("panic should be a function type"),
        }
    }

    #[test]
    fn display_trait_registered() {
        let (mut i, mut s) = fresh();
        register_prelude(&mut i, &mut s);
        let sym_id = s.lookup("Display").unwrap();
        let sym = s.get_symbol(sym_id);
        let ty = i.resolve(sym.ty);
        assert!(matches!(ty, Type::Trait { name, .. } if name == "Display"));
    }

    #[test]
    fn all_core_traits_registered() {
        let (mut i, mut s) = fresh();
        register_prelude(&mut i, &mut s);
        for name in &[
            "Display", "Debug", "Clone", "Copy", "Default", "Drop",
            "PartialEq", "Eq", "PartialOrd", "Ord", "Hash", "Iterator",
        ] {
            assert!(s.lookup(name).is_some(), "trait {} should be registered", name);
        }
    }

    #[test]
    fn iterator_default_methods_registered() {
        let (mut i, mut s) = fresh();
        register_prelude(&mut i, &mut s);
        let methods = [
            "Iterator::next",
            "Iterator::map",
            "Iterator::filter",
            "Iterator::fold",
            "Iterator::collect",
            "Iterator::zip",
            "Iterator::enumerate",
            "Iterator::take",
            "Iterator::skip",
            "Iterator::count",
            "Iterator::any",
            "Iterator::all",
            "Iterator::find",
            "Iterator::chain",
            "Iterator::flat_map",
        ];
        for method in &methods {
            assert!(s.lookup(method).is_some(), "{} should be registered", method);
        }
        assert_eq!(methods.len(), 15, "Iterator should have 15 methods total");
    }
}
