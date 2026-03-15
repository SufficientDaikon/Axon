// interner.rs — Global name interning for O(1) name comparisons
//
// Provides a `NameInterner` that deduplicates strings and returns
// lightweight `Name` handles. This is the infrastructure for future
// migration of AST/TAST/MIR identifiers from String to Name.

use serde::{Serialize, Deserialize};
use std::collections::HashMap;

/// A lightweight handle to an interned string.
/// Two `Name` values from the same interner are equal iff they
/// refer to the same string.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Name(pub u32);

/// Interns strings so that equal strings always map to the same `Name`.
pub struct NameInterner {
    map: HashMap<String, Name>,
    names: Vec<String>,
}

impl NameInterner {
    pub fn new() -> Self {
        NameInterner {
            map: HashMap::new(),
            names: Vec::new(),
        }
    }

    /// Intern a string, returning its `Name` handle.
    /// If the string was already interned, returns the existing handle.
    pub fn intern(&mut self, s: &str) -> Name {
        if let Some(&name) = self.map.get(s) {
            return name;
        }
        let id = Name(self.names.len() as u32);
        self.names.push(s.to_string());
        self.map.insert(s.to_string(), id);
        id
    }

    /// Resolve a `Name` handle back to its string.
    pub fn resolve(&self, name: Name) -> &str {
        &self.names[name.0 as usize]
    }

    /// Check if a string has already been interned.
    pub fn contains(&self, s: &str) -> bool {
        self.map.contains_key(s)
    }

    /// Return the number of interned strings.
    pub fn len(&self) -> usize {
        self.names.len()
    }

    /// Check if the interner is empty.
    pub fn is_empty(&self) -> bool {
        self.names.is_empty()
    }
}

impl Default for NameInterner {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for NameInterner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NameInterner")
            .field("count", &self.names.len())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_intern_returns_same_id() {
        let mut interner = NameInterner::new();
        let a = interner.intern("hello");
        let b = interner.intern("hello");
        assert_eq!(a, b);
    }

    #[test]
    fn test_different_strings_different_ids() {
        let mut interner = NameInterner::new();
        let a = interner.intern("hello");
        let b = interner.intern("world");
        assert_ne!(a, b);
    }

    #[test]
    fn test_resolve_roundtrip() {
        let mut interner = NameInterner::new();
        let name = interner.intern("test_string");
        assert_eq!(interner.resolve(name), "test_string");
    }

    #[test]
    fn test_empty_string() {
        let mut interner = NameInterner::new();
        let name = interner.intern("");
        assert_eq!(interner.resolve(name), "");
    }

    #[test]
    fn test_contains() {
        let mut interner = NameInterner::new();
        assert!(!interner.contains("hello"));
        interner.intern("hello");
        assert!(interner.contains("hello"));
        assert!(!interner.contains("world"));
    }

    #[test]
    fn test_len_and_is_empty() {
        let mut interner = NameInterner::new();
        assert!(interner.is_empty());
        assert_eq!(interner.len(), 0);
        interner.intern("a");
        interner.intern("b");
        interner.intern("a"); // duplicate, shouldn't increase len
        assert_eq!(interner.len(), 2);
        assert!(!interner.is_empty());
    }

    #[test]
    fn test_many_names() {
        let mut interner = NameInterner::new();
        let mut names = Vec::new();
        for i in 0..10_000 {
            names.push(interner.intern(&format!("name_{}", i)));
        }
        assert_eq!(interner.len(), 10_000);
        // Verify all unique
        let unique: std::collections::HashSet<Name> = names.iter().copied().collect();
        assert_eq!(unique.len(), 10_000);
        // Verify roundtrip
        assert_eq!(interner.resolve(names[0]), "name_0");
        assert_eq!(interner.resolve(names[9999]), "name_9999");
    }
}
