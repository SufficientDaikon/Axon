// stdlib/collections.rs — Vec, HashMap, HashSet, Option, Result.

use crate::symbol::SymbolTable;
use crate::types::*;

use super::{def_fn, def_method, def_struct};

/// Register collection types and their methods.
pub fn register_collections(interner: &mut TypeInterner, symbols: &mut SymbolTable) {
    register_vec(interner, symbols);
    register_hashmap(interner, symbols);
    register_hashset(interner, symbols);
    register_option(interner, symbols);
    register_result(interner, symbols);
}

// -- Vec<T> -------------------------------------------------------------------

fn register_vec(interner: &mut TypeInterner, symbols: &mut SymbolTable) {
    let vec_ty = def_struct(symbols, interner, "Vec", vec![], vec![]);

    def_method(symbols, interner, "Vec", "new", vec![], vec_ty);
    def_method(symbols, interner, "Vec", "with_capacity", vec![TypeId::INT64], vec_ty);
    def_method(symbols, interner, "Vec", "push", vec![vec_ty, TypeId::UNIT], TypeId::UNIT);
    def_method(symbols, interner, "Vec", "pop", vec![vec_ty], TypeId::UNIT);
    def_method(symbols, interner, "Vec", "len", vec![vec_ty], TypeId::INT64);
    def_method(symbols, interner, "Vec", "is_empty", vec![vec_ty], TypeId::BOOL);
    def_method(symbols, interner, "Vec", "get", vec![vec_ty, TypeId::INT64], TypeId::UNIT);
    def_method(symbols, interner, "Vec", "contains", vec![vec_ty, TypeId::UNIT], TypeId::BOOL);
    def_method(symbols, interner, "Vec", "clear", vec![vec_ty], TypeId::UNIT);
    def_method(symbols, interner, "Vec", "remove", vec![vec_ty, TypeId::INT64], TypeId::UNIT);
    def_method(
        symbols,
        interner,
        "Vec",
        "insert",
        vec![vec_ty, TypeId::INT64, TypeId::UNIT],
        TypeId::UNIT,
    );
    def_method(symbols, interner, "Vec", "sort", vec![vec_ty], TypeId::UNIT);
    def_method(symbols, interner, "Vec", "iter", vec![vec_ty], TypeId::UNIT);
    def_method(symbols, interner, "Vec", "extend", vec![vec_ty, TypeId::UNIT], TypeId::UNIT);
    def_method(symbols, interner, "Vec", "as_slice", vec![vec_ty], TypeId::UNIT);
    def_method(symbols, interner, "Vec", "reverse", vec![vec_ty], TypeId::UNIT);
    def_method(symbols, interner, "Vec", "truncate", vec![vec_ty, TypeId::INT64], TypeId::UNIT);
    def_method(symbols, interner, "Vec", "capacity", vec![vec_ty], TypeId::INT64);
}

// -- HashMap<K, V> ------------------------------------------------------------

fn register_hashmap(interner: &mut TypeInterner, symbols: &mut SymbolTable) {
    let map_ty = def_struct(symbols, interner, "HashMap", vec![], vec![]);

    def_method(symbols, interner, "HashMap", "new", vec![], map_ty);
    def_method(
        symbols,
        interner,
        "HashMap",
        "insert",
        vec![map_ty, TypeId::UNIT, TypeId::UNIT],
        TypeId::UNIT,
    );
    def_method(symbols, interner, "HashMap", "get", vec![map_ty, TypeId::UNIT], TypeId::UNIT);
    def_method(symbols, interner, "HashMap", "get_mut", vec![map_ty, TypeId::UNIT], TypeId::UNIT);
    def_method(symbols, interner, "HashMap", "remove", vec![map_ty, TypeId::UNIT], TypeId::UNIT);
    def_method(
        symbols,
        interner,
        "HashMap",
        "contains_key",
        vec![map_ty, TypeId::UNIT],
        TypeId::BOOL,
    );
    def_method(symbols, interner, "HashMap", "len", vec![map_ty], TypeId::INT64);
    def_method(symbols, interner, "HashMap", "is_empty", vec![map_ty], TypeId::BOOL);
    def_method(symbols, interner, "HashMap", "keys", vec![map_ty], TypeId::UNIT);
    def_method(symbols, interner, "HashMap", "values", vec![map_ty], TypeId::UNIT);
    def_method(symbols, interner, "HashMap", "iter", vec![map_ty], TypeId::UNIT);
    def_method(symbols, interner, "HashMap", "clear", vec![map_ty], TypeId::UNIT);
}

// -- HashSet<T> ---------------------------------------------------------------

fn register_hashset(interner: &mut TypeInterner, symbols: &mut SymbolTable) {
    let set_ty = def_struct(symbols, interner, "HashSet", vec![], vec![]);

    def_method(symbols, interner, "HashSet", "new", vec![], set_ty);
    def_method(symbols, interner, "HashSet", "insert", vec![set_ty, TypeId::UNIT], TypeId::BOOL);
    def_method(
        symbols,
        interner,
        "HashSet",
        "contains",
        vec![set_ty, TypeId::UNIT],
        TypeId::BOOL,
    );
    def_method(symbols, interner, "HashSet", "remove", vec![set_ty, TypeId::UNIT], TypeId::BOOL);
    def_method(symbols, interner, "HashSet", "len", vec![set_ty], TypeId::INT64);
    def_method(symbols, interner, "HashSet", "is_empty", vec![set_ty], TypeId::BOOL);
    def_method(symbols, interner, "HashSet", "union", vec![set_ty, set_ty], set_ty);
    def_method(symbols, interner, "HashSet", "intersection", vec![set_ty, set_ty], set_ty);
    def_method(symbols, interner, "HashSet", "difference", vec![set_ty, set_ty], set_ty);
    def_method(symbols, interner, "HashSet", "clear", vec![set_ty], TypeId::UNIT);
}

