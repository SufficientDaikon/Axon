// stdlib/string.rs — String manipulation methods.

use crate::symbol::SymbolTable;
use crate::types::*;

use super::def_method;

/// Register String methods on the built-in string type.
pub fn register_string(interner: &mut TypeInterner, symbols: &mut SymbolTable) {
    let s = TypeId::STRING;

    def_method(symbols, interner, "String", "len", vec![s], TypeId::INT64);
    def_method(symbols, interner, "String", "is_empty", vec![s], TypeId::BOOL);
    def_method(symbols, interner, "String", "contains", vec![s, s], TypeId::BOOL);
    def_method(symbols, interner, "String", "starts_with", vec![s, s], TypeId::BOOL);
    def_method(symbols, interner, "String", "ends_with", vec![s, s], TypeId::BOOL);
    def_method(symbols, interner, "String", "trim", vec![s], s);
    def_method(symbols, interner, "String", "trim_start", vec![s], s);
    def_method(symbols, interner, "String", "trim_end", vec![s], s);
    def_method(symbols, interner, "String", "split", vec![s, s], TypeId::UNIT);
    def_method(symbols, interner, "String", "chars", vec![s], TypeId::UNIT);
    def_method(symbols, interner, "String", "bytes", vec![s], TypeId::UNIT);
    def_method(symbols, interner, "String", "push_str", vec![s, s], TypeId::UNIT);
    def_method(symbols, interner, "String", "push", vec![s, TypeId::CHAR], TypeId::UNIT);
    def_method(symbols, interner, "String", "replace", vec![s, s, s], s);
    def_method(symbols, interner, "String", "to_uppercase", vec![s], s);
    def_method(symbols, interner, "String", "to_lowercase", vec![s], s);
    def_method(symbols, interner, "String", "substring", vec![s, TypeId::INT64, TypeId::INT64], s);
    def_method(symbols, interner, "String", "parse_int", vec![s], TypeId::INT64);
    def_method(symbols, interner, "String", "parse_float", vec![s], TypeId::FLOAT64);
    def_method(symbols, interner, "String", "repeat", vec![s, TypeId::INT64], s);
    def_method(symbols, interner, "String", "find", vec![s, s], TypeId::INT64);
    def_method(symbols, interner, "String", "rfind", vec![s, s], TypeId::INT64);
    def_method(symbols, interner, "String", "char_at", vec![s, TypeId::INT64], TypeId::CHAR);
    def_method(symbols, interner, "String", "concat", vec![s, s], s);
    def_method(symbols, interner, "String", "lines", vec![s], TypeId::UNIT);
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
    fn string_len_registered() {
        let (mut i, mut s) = fresh();
        register_string(&mut i, &mut s);
        assert!(s.lookup("String::len").is_some());
    }

    #[test]
    fn string_manipulation_methods() {
        let (mut i, mut s) = fresh();
        register_string(&mut i, &mut s);
        for method in &[
            "String::trim",
            "String::to_uppercase",
            "String::to_lowercase",
            "String::replace",
            "String::split",
        ] {
            assert!(s.lookup(method).is_some(), "{} should be registered", method);
        }
    }

    #[test]
    fn string_search_methods() {
        let (mut i, mut s) = fresh();
        register_string(&mut i, &mut s);
        assert!(s.lookup("String::contains").is_some());
        assert!(s.lookup("String::starts_with").is_some());
        assert!(s.lookup("String::ends_with").is_some());
        assert!(s.lookup("String::find").is_some());
    }

    #[test]
    fn string_parse_methods() {
        let (mut i, mut s) = fresh();
        register_string(&mut i, &mut s);
        assert!(s.lookup("String::parse_int").is_some());
        assert!(s.lookup("String::parse_float").is_some());
    }

    #[test]
    fn string_len_returns_int64() {
        let (mut i, mut s) = fresh();
        register_string(&mut i, &mut s);
        let sym_id = s.lookup("String::len").unwrap();
        let sym = s.get_symbol(sym_id);
        let ty = i.resolve(sym.ty);
        match ty {
            Type::Function { ret, .. } => assert_eq!(*ret, TypeId::INT64),
            _ => panic!("String::len should be a function"),
        }
    }
}
