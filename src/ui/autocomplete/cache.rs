//! Per-directory cache for file discovery results
//!
//! Caches discovered files keyed by directory path to avoid repeated scans.
//! Cache entries expire after a configurable TTL.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

/// Cache entry containing discovered files and timestamp
#[derive(Debug, Clone)]
struct CacheEntry {
    files: Vec<PathBuf>,
    timestamp: Instant,
}

/// Per-directory cache for file discovery
#[derive(Debug)]
pub struct PerDirectoryCache {
    entries: HashMap<PathBuf, CacheEntry>,
    ttl: Duration,
}

impl PerDirectoryCache {
    /// Create a new cache with default TTL
    pub fn new() -> Self {
        Self::with_ttl(Duration::from_secs(super::CACHE_TTL_SECONDS))
    }

    /// Create a new cache with custom TTL
    pub fn with_ttl(ttl: Duration) -> Self {
        Self {
            entries: HashMap::new(),
            ttl,
        }
    }

    /// Get cached files for a directory
    ///
    /// Returns None if directory not cached or cache expired
    pub fn get(&self, dir: &Path) -> Option<&[PathBuf]> {
        let entry = self.entries.get(dir)?;

        if self.is_expired(entry) {
            return None;
        }

        Some(&entry.files)
    }

    /// Store files for a directory
    pub fn put(&mut self, dir: PathBuf, files: Vec<PathBuf>) {
        let entry = CacheEntry {
            files,
            timestamp: Instant::now(),
        };
        self.entries.insert(dir, entry);
    }

    /// Invalidate cache entry for a directory
    pub fn invalidate(&mut self, dir: &Path) {
        self.entries.remove(dir);
    }

    /// Check if a cache entry is expired
    fn is_expired(&self, entry: &CacheEntry) -> bool {
        entry.timestamp.elapsed() > self.ttl
    }

    /// Get the number of cached directories
    #[cfg(test)]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if cache is empty
    #[cfg(test)]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

impl Default for PerDirectoryCache {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_cache_hit() {
        let mut cache = PerDirectoryCache::new();
        let dir = PathBuf::from("/test/dir");
        let files = vec![PathBuf::from("/test/dir/file.pdf")];

        cache.put(dir.clone(), files.clone());

        let result = cache.get(&dir);
        assert!(result.is_some());
        assert_eq!(result.unwrap(), &files);
    }

    #[test]
    fn test_cache_miss() {
        let cache = PerDirectoryCache::new();
        let dir = PathBuf::from("/nonexistent/dir");

        assert!(cache.get(&dir).is_none());
    }

    #[test]
    fn test_cache_expiration() {
        let mut cache = PerDirectoryCache::with_ttl(Duration::from_millis(50));
        let dir = PathBuf::from("/test/dir");
        let files = vec![PathBuf::from("/test/dir/file.pdf")];

        cache.put(dir.clone(), files);

        // Should be available immediately
        assert!(cache.get(&dir).is_some());

        // Wait for expiration
        thread::sleep(Duration::from_millis(60));

        // Should be expired now
        assert!(cache.get(&dir).is_none());
    }

    #[test]
    fn test_cache_invalidation() {
        let mut cache = PerDirectoryCache::new();
        let dir = PathBuf::from("/test/dir");
        let files = vec![PathBuf::from("/test/dir/file.pdf")];

        cache.put(dir.clone(), files);
        assert!(cache.get(&dir).is_some());

        cache.invalidate(&dir);
        assert!(cache.get(&dir).is_none());
    }

    #[test]
    fn test_per_directory_isolation() {
        let mut cache = PerDirectoryCache::new();
        let dir1 = PathBuf::from("/test/dir1");
        let dir2 = PathBuf::from("/test/dir2");
        let files1 = vec![PathBuf::from("/test/dir1/file1.pdf")];
        let files2 = vec![PathBuf::from("/test/dir2/file2.pdf")];

        cache.put(dir1.clone(), files1.clone());
        cache.put(dir2.clone(), files2.clone());

        assert_eq!(cache.get(&dir1).unwrap(), &files1);
        assert_eq!(cache.get(&dir2).unwrap(), &files2);

        cache.invalidate(&dir1);
        assert!(cache.get(&dir1).is_none());
        assert!(cache.get(&dir2).is_some());
    }
}