// -- Option<T> ----------------------------------------------------------------

fn register_option(interner: &mut TypeInterner, symbols: &mut SymbolTable) {
    // Option as an enum with Some(T), None
    let option_ty = interner.intern(Type::Enum {
        name: "Option".to_string(),
        variants: vec![
            ("Some".into(), EnumVariantType::Tuple(vec![TypeId::UNIT])),
            ("None".into(), EnumVariantType::Unit),
        ],
        generics: vec![],
    });

    let info = crate::symbol::SymbolInfo {
        name: "Option".to_string(),
        ty: option_ty,
        kind: crate::symbol::SymbolKind::EnumDef,
        mutable: false,
        span: crate::span::Span::dummy(),
        scope: symbols.current_scope(),
        visible: true,
    };
    let _ = symbols.define("Option".to_string(), info);

    def_method(symbols, interner, "Option", "is_some", vec![option_ty], TypeId::BOOL);
    def_method(symbols, interner, "Option", "is_none", vec![option_ty], TypeId::BOOL);
    def_method(symbols, interner, "Option", "unwrap", vec![option_ty], TypeId::UNIT);
    def_method(
        symbols,
        interner,
        "Option",
        "unwrap_or",
        vec![option_ty, TypeId::UNIT],
        TypeId::UNIT,
    );
    def_method(symbols, interner, "Option", "map", vec![option_ty, TypeId::UNIT], option_ty);
    def_method(symbols, interner, "Option", "and_then", vec![option_ty, TypeId::UNIT], option_ty);

    // Global constructors
    def_fn(symbols, interner, "Some", vec![TypeId::UNIT], option_ty);
    def_fn(symbols, interner, "None", vec![], option_ty);
}

// -- Result<T, E> -------------------------------------------------------------

fn register_result(interner: &mut TypeInterner, symbols: &mut SymbolTable) {
    let result_ty = interner.intern(Type::Enum {
        name: "Result".to_string(),
        variants: vec![
            ("Ok".into(), EnumVariantType::Tuple(vec![TypeId::UNIT])),
            ("Err".into(), EnumVariantType::Tuple(vec![TypeId::STRING])),
        ],
        generics: vec![],
    });

    let info = crate::symbol::SymbolInfo {
        name: "Result".to_string(),
        ty: result_ty,
        kind: crate::symbol::SymbolKind::EnumDef,
        mutable: false,
        span: crate::span::Span::dummy(),
        scope: symbols.current_scope(),
        visible: true,
    };
    let _ = symbols.define("Result".to_string(), info);

    def_method(symbols, interner, "Result", "is_ok", vec![result_ty], TypeId::BOOL);
    def_method(symbols, interner, "Result", "is_err", vec![result_ty], TypeId::BOOL);
    def_method(symbols, interner, "Result", "unwrap", vec![result_ty], TypeId::UNIT);
    def_method(symbols, interner, "Result", "unwrap_err", vec![result_ty], TypeId::STRING);
    def_method(
        symbols,
        interner,
        "Result",
        "unwrap_or",
        vec![result_ty, TypeId::UNIT],
        TypeId::UNIT,
    );
    def_method(symbols, interner, "Result", "map", vec![result_ty, TypeId::UNIT], result_ty);
    def_method(symbols, interner, "Result", "map_err", vec![result_ty, TypeId::UNIT], result_ty);

    // Global constructors
    def_fn(symbols, interner, "Ok", vec![TypeId::UNIT], result_ty);
    def_fn(symbols, interner, "Err", vec![TypeId::STRING], result_ty);
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
    fn vec_registered() {
        let (mut i, mut s) = fresh();
        register_collections(&mut i, &mut s);
        assert!(s.lookup("Vec").is_some());
    }

    #[test]
    fn vec_methods_registered() {
        let (mut i, mut s) = fresh();
        register_collections(&mut i, &mut s);
        for method in &["Vec::new", "Vec::push", "Vec::pop", "Vec::len", "Vec::sort"] {
            assert!(s.lookup(method).is_some(), "{} should be registered", method);
        }
    }

    #[test]
    fn hashmap_registered() {
        let (mut i, mut s) = fresh();
        register_collections(&mut i, &mut s);
        assert!(s.lookup("HashMap").is_some());
        assert!(s.lookup("HashMap::insert").is_some());
        assert!(s.lookup("HashMap::get").is_some());
    }

    #[test]
    fn hashset_registered() {
        let (mut i, mut s) = fresh();
        register_collections(&mut i, &mut s);
        assert!(s.lookup("HashSet").is_some());
        assert!(s.lookup("HashSet::union").is_some());
    }

    #[test]
    fn option_registered() {
        let (mut i, mut s) = fresh();
        register_collections(&mut i, &mut s);
        assert!(s.lookup("Option").is_some());
        assert!(s.lookup("Some").is_some());
        assert!(s.lookup("None").is_some());
        assert!(s.lookup("Option::unwrap").is_some());
    }

    #[test]
    fn result_registered() {
        let (mut i, mut s) = fresh();
        register_collections(&mut i, &mut s);
        assert!(s.lookup("Result").is_some());
        assert!(s.lookup("Ok").is_some());
        assert!(s.lookup("Err").is_some());
        assert!(s.lookup("Result::unwrap").is_some());
    }
}
