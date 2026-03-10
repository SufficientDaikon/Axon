// stdlib/collections.rs — Vec, HashMap, HashSet, Option, Result.

use crate::symbol::SymbolTable;
use crate::types::*;

use super::{def_fn, def_method, def_struct};

/// Register collection types and their methods.
pub fn register_collections(interner: &mut TypeInterner, symbols: &mut SymbolTable) {
    register_vec(interner, symbols);
    register_hashmap(interner, symbols);
    register_hashset(interner, symbols);
    register_btreemap(interner, symbols);
    register_deque(interner, symbols);
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
    def_method(symbols, interner, "Vec", "first", vec![vec_ty], TypeId::UNIT);
    def_method(symbols, interner, "Vec", "last", vec![vec_ty], TypeId::UNIT);
    def_method(symbols, interner, "Vec", "swap", vec![vec_ty, TypeId::INT64, TypeId::INT64], TypeId::UNIT);
    def_method(symbols, interner, "Vec", "retain", vec![vec_ty, TypeId::UNIT], TypeId::UNIT);
    def_method(symbols, interner, "Vec", "dedup", vec![vec_ty], TypeId::UNIT);
    def_method(symbols, interner, "Vec", "windows", vec![vec_ty, TypeId::INT64], TypeId::UNIT);
    def_method(symbols, interner, "Vec", "chunks", vec![vec_ty, TypeId::INT64], TypeId::UNIT);
    def_method(symbols, interner, "Vec", "split_at", vec![vec_ty, TypeId::INT64], TypeId::UNIT);
    def_method(symbols, interner, "Vec", "resize", vec![vec_ty, TypeId::INT64, TypeId::UNIT], TypeId::UNIT);
    def_method(symbols, interner, "Vec", "drain", vec![vec_ty, TypeId::INT64, TypeId::INT64], TypeId::UNIT);
    def_method(symbols, interner, "Vec", "map", vec![vec_ty, TypeId::UNIT], vec_ty);
    def_method(symbols, interner, "Vec", "filter", vec![vec_ty, TypeId::UNIT], vec_ty);
    def_method(symbols, interner, "Vec", "fold", vec![vec_ty, TypeId::UNIT, TypeId::UNIT], TypeId::UNIT);
    def_method(symbols, interner, "Vec", "any", vec![vec_ty, TypeId::UNIT], TypeId::BOOL);
    def_method(symbols, interner, "Vec", "all", vec![vec_ty, TypeId::UNIT], TypeId::BOOL);
    def_method(symbols, interner, "Vec", "zip", vec![vec_ty, vec_ty], TypeId::UNIT);
    def_method(symbols, interner, "Vec", "enumerate", vec![vec_ty], TypeId::UNIT);
    def_method(symbols, interner, "Vec", "flatten", vec![vec_ty], vec_ty);
    def_method(symbols, interner, "Vec", "join", vec![vec_ty, TypeId::STRING], TypeId::STRING);
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
    def_method(symbols, interner, "HashMap", "entry", vec![map_ty, TypeId::UNIT], TypeId::UNIT);
    def_method(symbols, interner, "HashMap", "or_insert", vec![map_ty, TypeId::UNIT, TypeId::UNIT], TypeId::UNIT);
    def_method(symbols, interner, "HashMap", "retain", vec![map_ty, TypeId::UNIT], TypeId::UNIT);
    def_method(
        symbols,
        interner,
        "HashMap",
        "get_or_insert_with",
        vec![map_ty, TypeId::UNIT, TypeId::UNIT],
        TypeId::UNIT,
    );
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
    def_method(symbols, interner, "HashSet", "symmetric_difference", vec![set_ty, set_ty], set_ty);
    def_method(symbols, interner, "HashSet", "is_subset", vec![set_ty, set_ty], TypeId::BOOL);
    def_method(symbols, interner, "HashSet", "is_superset", vec![set_ty, set_ty], TypeId::BOOL);
    def_method(symbols, interner, "HashSet", "is_disjoint", vec![set_ty, set_ty], TypeId::BOOL);
    def_method(symbols, interner, "HashSet", "clear", vec![set_ty], TypeId::UNIT);
    def_method(symbols, interner, "HashSet", "iter", vec![set_ty], TypeId::UNIT);
    def_method(symbols, interner, "HashSet", "retain", vec![set_ty, TypeId::UNIT], TypeId::UNIT);
}

// -- BTreeMap<K, V> -----------------------------------------------------------

