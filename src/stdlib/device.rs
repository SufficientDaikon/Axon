// stdlib/device.rs — Device enum (Cpu, Gpu, Tpu) and methods.

use crate::symbol::SymbolTable;
use crate::types::*;

use super::{def_enum, def_method};

/// Register the Device enum and its methods.
pub fn register_device(interner: &mut TypeInterner, symbols: &mut SymbolTable) {
    let device_ty = def_enum(
        symbols,
        interner,
        "Device",
        vec![
            ("Cpu".into(), EnumVariantType::Unit),
            ("Gpu".into(), EnumVariantType::Tuple(vec![TypeId::INT32])),
            ("Tpu".into(), EnumVariantType::Tuple(vec![TypeId::INT32])),
        ],
        vec![],
    );

    def_method(symbols, interner, "Device", "is_available", vec![device_ty], TypeId::BOOL);
    def_method(symbols, interner, "Device", "count", vec![device_ty], TypeId::INT64);
    def_method(symbols, interner, "Device", "current", vec![], device_ty);
    def_method(symbols, interner, "Device", "set_default", vec![device_ty], TypeId::UNIT);
    def_method(symbols, interner, "Device", "name", vec![device_ty], TypeId::STRING);
    def_method(symbols, interner, "Device", "memory_total", vec![device_ty], TypeId::INT64);
    def_method(symbols, interner, "Device", "memory_free", vec![device_ty], TypeId::INT64);
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
    fn device_enum_registered() {
        let (mut i, mut s) = fresh();
        register_device(&mut i, &mut s);
        let sym_id = s.lookup("Device").unwrap();
        let sym = s.get_symbol(sym_id);
        assert!(matches!(i.resolve(sym.ty), Type::Enum { name, .. } if name == "Device"));
    }

    #[test]
    fn device_methods_registered() {
        let (mut i, mut s) = fresh();
        register_device(&mut i, &mut s);
        assert!(s.lookup("Device::is_available").is_some());
        assert!(s.lookup("Device::count").is_some());
        assert!(s.lookup("Device::current").is_some());
        assert!(s.lookup("Device::set_default").is_some());
    }

    #[test]
    fn device_has_gpu_variant() {
        let (mut i, mut s) = fresh();
        register_device(&mut i, &mut s);
        let sym_id = s.lookup("Device").unwrap();
        let sym = s.get_symbol(sym_id);
        match i.resolve(sym.ty) {
            Type::Enum { variants, .. } => {
                let names: Vec<&str> = variants.iter().map(|(n, _)| n.as_str()).collect();
                assert!(names.contains(&"Gpu"));
                assert!(names.contains(&"Cpu"));
                assert!(names.contains(&"Tpu"));
            }
            _ => panic!("Device should be an enum"),
        }
    }

    #[test]
    fn device_memory_methods() {
        let (mut i, mut s) = fresh();
        register_device(&mut i, &mut s);
        assert!(s.lookup("Device::memory_total").is_some());
        assert!(s.lookup("Device::memory_free").is_some());
    }
}
