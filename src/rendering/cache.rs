//! Word-Level LRU Cache for rendered word buffers
//!
//! Provides caching of pre-rendered RGBA word buffers to eliminate redundant
//! rasterization and enable consistent 1000+ WPM reading speeds.

use crate::engine::config::{DEFAULT_FONT_SIZE, DEFAULT_MEMORY_CAP_BYTES};
use imageproc::image::ImageBuffer;
use imageproc::image::Rgba;
use lru::LruCache;
use std::hash::{Hash, Hasher};
use std::num::NonZeroUsize;

/// Cache key for word rendering lookups
///
/// Uses tuple-based keys instead of string formatting to avoid
/// allocation overhead on every cache lookup.
#[derive(Debug, Clone, PartialEq)]
pub struct CacheKey {
    /// The word text content
    pub word: String,
    /// Font size in pixels
    pub font_size: f32,
    /// Anchor position (character index for OVP highlighting)
    pub anchor_position: usize,
}

// Manual Eq implementation - safe because font_size should never be NaN in practice
impl Eq for CacheKey {}

impl Hash for CacheKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // Hash word content
        self.word.hash(state);
        // Hash font size as bits to avoid float precision issues
        self.font_size.to_bits().hash(state);
        // Hash anchor position
        self.anchor_position.hash(state);
    }
}

/// Cached word with pre-rendered RGBA buffer
#[derive(Debug, Clone)]
pub struct CachedWord {
    /// Pre-rendered RGBA buffer
    pub buffer: ImageBuffer<Rgba<u8>, Vec<u8>>,
    /// Image width in pixels
    pub width: u32,
    /// Image height in pixels
    pub height: u32,
}

impl CachedWord {
    /// Calculate memory size of this cached word in bytes
    pub fn memory_size_bytes(&self) -> u64 {
        // width * height * 4 bytes per pixel (RGBA)
        (self.width as u64) * (self.height as u64) * 4
    }
}

/// Error type for cache operations
#[derive(Debug, Clone, PartialEq)]
pub enum CacheError {
    /// Failed to rasterize word
    RasterizationFailed(String),
}

impl std::fmt::Display for CacheError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CacheError::RasterizationFailed(msg) => write!(f, "Rasterization failed: {}", msg),
        }
    }
}

impl std::error::Error for CacheError {}

/// Word-Level LRU Cache for rendered word buffers
///
/// Provides O(1) cache lookups for pre-rendered word buffers,
/// with memory tracking and hit/miss statistics.
pub struct WordCache {
    /// LRU cache storage
    cache: LruCache<CacheKey, CachedWord>,
    /// Current font size (for key generation)
    font_size: f32,
    /// Hit counter for telemetry
    hits: u64,
    /// Miss counter for telemetry
    misses: u64,
    /// Total memory used by cached entries (bytes)
    total_cached_bytes: u64,
    /// Memory cap in bytes
    memory_cap_bytes: u64,
}

impl WordCache {
    /// Create a new WordCache with specified capacity and default memory cap
    ///
    /// # Arguments
    /// * `capacity` - Maximum number of cached entries
    ///
    /// # Example
    /// ```ignore
    /// use speedy::rendering::cache::WordCache;
    /// let cache = WordCache::new(1000);
    /// ```
    pub fn new(capacity: usize) -> Self {
        // SAFETY: 100 is guaranteed non-zero, so this unwrap is safe
        let default_capacity = NonZeroUsize::new(100).expect("100 is guaranteed non-zero");
        let capacity = NonZeroUsize::new(capacity).unwrap_or(default_capacity);

        Self {
            cache: LruCache::new(capacity),
            font_size: DEFAULT_FONT_SIZE,
            hits: 0,
            misses: 0,
            total_cached_bytes: 0,
            memory_cap_bytes: DEFAULT_MEMORY_CAP_BYTES,
        }
    }

