// stdlib/thread.rs — Thread spawning and JoinHandle.

use crate::symbol::SymbolTable;
use crate::types::*;

use super::{def_fn, def_method, def_struct};

/// Register threading types and functions.
pub fn register_thread(interner: &mut TypeInterner, symbols: &mut SymbolTable) {
    let join_handle_ty = def_struct(symbols, interner, "JoinHandle", vec![], vec![]);

    // spawn(f: fn() -> T) -> JoinHandle<T>
    let closure_ty = interner.intern(Type::Function {
        params: vec![],
        ret: TypeId::UNIT,
    });
    def_fn(symbols, interner, "spawn", vec![closure_ty], join_handle_ty);

    // JoinHandle::join(self) -> Result<T, E>
    let result_ty = interner.intern(Type::Result(TypeId::UNIT, TypeId::STRING));
    def_method(symbols, interner, "JoinHandle", "join", vec![join_handle_ty], result_ty);
    def_method(symbols, interner, "JoinHandle", "is_finished", vec![join_handle_ty], TypeId::BOOL);

    // Thread utility functions
    def_fn(symbols, interner, "sleep", vec![TypeId::INT64], TypeId::UNIT);
    def_fn(symbols, interner, "yield_now", vec![], TypeId::UNIT);
    def_fn(symbols, interner, "current_thread_id", vec![], TypeId::INT64);
    def_fn(symbols, interner, "available_parallelism", vec![], TypeId::INT64);
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
    fn spawn_registered() {
        let (mut i, mut s) = fresh();
        register_thread(&mut i, &mut s);
        assert!(s.lookup("spawn").is_some());
    }

    #[test]
    fn join_handle_registered() {
        let (mut i, mut s) = fresh();
        register_thread(&mut i, &mut s);
        assert!(s.lookup("JoinHandle").is_some());
        assert!(s.lookup("JoinHandle::join").is_some());
    }

    #[test]
    fn sleep_registered() {
        let (mut i, mut s) = fresh();
        register_thread(&mut i, &mut s);
        assert!(s.lookup("sleep").is_some());
    }

    #[test]
    fn spawn_takes_function_param() {
        let (mut i, mut s) = fresh();
        register_thread(&mut i, &mut s);
        let sym_id = s.lookup("spawn").unwrap();
        let sym = s.get_symbol(sym_id);
        let ty = i.resolve(sym.ty);
        match ty {
            Type::Function { params, .. } => {
                assert_eq!(params.len(), 1);
                assert!(matches!(i.resolve(params[0]), Type::Function { .. }));
            }
            _ => panic!("spawn should be a function"),
        }
    }

    #[test]
    fn utility_functions_registered() {
        let (mut i, mut s) = fresh();
        register_thread(&mut i, &mut s);
        assert!(s.lookup("yield_now").is_some());
        assert!(s.lookup("available_parallelism").is_some());
    }
}
