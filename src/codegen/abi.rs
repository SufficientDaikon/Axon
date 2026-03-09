use serde::Serialize;

/// Mangling scheme: _AX{hash}N{ns_len}{ns}F{fn_len}{fn}G{generic_args}
/// Example: std::math::sin<Float32> → _AX7d3f4aN4mathF3sinGF32

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum CallingConv {
    AxonDefault,
    C,
}

pub struct NameMangler;

impl NameMangler {
    /// Mangle an Axon function name into a linker symbol.
    /// For `main`, always returns `main` (entry point).
    /// For other functions, applies Axon mangling scheme.
    pub fn mangle(namespace: &[String], name: &str, generic_args: &[String]) -> String {
        if name == "main" && namespace.is_empty() {
            return "main".to_string();
        }

        let mut mangled = String::from("_AX");

        // Hash of full path (first 8 hex chars of a simple hash)
        let full_path = format!("{}::{}", namespace.join("::"), name);
        let hash = simple_hash(&full_path);
        mangled.push_str(&format!("{:08x}", hash));

        // Namespace components
        for ns in namespace {
            mangled.push('N');
            mangled.push_str(&format!("{}", ns.len()));
            mangled.push_str(ns);
        }

        // Function name
        mangled.push('F');
        mangled.push_str(&format!("{}", name.len()));
        mangled.push_str(name);

        // Generic arguments
        if !generic_args.is_empty() {
            mangled.push('G');
            for arg in generic_args {
                mangled.push_str(&format!("{}{}", arg.len(), arg));
            }
        }

        mangled
    }

    /// Demangle a symbol back to human-readable form.
    pub fn demangle(symbol: &str) -> Option<String> {
        if !symbol.starts_with("_AX") {
            return None;
        }
        Some(symbol.to_string()) // placeholder
    }

    /// Mangle a struct/enum type name for LLVM struct types.
    pub fn mangle_type(namespace: &[String], name: &str) -> String {
        if namespace.is_empty() {
            format!("axon.{}", name)
        } else {
            format!("axon.{}.{}", namespace.join("."), name)
        }
    }
}

fn simple_hash(s: &str) -> u32 {
    let mut hash: u32 = 5381;
    for byte in s.bytes() {
        hash = hash.wrapping_mul(33).wrapping_add(byte as u32);
    }
    hash
}

/// Describes how a type should be passed across function boundaries.
#[derive(Debug, Clone, PartialEq)]
pub enum PassMode {
    /// Pass directly in a register (scalars, small tuples)
    Direct,
    /// Pass by pointer (large structs, arrays)
    Indirect,
    /// Ignore (unit type, zero-sized types)
    Ignore,
}

/// Determine how to pass a type at the ABI level.
pub fn pass_mode_for_type(ty: &crate::types::Type) -> PassMode {
    use crate::types::Type;
    match ty {
        Type::Unit | Type::Never => PassMode::Ignore,
        Type::Primitive(_) => PassMode::Direct,
        Type::Reference { .. } => PassMode::Direct,
        Type::Function { .. } => PassMode::Direct,
        Type::Tuple(fields) if fields.len() <= 2 => PassMode::Direct,
        Type::Tuple(_) => PassMode::Indirect,
        Type::Struct { fields, .. } if fields.len() <= 2 => PassMode::Direct,
        Type::Struct { .. } => PassMode::Indirect,
        Type::Enum { .. } => PassMode::Indirect,
        Type::Tensor(_) => PassMode::Indirect,
        Type::Array { .. } => PassMode::Indirect,
        _ => PassMode::Direct,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mangle_main_returns_main() {
        let result = NameMangler::mangle(&[], "main", &[]);
        assert_eq!(result, "main");
    }

    #[test]
    fn mangle_with_namespace() {
        let ns = vec!["std".to_string(), "math".to_string()];
        let result = NameMangler::mangle(&ns, "sin", &[]);
        assert!(result.starts_with("_AX"));
        assert!(result.contains("N3std"));
        assert!(result.contains("N4math"));
        assert!(result.contains("F3sin"));
        assert!(!result.contains('G'));
    }

    #[test]
    fn mangle_with_generics() {
        let ns = vec!["std".to_string()];
        let generics = vec!["Float32".to_string()];
        let result = NameMangler::mangle(&ns, "add", &generics);
        assert!(result.starts_with("_AX"));
        assert!(result.contains("N3std"));
        assert!(result.contains("F3add"));
        assert!(result.contains("G7Float32"));
    }

    #[test]
    fn mangle_type_no_namespace() {
        let result = NameMangler::mangle_type(&[], "Vec");
        assert_eq!(result, "axon.Vec");
    }

    #[test]
    fn mangle_type_with_namespace() {
        let ns = vec!["std".to_string(), "collections".to_string()];
        let result = NameMangler::mangle_type(&ns, "HashMap");
        assert_eq!(result, "axon.std.collections.HashMap");
    }

    #[test]
    fn pass_mode_unit_and_never() {
        assert_eq!(pass_mode_for_type(&crate::types::Type::Unit), PassMode::Ignore);
        assert_eq!(pass_mode_for_type(&crate::types::Type::Never), PassMode::Ignore);
    }

    #[test]
    fn pass_mode_primitive_direct() {
        let ty = crate::types::Type::Primitive(crate::types::PrimKind::Int32);
        assert_eq!(pass_mode_for_type(&ty), PassMode::Direct);
    }

    #[test]
    fn pass_mode_small_tuple_direct() {
        let ty = crate::types::Type::Tuple(vec![crate::types::TypeId::INT32]);
        assert_eq!(pass_mode_for_type(&ty), PassMode::Direct);
    }

    #[test]
    fn pass_mode_large_tuple_indirect() {
        let ty = crate::types::Type::Tuple(vec![
            crate::types::TypeId::INT32,
            crate::types::TypeId::INT32,
            crate::types::TypeId::INT32,
        ]);
        assert_eq!(pass_mode_for_type(&ty), PassMode::Indirect);
    }

    #[test]
    fn pass_mode_array_indirect() {
        let ty = crate::types::Type::Array {
            elem: crate::types::TypeId::INT32,
            size: 10,
        };
        assert_eq!(pass_mode_for_type(&ty), PassMode::Indirect);
    }

    #[test]
    fn simple_hash_consistency() {
        let h1 = simple_hash("hello");
        let h2 = simple_hash("hello");
        assert_eq!(h1, h2);

        let h3 = simple_hash("world");
        assert_ne!(h1, h3);
    }

    #[test]
    fn demangle_non_axon_returns_none() {
        assert_eq!(NameMangler::demangle("_ZN3foo3barE"), None);
    }

    #[test]
    fn demangle_axon_returns_some() {
        assert!(NameMangler::demangle("_AX12345678F3foo").is_some());
    }
}