fn register_btreemap(interner: &mut TypeInterner, symbols: &mut SymbolTable) {
    let btree_ty = def_struct(symbols, interner, "BTreeMap", vec![], vec![]);

    def_method(symbols, interner, "BTreeMap", "new", vec![], btree_ty);
    def_method(
        symbols,
        interner,
        "BTreeMap",
        "insert",
        vec![btree_ty, TypeId::UNIT, TypeId::UNIT],
        TypeId::UNIT,
    );
    def_method(symbols, interner, "BTreeMap", "get", vec![btree_ty, TypeId::UNIT], TypeId::UNIT);
    def_method(symbols, interner, "BTreeMap", "remove", vec![btree_ty, TypeId::UNIT], TypeId::UNIT);
    def_method(
        symbols,
        interner,
        "BTreeMap",
        "contains_key",
        vec![btree_ty, TypeId::UNIT],
        TypeId::BOOL,
    );
    def_method(symbols, interner, "BTreeMap", "len", vec![btree_ty], TypeId::INT64);
    def_method(symbols, interner, "BTreeMap", "is_empty", vec![btree_ty], TypeId::BOOL);
    def_method(symbols, interner, "BTreeMap", "keys", vec![btree_ty], TypeId::UNIT);
    def_method(symbols, interner, "BTreeMap", "values", vec![btree_ty], TypeId::UNIT);
    def_method(symbols, interner, "BTreeMap", "iter", vec![btree_ty], TypeId::UNIT);
    def_method(symbols, interner, "BTreeMap", "clear", vec![btree_ty], TypeId::UNIT);
    def_method(symbols, interner, "BTreeMap", "first_key_value", vec![btree_ty], TypeId::UNIT);
    def_method(symbols, interner, "BTreeMap", "last_key_value", vec![btree_ty], TypeId::UNIT);
    def_method(symbols, interner, "BTreeMap", "range", vec![btree_ty, TypeId::UNIT, TypeId::UNIT], TypeId::UNIT);
    def_method(symbols, interner, "BTreeMap", "entry", vec![btree_ty, TypeId::UNIT], TypeId::UNIT);
}

// -- Deque<T> -----------------------------------------------------------------

