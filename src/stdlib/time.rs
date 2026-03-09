// stdlib/time.rs — Duration and Instant types.

use crate::symbol::SymbolTable;
use crate::types::*;

use super::{def_method, def_struct};

/// Register time-related types and methods.
pub fn register_time(interner: &mut TypeInterner, symbols: &mut SymbolTable) {
    register_duration(interner, symbols);
    register_instant(interner, symbols);
}

fn register_duration(interner: &mut TypeInterner, symbols: &mut SymbolTable) {
    let dur_ty = def_struct(symbols, interner, "Duration", vec![], vec![]);

    def_method(symbols, interner, "Duration", "from_secs", vec![TypeId::INT64], dur_ty);
    def_method(symbols, interner, "Duration", "from_millis", vec![TypeId::INT64], dur_ty);
    def_method(symbols, interner, "Duration", "from_micros", vec![TypeId::INT64], dur_ty);
    def_method(symbols, interner, "Duration", "from_nanos", vec![TypeId::INT64], dur_ty);
    def_method(symbols, interner, "Duration", "as_secs", vec![dur_ty], TypeId::INT64);
    def_method(symbols, interner, "Duration", "as_millis", vec![dur_ty], TypeId::INT64);
    def_method(symbols, interner, "Duration", "as_secs_f64", vec![dur_ty], TypeId::FLOAT64);
    def_method(symbols, interner, "Duration", "is_zero", vec![dur_ty], TypeId::BOOL);
}

fn register_instant(interner: &mut TypeInterner, symbols: &mut SymbolTable) {
    let inst_ty = def_struct(symbols, interner, "Instant", vec![], vec![]);
    let dur_sym = symbols.lookup("Duration");
    // Use Duration's type if available, otherwise fall back to UNIT
    let dur_ty = dur_sym
        .map(|id| symbols.get_symbol(id).ty)
        .unwrap_or(TypeId::UNIT);

    def_method(symbols, interner, "Instant", "now", vec![], inst_ty);
    def_method(symbols, interner, "Instant", "elapsed", vec![inst_ty], dur_ty);
    def_method(symbols, interner, "Instant", "duration_since", vec![inst_ty, inst_ty], dur_ty);
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
    fn duration_registered() {
        let (mut i, mut s) = fresh();
        register_time(&mut i, &mut s);
        assert!(s.lookup("Duration").is_some());
        assert!(s.lookup("Duration::from_secs").is_some());
        assert!(s.lookup("Duration::from_millis").is_some());
    }

    #[test]
    fn instant_registered() {
        let (mut i, mut s) = fresh();
        register_time(&mut i, &mut s);
        assert!(s.lookup("Instant").is_some());
        assert!(s.lookup("Instant::now").is_some());
        assert!(s.lookup("Instant::elapsed").is_some());
    }

    #[test]
    fn duration_as_secs_returns_int64() {
        let (mut i, mut s) = fresh();
        register_time(&mut i, &mut s);
        let sym_id = s.lookup("Duration::as_secs").unwrap();
        let sym = s.get_symbol(sym_id);
        let ty = i.resolve(sym.ty);
        match ty {
            Type::Function { ret, .. } => assert_eq!(*ret, TypeId::INT64),
            _ => panic!("Duration::as_secs should be a function"),
        }
    }

    #[test]
    fn instant_elapsed_returns_duration_type() {
        let (mut i, mut s) = fresh();
        register_time(&mut i, &mut s);
        let elapsed_sym = s.lookup("Instant::elapsed").unwrap();
        let elapsed_info = s.get_symbol(elapsed_sym);
        let ty = i.resolve(elapsed_info.ty);
        match ty {
            Type::Function { ret, .. } => {
                // The return type should be Duration (a Struct type)
                assert!(
                    matches!(i.resolve(*ret), Type::Struct { name, .. } if name == "Duration"),
                    "Instant::elapsed should return Duration"
                );
            }
            _ => panic!("Instant::elapsed should be a function"),
        }
    }

    #[test]
    fn duration_from_nanos_registered() {
        let (mut i, mut s) = fresh();
        register_time(&mut i, &mut s);
        assert!(s.lookup("Duration::from_nanos").is_some());
    }
}
