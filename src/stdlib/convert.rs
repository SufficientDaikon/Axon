// stdlib/convert.rs — Conversion traits (From, Into, TryFrom, TryInto).

use crate::symbol::SymbolTable;
use crate::types::*;

use super::def_trait;

/// Register conversion traits.
pub fn register_convert(interner: &mut TypeInterner, symbols: &mut SymbolTable) {
    // From<T>: fn from(val: T) -> Self
    let from_method = interner.intern(Type::Function {
        params: vec![TypeId::UNIT],
        ret: TypeId::UNIT,
    });
    def_trait(symbols, interner, "From", vec![("from".into(), from_method)], vec![]);

    // Into<T>: fn into(self) -> T
    let into_method = interner.intern(Type::Function {
        params: vec![],
        ret: TypeId::UNIT,
    });
    def_trait(symbols, interner, "Into", vec![("into".into(), into_method)], vec![]);

    // TryFrom<T>: fn try_from(val: T) -> Result<Self, E>
    let try_from_method = interner.intern(Type::Function {
        params: vec![TypeId::UNIT],
        ret: TypeId::UNIT,
    });
    def_trait(
        symbols,
        interner,
        "TryFrom",
        vec![("try_from".into(), try_from_method)],
        vec![],
    );

    // TryInto<T>: fn try_into(self) -> Result<T, E>
    let try_into_method = interner.intern(Type::Function {
        params: vec![],
        ret: TypeId::UNIT,
    });
    def_trait(
        symbols,
        interner,
        "TryInto",
        vec![("try_into".into(), try_into_method)],
        vec![],
    );

    // AsRef / AsMut convenience traits
    let as_ref_method = interner.intern(Type::Function {
        params: vec![],
        ret: TypeId::UNIT,
    });
    def_trait(symbols, interner, "AsRef", vec![("as_ref".into(), as_ref_method)], vec![]);

    let as_mut_method = interner.intern(Type::Function {
        params: vec![],
        ret: TypeId::UNIT,
    });
    def_trait(symbols, interner, "AsMut", vec![("as_mut".into(), as_mut_method)], vec![]);
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
    fn from_trait_registered() {
        let (mut i, mut s) = fresh();
        register_convert(&mut i, &mut s);
        assert!(s.lookup("From").is_some());
    }

    #[test]
    fn into_trait_registered() {
        let (mut i, mut s) = fresh();
        register_convert(&mut i, &mut s);
        assert!(s.lookup("Into").is_some());
    }

    #[test]
    fn try_from_trait_registered() {
        let (mut i, mut s) = fresh();
        register_convert(&mut i, &mut s);
        assert!(s.lookup("TryFrom").is_some());
    }

    #[test]
    fn try_into_trait_registered() {
        let (mut i, mut s) = fresh();
        register_convert(&mut i, &mut s);
        assert!(s.lookup("TryInto").is_some());
    }

    #[test]
    fn all_conversion_traits_are_trait_types() {
        let (mut i, mut s) = fresh();
        register_convert(&mut i, &mut s);
        for name in &["From", "Into", "TryFrom", "TryInto", "AsRef", "AsMut"] {
            let sym_id = s.lookup(name).unwrap();
            let sym = s.get_symbol(sym_id);
            assert!(
                matches!(i.resolve(sym.ty), Type::Trait { .. }),
                "{} should be a trait type",
                name
            );
        }
    }
}