fn register_deque(interner: &mut TypeInterner, symbols: &mut SymbolTable) {
    let deque_ty = def_struct(symbols, interner, "Deque", vec![], vec![]);

    def_method(symbols, interner, "Deque", "new", vec![], deque_ty);
    def_method(symbols, interner, "Deque", "with_capacity", vec![TypeId::INT64], deque_ty);
    def_method(symbols, interner, "Deque", "push_back", vec![deque_ty, TypeId::UNIT], TypeId::UNIT);
    def_method(symbols, interner, "Deque", "push_front", vec![deque_ty, TypeId::UNIT], TypeId::UNIT);
    def_method(symbols, interner, "Deque", "pop_back", vec![deque_ty], TypeId::UNIT);
    def_method(symbols, interner, "Deque", "pop_front", vec![deque_ty], TypeId::UNIT);
    def_method(symbols, interner, "Deque", "front", vec![deque_ty], TypeId::UNIT);
    def_method(symbols, interner, "Deque", "back", vec![deque_ty], TypeId::UNIT);
    def_method(symbols, interner, "Deque", "len", vec![deque_ty], TypeId::INT64);
    def_method(symbols, interner, "Deque", "is_empty", vec![deque_ty], TypeId::BOOL);
    def_method(symbols, interner, "Deque", "clear", vec![deque_ty], TypeId::UNIT);
    def_method(symbols, interner, "Deque", "get", vec![deque_ty, TypeId::INT64], TypeId::UNIT);
    def_method(symbols, interner, "Deque", "contains", vec![deque_ty, TypeId::UNIT], TypeId::BOOL);
    def_method(symbols, interner, "Deque", "iter", vec![deque_ty], TypeId::UNIT);
    def_method(symbols, interner, "Deque", "rotate_left", vec![deque_ty, TypeId::INT64], TypeId::UNIT);
    def_method(symbols, interner, "Deque", "rotate_right", vec![deque_ty, TypeId::INT64], TypeId::UNIT);
    def_method(symbols, interner, "Deque", "swap", vec![deque_ty, TypeId::INT64, TypeId::INT64], TypeId::UNIT);
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
    def_method(symbols, interner, "Option", "filter", vec![option_ty, TypeId::UNIT], option_ty);
    def_method(symbols, interner, "Option", "or_else", vec![option_ty, TypeId::UNIT], option_ty);
    def_method(symbols, interner, "Option", "flatten", vec![option_ty], option_ty);
    def_method(symbols, interner, "Option", "expect", vec![option_ty, TypeId::STRING], TypeId::UNIT);
    def_method(symbols, interner, "Option", "zip", vec![option_ty, option_ty], option_ty);
    def_method(symbols, interner, "Option", "ok_or", vec![option_ty, TypeId::STRING], TypeId::UNIT);

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
    def_method(symbols, interner, "Result", "and_then", vec![result_ty, TypeId::UNIT], result_ty);
    def_method(symbols, interner, "Result", "or_else", vec![result_ty, TypeId::UNIT], result_ty);
    def_method(symbols, interner, "Result", "expect", vec![result_ty, TypeId::STRING], TypeId::UNIT);
    def_method(symbols, interner, "Result", "expect_err", vec![result_ty, TypeId::STRING], TypeId::STRING);
    def_method(symbols, interner, "Result", "ok", vec![result_ty], TypeId::UNIT);
    def_method(symbols, interner, "Result", "err", vec![result_ty], TypeId::UNIT);
    def_method(symbols, interner, "Result", "flatten", vec![result_ty], result_ty);

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
        for method in &[
            "Vec::new", "Vec::push", "Vec::pop", "Vec::len", "Vec::sort",
            "Vec::first", "Vec::last", "Vec::retain", "Vec::dedup",
            "Vec::windows", "Vec::chunks", "Vec::map", "Vec::filter",
            "Vec::join", "Vec::flatten", "Vec::enumerate",
        ] {
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
        assert!(s.lookup("HashMap::entry").is_some());
        assert!(s.lookup("HashMap::retain").is_some());
    }

    #[test]
    fn hashset_registered() {
        let (mut i, mut s) = fresh();
        register_collections(&mut i, &mut s);
        assert!(s.lookup("HashSet").is_some());
        assert!(s.lookup("HashSet::union").is_some());
        assert!(s.lookup("HashSet::symmetric_difference").is_some());
        assert!(s.lookup("HashSet::is_subset").is_some());
        assert!(s.lookup("HashSet::iter").is_some());
    }

    #[test]
    fn btreemap_registered() {
        let (mut i, mut s) = fresh();
        register_collections(&mut i, &mut s);
        assert!(s.lookup("BTreeMap").is_some());
        assert!(s.lookup("BTreeMap::insert").is_some());
        assert!(s.lookup("BTreeMap::get").is_some());
        assert!(s.lookup("BTreeMap::first_key_value").is_some());
        assert!(s.lookup("BTreeMap::last_key_value").is_some());
        assert!(s.lookup("BTreeMap::range").is_some());
    }

    #[test]
    fn deque_registered() {
        let (mut i, mut s) = fresh();
        register_collections(&mut i, &mut s);
        assert!(s.lookup("Deque").is_some());
        assert!(s.lookup("Deque::push_back").is_some());
        assert!(s.lookup("Deque::push_front").is_some());
        assert!(s.lookup("Deque::pop_back").is_some());
        assert!(s.lookup("Deque::pop_front").is_some());
        assert!(s.lookup("Deque::front").is_some());
        assert!(s.lookup("Deque::back").is_some());
        assert!(s.lookup("Deque::rotate_left").is_some());
    }

    #[test]
    fn option_registered() {
        let (mut i, mut s) = fresh();
        register_collections(&mut i, &mut s);
        assert!(s.lookup("Option").is_some());
        assert!(s.lookup("Some").is_some());
        assert!(s.lookup("None").is_some());
        assert!(s.lookup("Option::unwrap").is_some());
        assert!(s.lookup("Option::map").is_some());
        assert!(s.lookup("Option::and_then").is_some());
        assert!(s.lookup("Option::filter").is_some());
        assert!(s.lookup("Option::or_else").is_some());
        assert!(s.lookup("Option::expect").is_some());
        assert!(s.lookup("Option::flatten").is_some());
    }

    #[test]
    fn result_registered() {
        let (mut i, mut s) = fresh();
        register_collections(&mut i, &mut s);
        assert!(s.lookup("Result").is_some());
        assert!(s.lookup("Ok").is_some());
        assert!(s.lookup("Err").is_some());
        assert!(s.lookup("Result::unwrap").is_some());
        assert!(s.lookup("Result::and_then").is_some());
        assert!(s.lookup("Result::or_else").is_some());
        assert!(s.lookup("Result::expect").is_some());
        assert!(s.lookup("Result::ok").is_some());
        assert!(s.lookup("Result::err").is_some());
    }
}
