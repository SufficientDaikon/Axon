// stdlib/ops.rs — Operator traits (Add, Sub, Mul, etc.)

use crate::symbol::SymbolTable;
use crate::types::*;

use super::def_trait;

/// Register operator traits.
pub fn register_ops(interner: &mut TypeInterner, symbols: &mut SymbolTable) {
    // Binary operator traits: fn op(&self, rhs: Self) -> Self
    let binop_method = interner.intern(Type::Function {
        params: vec![TypeId::UNIT],
        ret: TypeId::UNIT,
    });

    def_trait(symbols, interner, "Add", vec![("add".into(), binop_method)], vec![]);
    def_trait(symbols, interner, "Sub", vec![("sub".into(), binop_method)], vec![]);
    def_trait(symbols, interner, "Mul", vec![("mul".into(), binop_method)], vec![]);
    def_trait(symbols, interner, "Div", vec![("div".into(), binop_method)], vec![]);
    def_trait(symbols, interner, "Mod", vec![("rem".into(), binop_method)], vec![]);
    def_trait(symbols, interner, "MatMul", vec![("matmul".into(), binop_method)], vec![]);

    // Unary operator traits: fn op(&self) -> Self
    let unop_method = interner.intern(Type::Function {
        params: vec![],
        ret: TypeId::UNIT,
    });
    def_trait(symbols, interner, "Neg", vec![("neg".into(), unop_method)], vec![]);
    def_trait(symbols, interner, "Not", vec![("not".into(), unop_method)], vec![]);

    // Index: fn index(&self, idx: I) -> &T  (approximated)
    let index_method = interner.intern(Type::Function {
        params: vec![TypeId::INT64],
        ret: TypeId::UNIT,
    });
    def_trait(symbols, interner, "Index", vec![("index".into(), index_method)], vec![]);

    let index_mut_method = interner.intern(Type::Function {
        params: vec![TypeId::INT64],
        ret: TypeId::UNIT,
    });
    def_trait(
        symbols,
        interner,
        "IndexMut",
        vec![("index_mut".into(), index_mut_method)],
        vec![],
    );

    // Bitwise operator traits
    let bitop_method = interner.intern(Type::Function {
        params: vec![TypeId::UNIT],
        ret: TypeId::UNIT,
    });
    def_trait(symbols, interner, "BitAnd", vec![("bitand".into(), bitop_method)], vec![]);
    def_trait(symbols, interner, "BitOr", vec![("bitor".into(), bitop_method)], vec![]);
    def_trait(symbols, interner, "BitXor", vec![("bitxor".into(), bitop_method)], vec![]);
    def_trait(symbols, interner, "Shl", vec![("shl".into(), bitop_method)], vec![]);
    def_trait(symbols, interner, "Shr", vec![("shr".into(), bitop_method)], vec![]);
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
    fn add_trait_registered() {
        let (mut i, mut s) = fresh();
        register_ops(&mut i, &mut s);
        let sym_id = s.lookup("Add").unwrap();
        let sym = s.get_symbol(sym_id);
        assert!(matches!(i.resolve(sym.ty), Type::Trait { name, .. } if name == "Add"));
    }

    #[test]
    fn all_arithmetic_traits_registered() {
        let (mut i, mut s) = fresh();
        register_ops(&mut i, &mut s);
        for name in &["Add", "Sub", "Mul", "Div", "Mod"] {
            assert!(s.lookup(name).is_some(), "{} should be registered", name);
        }
    }

    #[test]
    fn unary_traits_registered() {
        let (mut i, mut s) = fresh();
        register_ops(&mut i, &mut s);
        assert!(s.lookup("Neg").is_some());
        assert!(s.lookup("Not").is_some());
    }

    #[test]
    fn index_traits_registered() {
        let (mut i, mut s) = fresh();
        register_ops(&mut i, &mut s);
        assert!(s.lookup("Index").is_some());
        assert!(s.lookup("IndexMut").is_some());
    }

    #[test]
    fn matmul_trait_registered() {
        let (mut i, mut s) = fresh();
        register_ops(&mut i, &mut s);
        assert!(s.lookup("MatMul").is_some());
    }
}
