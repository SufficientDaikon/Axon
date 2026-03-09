// stdlib/mod.rs — Standard library registration for the Axon type checker.
//
// Registers all built-in types, traits, and functions so that user code can
// reference them without explicit imports.

pub mod prelude;
pub mod ops;
pub mod convert;
pub mod collections;
pub mod tensor;
pub mod math;
pub mod io;
pub mod sync;
pub mod thread;
pub mod data;
pub mod device;
pub mod random;
pub mod string;
pub mod mem;
pub mod time;

use crate::span::Span;
use crate::symbol::*;
use crate::types::*;

// ---------------------------------------------------------------------------
// Helpers shared by all sub-modules
// ---------------------------------------------------------------------------

/// Register a free function in the current scope.
pub(crate) fn def_fn(
    symbols: &mut SymbolTable,
    interner: &mut TypeInterner,
    name: &str,
    params: Vec<TypeId>,
    ret: TypeId,
) {
    let ty = interner.intern(Type::Function { params, ret });
    let info = SymbolInfo {
        name: name.to_string(),
        ty,
        kind: SymbolKind::Function,
        mutable: false,
        span: Span::dummy(),
        scope: symbols.current_scope(),
        visible: true,
    };
    let _ = symbols.define(name.to_string(), info);
}

/// Register a struct type definition in the current scope.
pub(crate) fn def_struct(
    symbols: &mut SymbolTable,
    interner: &mut TypeInterner,
    name: &str,
    fields: Vec<(String, TypeId)>,
    generics: Vec<TypeId>,
) -> TypeId {
    let ty = interner.intern(Type::Struct {
        name: name.to_string(),
        fields,
        generics,
    });
    let info = SymbolInfo {
        name: name.to_string(),
        ty,
        kind: SymbolKind::StructDef,
        mutable: false,
        span: Span::dummy(),
        scope: symbols.current_scope(),
        visible: true,
    };
    let _ = symbols.define(name.to_string(), info);
    ty
}

/// Register a trait type definition in the current scope.
pub(crate) fn def_trait(
    symbols: &mut SymbolTable,
    interner: &mut TypeInterner,
    name: &str,
    methods: Vec<(String, TypeId)>,
    supertraits: Vec<TypeId>,
) -> TypeId {
    let ty = interner.intern(Type::Trait {
        name: name.to_string(),
        methods,
        supertraits,
    });
    let info = SymbolInfo {
        name: name.to_string(),
        ty,
        kind: SymbolKind::TraitDef,
        mutable: false,
        span: Span::dummy(),
        scope: symbols.current_scope(),
        visible: true,
    };
    let _ = symbols.define(name.to_string(), info);
    ty
}

/// Register a method (namespaced as `Type::method`).
pub(crate) fn def_method(
    symbols: &mut SymbolTable,
    interner: &mut TypeInterner,
    type_name: &str,
    method_name: &str,
    params: Vec<TypeId>,
    ret: TypeId,
) {
    let ty = interner.intern(Type::Function { params, ret });
    let qualified = format!("{}::{}", type_name, method_name);
    let info = SymbolInfo {
        name: qualified.clone(),
        ty,
        kind: SymbolKind::Method,
        mutable: false,
        span: Span::dummy(),
        scope: symbols.current_scope(),
        visible: true,
    };
    let _ = symbols.define(qualified, info);
}

/// Register a global constant / variable.
pub(crate) fn def_const(
    symbols: &mut SymbolTable,
    name: &str,
    ty: TypeId,
) {
    let info = SymbolInfo {
        name: name.to_string(),
        ty,
        kind: SymbolKind::Variable,
        mutable: false,
        span: Span::dummy(),
        scope: symbols.current_scope(),
        visible: true,
    };
    let _ = symbols.define(name.to_string(), info);
}

/// Register an enum type definition in the current scope.
pub(crate) fn def_enum(
    symbols: &mut SymbolTable,
    interner: &mut TypeInterner,
    name: &str,
    variants: Vec<(String, EnumVariantType)>,
    generics: Vec<TypeId>,
) -> TypeId {
    let ty = interner.intern(Type::Enum {
        name: name.to_string(),
        variants,
        generics,
    });
    let info = SymbolInfo {
        name: name.to_string(),
        ty,
        kind: SymbolKind::EnumDef,
        mutable: false,
        span: Span::dummy(),
        scope: symbols.current_scope(),
        visible: true,
    };
    let _ = symbols.define(name.to_string(), info);
    ty
}

// ---------------------------------------------------------------------------
// Main entry point
// ---------------------------------------------------------------------------

/// Register the entire Axon standard library into the type checker.
///
/// Call this once before type-checking user code so that built-in types,
/// traits, and functions are available for name resolution.
pub fn register_stdlib(interner: &mut TypeInterner, symbols: &mut SymbolTable) {
    prelude::register_prelude(interner, symbols);
    ops::register_ops(interner, symbols);
    convert::register_convert(interner, symbols);
    collections::register_collections(interner, symbols);
    string::register_string(interner, symbols);
    math::register_math(interner, symbols);
    io::register_io(interner, symbols);
    tensor::register_tensor(interner, symbols);
    sync::register_sync(interner, symbols);
    thread::register_thread(interner, symbols);
    data::register_data(interner, symbols);
    device::register_device(interner, symbols);
    random::register_random(interner, symbols);
    mem::register_mem(interner, symbols);
    time::register_time(interner, symbols);
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn fresh() -> (TypeInterner, SymbolTable) {
        (TypeInterner::new(), SymbolTable::new())
    }

    #[test]
    fn register_stdlib_runs_without_panic() {
        let (mut interner, mut symbols) = fresh();
        register_stdlib(&mut interner, &mut symbols);
    }

    #[test]
    fn println_is_registered() {
        let (mut interner, mut symbols) = fresh();
        register_stdlib(&mut interner, &mut symbols);
        assert!(symbols.lookup("println").is_some(), "println should be defined");
    }

    #[test]
    fn print_is_registered() {
        let (mut interner, mut symbols) = fresh();
        register_stdlib(&mut interner, &mut symbols);
        assert!(symbols.lookup("print").is_some(), "print should be defined");
    }

    #[test]
    fn math_sin_is_registered() {
        let (mut interner, mut symbols) = fresh();
        register_stdlib(&mut interner, &mut symbols);
        assert!(symbols.lookup("sin").is_some(), "sin should be defined");
    }

    #[test]
    fn math_pi_is_registered() {
        let (mut interner, mut symbols) = fresh();
        register_stdlib(&mut interner, &mut symbols);
        assert!(symbols.lookup("PI").is_some(), "PI should be defined");
    }

    #[test]
    fn display_trait_is_registered() {
        let (mut interner, mut symbols) = fresh();
        register_stdlib(&mut interner, &mut symbols);
        assert!(symbols.lookup("Display").is_some(), "Display trait should be defined");
    }

    #[test]
    fn vec_struct_is_registered() {
        let (mut interner, mut symbols) = fresh();
        register_stdlib(&mut interner, &mut symbols);
        assert!(symbols.lookup("Vec").is_some(), "Vec should be defined");
    }

    #[test]
    fn interner_grows_after_stdlib() {
        let (mut interner, mut symbols) = fresh();
        let before = interner.len();
        register_stdlib(&mut interner, &mut symbols);
        assert!(interner.len() > before, "interner should have new types after stdlib registration");
    }
}