    /// Get or render a word, returning the cached entry
    ///
    /// On cache hit: Returns cached entry, increments hit counter (O(1))
    /// On cache miss: Renders word, stores in cache, increments miss counter
    ///
    /// # Arguments
    /// * `word` - The word to render
    /// * `anchor_position` - Character index for OVP highlighting
    /// * `font` - Font reference for rasterization
    /// * `metrics` - Font metrics for sizing
    ///
    /// # Returns
    /// Cached word buffer (cloned from cache)
    pub fn get_or_render<F>(
        &mut self,
        word: &str,
        anchor_position: usize,
        render: F,
    ) -> Result<CachedWord, CacheError>
    where
        F: FnOnce() -> Option<ImageBuffer<Rgba<u8>, Vec<u8>>>,
    {
        // Check memory cap and evict if needed
        self.enforce_memory_cap();

        // Construct cache key
        let key = CacheKey {
            word: word.to_string(),
            font_size: self.font_size,
            anchor_position,
        };

        // Try cache lookup
        if let Some(cached) = self.cache.get(&key) {
            self.hits += 1;
            let cached_word: CachedWord = cached.clone();
            return Ok(cached_word);
        }

        // Cache miss - invoke caller's rasterizer
        self.misses += 1;

        let image = render().ok_or_else(|| {
            CacheError::RasterizationFailed(format!("Failed to rasterize word: {}", word))
        })?;

        let cached_word = CachedWord {
            width: image.width(),
            height: image.height(),
            buffer: image,
        };

        // Check if we're at capacity before inserting
        // The lru crate doesn't reliably return evicted items from put(),
        // so we need to manually handle capacity enforcement
        if self.cache.len() >= self.cache.cap().get() {
            // We're at capacity, pop the LRU item first
            if let Some((_, evicted_word)) = self.cache.pop_lru() {
                self.total_cached_bytes -= evicted_word.memory_size_bytes();
            }
        }

        // Update memory tracking for the new item
        self.total_cached_bytes += cached_word.memory_size_bytes();

        // Store in cache
        self.cache.put(key, cached_word.clone());

        Ok(cached_word)
    }

    /// Clear the cache and reset statistics
    pub fn clear(&mut self) {
        self.cache.clear();
        self.hits = 0;
        self.misses = 0;
        self.total_cached_bytes = 0;
    }

    /// Set font size and clear cache (prevents stale entries)
    ///
    /// Font size affects rendered appearance, so cached entries
    /// from different font sizes are not reusable.
    pub fn set_font_size(&mut self, font_size: f32) {
        if (font_size - self.font_size).abs() > f32::EPSILON {
            self.font_size = font_size;
            self.clear();
        }
    }

