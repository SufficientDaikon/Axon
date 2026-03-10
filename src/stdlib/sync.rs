// stdlib/sync.rs — Synchronization primitives (Mutex, RwLock, Channel, Arc).

use crate::symbol::SymbolTable;
use crate::types::*;

use super::{def_method, def_struct};

/// Register synchronization types and their methods.
pub fn register_sync(interner: &mut TypeInterner, symbols: &mut SymbolTable) {
    register_mutex(interner, symbols);
    register_rwlock(interner, symbols);
    register_channel(interner, symbols);
    register_arc(interner, symbols);
    register_atomics(interner, symbols);
}

fn register_mutex(interner: &mut TypeInterner, symbols: &mut SymbolTable) {
    let mutex_ty = def_struct(symbols, interner, "Mutex", vec![], vec![]);

    def_method(symbols, interner, "Mutex", "new", vec![TypeId::UNIT], mutex_ty);
    def_method(symbols, interner, "Mutex", "lock", vec![mutex_ty], TypeId::UNIT);
    def_method(symbols, interner, "Mutex", "unlock", vec![mutex_ty], TypeId::UNIT);
    def_method(symbols, interner, "Mutex", "try_lock", vec![mutex_ty], TypeId::UNIT);
    def_method(symbols, interner, "Mutex", "is_locked", vec![mutex_ty], TypeId::BOOL);
}

fn register_rwlock(interner: &mut TypeInterner, symbols: &mut SymbolTable) {
    let rwlock_ty = def_struct(symbols, interner, "RwLock", vec![], vec![]);

    def_method(symbols, interner, "RwLock", "new", vec![TypeId::UNIT], rwlock_ty);
    def_method(symbols, interner, "RwLock", "read", vec![rwlock_ty], TypeId::UNIT);
    def_method(symbols, interner, "RwLock", "write", vec![rwlock_ty], TypeId::UNIT);
    def_method(symbols, interner, "RwLock", "try_read", vec![rwlock_ty], TypeId::UNIT);
    def_method(symbols, interner, "RwLock", "try_write", vec![rwlock_ty], TypeId::UNIT);
}

fn register_channel(interner: &mut TypeInterner, symbols: &mut SymbolTable) {
    let channel_ty = def_struct(symbols, interner, "Channel", vec![], vec![]);

    def_method(symbols, interner, "Channel", "new", vec![], channel_ty);
    def_method(symbols, interner, "Channel", "bounded", vec![TypeId::INT64], channel_ty);
    def_method(symbols, interner, "Channel", "send", vec![channel_ty, TypeId::UNIT], TypeId::UNIT);
    def_method(symbols, interner, "Channel", "recv", vec![channel_ty], TypeId::UNIT);
    def_method(symbols, interner, "Channel", "try_recv", vec![channel_ty], TypeId::UNIT);
    def_method(symbols, interner, "Channel", "try_send", vec![channel_ty, TypeId::UNIT], TypeId::BOOL);
    def_method(symbols, interner, "Channel", "is_empty", vec![channel_ty], TypeId::BOOL);
    def_method(symbols, interner, "Channel", "is_full", vec![channel_ty], TypeId::BOOL);
    def_method(symbols, interner, "Channel", "len", vec![channel_ty], TypeId::INT64);
    def_method(symbols, interner, "Channel", "capacity", vec![channel_ty], TypeId::INT64);
    def_method(symbols, interner, "Channel", "close", vec![channel_ty], TypeId::UNIT);
}

fn register_arc(interner: &mut TypeInterner, symbols: &mut SymbolTable) {
    let arc_ty = def_struct(symbols, interner, "Arc", vec![], vec![]);

    def_method(symbols, interner, "Arc", "new", vec![TypeId::UNIT], arc_ty);
    def_method(symbols, interner, "Arc", "clone", vec![arc_ty], arc_ty);
    def_method(symbols, interner, "Arc", "strong_count", vec![arc_ty], TypeId::INT64);
}

