// cache.rs — Content hashing for incremental compilation / LSP cache invalidation

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// A content-based hash of a source file's text.
/// Two `ContentHash` values are equal iff the source texts are equal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ContentHash(u64);

impl ContentHash {
    /// Compute the content hash of a source string.
    pub fn of(source: &str) -> Self {
        let mut hasher = DefaultHasher::new();
        source.hash(&mut hasher);
        ContentHash(hasher.finish())
    }

    /// Return the raw u64 value (for serialization / debugging).
    pub fn raw(&self) -> u64 {
        self.0
    }
}

/// Per-file cache entry storing the hash and whether the last compilation succeeded.
#[derive(Debug, Clone)]
pub struct CacheEntry {
    pub hash: ContentHash,
    pub had_errors: bool,
}

/// A simple cache mapping filenames to their last-known content hash.
/// Used by the LSP to skip re-analysis when the source hasn't changed.
#[derive(Debug, Default)]
pub struct CompilationCache {
    entries: std::collections::HashMap<String, CacheEntry>,
}

impl CompilationCache {
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns `true` if the source has changed since the last check.
    /// Also returns `true` if the file has never been seen.
    pub fn is_dirty(&self, filename: &str, source: &str) -> bool {
        let new_hash = ContentHash::of(source);
        match self.entries.get(filename) {
            Some(entry) => entry.hash != new_hash,
            None => true,
        }
    }

    /// Record that `filename` was compiled from `source`, with the given error status.
    pub fn update(&mut self, filename: &str, source: &str, had_errors: bool) {
        self.entries.insert(filename.to_string(), CacheEntry {
            hash: ContentHash::of(source),
            had_errors,
        });
    }

    /// Check if the last compilation of `filename` had errors.
    pub fn had_errors(&self, filename: &str) -> bool {
        self.entries.get(filename).map_or(false, |e| e.had_errors)
    }

    /// Remove an entry from the cache (e.g. when a file is deleted).
    pub fn invalidate(&mut self, filename: &str) {
        self.entries.remove(filename);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_content_hash_equality() {
        let h1 = ContentHash::of("fn main() {}");
        let h2 = ContentHash::of("fn main() {}");
        assert_eq!(h1, h2);
    }

    #[test]
    fn test_content_hash_inequality() {
        let h1 = ContentHash::of("fn main() {}");
        let h2 = ContentHash::of("fn main() { 1 }");
        assert_ne!(h1, h2);
    }

    #[test]
    fn test_cache_dirty_check() {
        let mut cache = CompilationCache::new();
        assert!(cache.is_dirty("test.axon", "fn main() {}"));

        cache.update("test.axon", "fn main() {}", false);
        assert!(!cache.is_dirty("test.axon", "fn main() {}"));

        assert!(cache.is_dirty("test.axon", "fn main() { 1 }"));
    }

    #[test]
    fn test_cache_error_tracking() {
        let mut cache = CompilationCache::new();
        assert!(!cache.had_errors("test.axon"));

        cache.update("test.axon", "fn main() {}", true);
        assert!(cache.had_errors("test.axon"));

        cache.update("test.axon", "fn main() {}", false);
        assert!(!cache.had_errors("test.axon"));
    }

    #[test]
    fn test_cache_invalidate() {
        let mut cache = CompilationCache::new();
        cache.update("test.axon", "fn main() {}", false);
        assert!(!cache.is_dirty("test.axon", "fn main() {}"));

        cache.invalidate("test.axon");
        assert!(cache.is_dirty("test.axon", "fn main() {}"));
    }
}