    /// Get cache hit rate (0.0 to 1.0)
    ///
    #[cfg(test)]
    /// Returns hits / (hits + misses), or 0.0 if no lookups performed
    pub fn get_hit_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 {
            0.0
        } else {
            self.hits as f64 / total as f64
        }
    }

    #[cfg(test)]
    /// Get memory usage in megabytes
    pub fn get_memory_usage_mb(&self) -> f64 {
        self.total_cached_bytes as f64 / (1024.0 * 1024.0)
    }

    #[cfg(test)]
    /// Get total cached bytes
    pub fn total_cached_bytes(&self) -> u64 {
        self.total_cached_bytes
    }

    #[cfg(test)]
    /// Get number of cache hits
    pub fn hits(&self) -> u64 {
        self.hits
    }

    #[cfg(test)]
    /// Get number of cache misses
    pub fn misses(&self) -> u64 {
        self.misses
    }

    #[cfg(test)]
    /// Get current number of cached entries
    pub fn len(&self) -> usize {
        self.cache.len()
    }

    /// Check if cache is empty
    pub fn is_empty(&self) -> bool {
        self.cache.is_empty()
    }

    /// Enforce memory cap by evicting entries if needed
    fn enforce_memory_cap(&mut self) {
        while self.total_cached_bytes >= self.memory_cap_bytes && !self.is_empty() {
            // Evict least-recently-used entry
            if let Some((_, evicted)) = self.cache.pop_lru() {
                let evicted_word: CachedWord = evicted;
                self.total_cached_bytes -= evicted_word.memory_size_bytes();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_key_hashing() {
        let key1 = CacheKey {
            word: "hello".to_string(),
            font_size: 24.0,
            anchor_position: 1,
        };

        let key2 = CacheKey {
            word: "hello".to_string(),
            font_size: 24.0,
            anchor_position: 1,
        };

        // Same keys should hash to same value
        let mut hasher1 = std::collections::hash_map::DefaultHasher::new();
        let mut hasher2 = std::collections::hash_map::DefaultHasher::new();
        key1.hash(&mut hasher1);
        key2.hash(&mut hasher2);

        assert_eq!(hasher1.finish(), hasher2.finish());
    }

    #[test]
    fn test_cache_key_different_words() {
        let key1 = CacheKey {
            word: "hello".to_string(),
            font_size: 24.0,
            anchor_position: 1,
        };

        let key2 = CacheKey {
            word: "world".to_string(),
            font_size: 24.0,
            anchor_position: 1,
        };

        // Different words should produce different hashes
        let mut hasher1 = std::collections::hash_map::DefaultHasher::new();
        let mut hasher2 = std::collections::hash_map::DefaultHasher::new();
        key1.hash(&mut hasher1);
        key2.hash(&mut hasher2);

        assert_ne!(hasher1.finish(), hasher2.finish());
    }

    #[test]
    fn test_cache_creation() {
        let cache = WordCache::new(100);

        assert_eq!(cache.len(), 0);
        assert!(cache.is_empty());
        assert_eq!(cache.hits(), 0);
        assert_eq!(cache.misses(), 0);
        assert_eq!(cache.get_hit_rate(), 0.0);
    }

    #[test]
    fn test_cache_clear() {
        let mut cache = WordCache::new(10);
        cache.hits = 5;
        cache.misses = 3;
        cache.total_cached_bytes = 1000;

        cache.clear();

        assert_eq!(cache.hits(), 0);
        assert_eq!(cache.misses(), 0);
        assert_eq!(cache.total_cached_bytes(), 0);
    }

    #[test]
    fn test_cached_word_memory_size() {
        let buffer = ImageBuffer::from_pixel(100, 50, Rgba([0, 0, 0, 255]));
        let cached = CachedWord {
            buffer,
            width: 100,
            height: 50,
        };

        // 100 * 50 * 4 bytes = 20,000 bytes
        assert_eq!(cached.memory_size_bytes(), 20_000);
    }

    #[test]
    fn test_get_hit_rate() {
        let mut cache = WordCache::new(100);

        // No lookups yet
        assert_eq!(cache.get_hit_rate(), 0.0);

        cache.hits = 70;
        cache.misses = 30;

        // 70 / 100 = 0.7
        assert!((cache.get_hit_rate() - 0.7).abs() < f64::EPSILON);
    }

    #[test]
    fn test_memory_usage_mb() {
        let mut cache = WordCache::new(100);
        cache.total_cached_bytes = 75 * 1024 * 1024; // 75MB

        assert!((cache.get_memory_usage_mb() - 75.0).abs() < 0.01);
    }

    #[test]
    fn test_memory_accounting_with_eviction() {
        // This test verifies the HIGH priority fix: when put() evicts an item
        // due to capacity limits, the evicted item's size is properly subtracted
        // ponytail: dummy factory instead of real rasterizer — keeps the cache cell
        // decoupled from kitty in tests (would otherwise create a false import cycle).
        fn dummy_image() -> ImageBuffer<Rgba<u8>, Vec<u8>> {
            ImageBuffer::from_pixel(4, 4, Rgba([255, 255, 255, 255]))
        }
        let mut cache = WordCache::new(2); // Very small capacity to force eviction

        // Add first word
        let word1 = cache
            .get_or_render("first", 0, || Some(dummy_image()))
            .unwrap();
        let size1 = word1.memory_size_bytes();
        let memory_after_first = cache.total_cached_bytes();
        assert_eq!(
            memory_after_first, size1,
            "Memory should equal first word size"
        );

        // Add second word
        let word2 = cache
            .get_or_render("second", 0, || Some(dummy_image()))
            .unwrap();
        let size2 = word2.memory_size_bytes();
        let memory_after_second = cache.total_cached_bytes();
        assert_eq!(
            memory_after_second,
            size1 + size2,
            "Memory should equal sum of both words"
        );

        // Add third word - this should evict the first word due to capacity=2
        let word3 = cache
            .get_or_render("third", 0, || Some(dummy_image()))
            .unwrap();
        let size3 = word3.memory_size_bytes();
        let memory_after_third = cache.total_cached_bytes();

        // After eviction, we should have word2 + word3, not word1 + word2 + word3
        let expected_memory = size2 + size3;
        assert_eq!(
            memory_after_third, expected_memory,
            "Memory accounting should subtract evicted items. Got {} bytes, expected {} bytes (word2 + word3)",
            memory_after_third, expected_memory
        );

        // Verify the cache still has only 2 entries
        assert_eq!(
            cache.len(),
            2,
            "Cache should have exactly 2 entries after eviction"
        );

        // Verify we can still access the third word (cache hit)
        let _ = cache.get_or_render("third", 0, || Some(dummy_image()));
        assert_eq!(cache.hits(), 1, "Should have 1 hit for third word");
    }
}