fn register_atomics(interner: &mut TypeInterner, symbols: &mut SymbolTable) {
    let atomic_ty = def_struct(symbols, interner, "AtomicI64", vec![], vec![]);

    def_method(symbols, interner, "AtomicI64", "new", vec![TypeId::INT64], atomic_ty);
    def_method(symbols, interner, "AtomicI64", "load", vec![atomic_ty], TypeId::INT64);
    def_method(symbols, interner, "AtomicI64", "store", vec![atomic_ty, TypeId::INT64], TypeId::UNIT);
    def_method(symbols, interner, "AtomicI64", "fetch_add", vec![atomic_ty, TypeId::INT64], TypeId::INT64);
    def_method(symbols, interner, "AtomicI64", "fetch_sub", vec![atomic_ty, TypeId::INT64], TypeId::INT64);
    def_method(symbols, interner, "AtomicI64", "compare_exchange", vec![atomic_ty, TypeId::INT64, TypeId::INT64], TypeId::BOOL);
    def_method(symbols, interner, "AtomicI64", "swap", vec![atomic_ty, TypeId::INT64], TypeId::INT64);

    // AtomicBool
    let abool_ty = def_struct(symbols, interner, "AtomicBool", vec![], vec![]);
    def_method(symbols, interner, "AtomicBool", "new", vec![TypeId::BOOL], abool_ty);
    def_method(symbols, interner, "AtomicBool", "load", vec![abool_ty], TypeId::BOOL);
    def_method(symbols, interner, "AtomicBool", "store", vec![abool_ty, TypeId::BOOL], TypeId::UNIT);
    def_method(symbols, interner, "AtomicBool", "swap", vec![abool_ty, TypeId::BOOL], TypeId::BOOL);

    // Condvar
    let condvar_ty = def_struct(symbols, interner, "Condvar", vec![], vec![]);
    def_method(symbols, interner, "Condvar", "new", vec![], condvar_ty);
    def_method(symbols, interner, "Condvar", "wait", vec![condvar_ty, TypeId::UNIT], TypeId::UNIT);
    def_method(symbols, interner, "Condvar", "notify_one", vec![condvar_ty], TypeId::UNIT);
    def_method(symbols, interner, "Condvar", "notify_all", vec![condvar_ty], TypeId::UNIT);

    // Barrier
    let barrier_ty = def_struct(symbols, interner, "Barrier", vec![], vec![]);
    def_method(symbols, interner, "Barrier", "new", vec![TypeId::INT64], barrier_ty);
    def_method(symbols, interner, "Barrier", "wait", vec![barrier_ty], TypeId::UNIT);
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
    fn mutex_registered() {
        let (mut i, mut s) = fresh();
        register_sync(&mut i, &mut s);
        assert!(s.lookup("Mutex").is_some());
        assert!(s.lookup("Mutex::new").is_some());
        assert!(s.lookup("Mutex::lock").is_some());
    }

    #[test]
    fn rwlock_registered() {
        let (mut i, mut s) = fresh();
        register_sync(&mut i, &mut s);
        assert!(s.lookup("RwLock").is_some());
        assert!(s.lookup("RwLock::read").is_some());
        assert!(s.lookup("RwLock::write").is_some());
    }

    #[test]
    fn channel_registered() {
        let (mut i, mut s) = fresh();
        register_sync(&mut i, &mut s);
        assert!(s.lookup("Channel").is_some());
        assert!(s.lookup("Channel::send").is_some());
        assert!(s.lookup("Channel::recv").is_some());
        assert!(s.lookup("Channel::bounded").is_some());
        assert!(s.lookup("Channel::close").is_some());
        assert!(s.lookup("Channel::try_send").is_some());
        assert!(s.lookup("Channel::len").is_some());
        assert!(s.lookup("Channel::capacity").is_some());
    }

    #[test]
    fn arc_registered() {
        let (mut i, mut s) = fresh();
        register_sync(&mut i, &mut s);
        assert!(s.lookup("Arc").is_some());
        assert!(s.lookup("Arc::new").is_some());
    }

    #[test]
    fn atomics_registered() {
        let (mut i, mut s) = fresh();
        register_sync(&mut i, &mut s);
        assert!(s.lookup("AtomicI64").is_some());
        assert!(s.lookup("AtomicI64::compare_exchange").is_some());
        assert!(s.lookup("AtomicBool").is_some());
        assert!(s.lookup("AtomicBool::load").is_some());
    }

    #[test]
    fn condvar_registered() {
        let (mut i, mut s) = fresh();
        register_sync(&mut i, &mut s);
        assert!(s.lookup("Condvar").is_some());
        assert!(s.lookup("Condvar::wait").is_some());
        assert!(s.lookup("Condvar::notify_one").is_some());
    }

    #[test]
    fn barrier_registered() {
        let (mut i, mut s) = fresh();
        register_sync(&mut i, &mut s);
        assert!(s.lookup("Barrier").is_some());
        assert!(s.lookup("Barrier::wait").is_some());
    }

    #[test]
    fn mutex_unlock_registered() {
        let (mut i, mut s) = fresh();
        register_sync(&mut i, &mut s);
        assert!(s.lookup("Mutex::unlock").is_some());
    }
}
