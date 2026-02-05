# Epic 3: Word-Level LRU Cache Implementation

**Status:** Planning Complete ✅ Ready for Implementation
**Created:** 2026-02-05
**Purpose:** Performance optimization to enable consistent 1000+ WPM reading speeds
**PRD Reference:** Section 6.3 Performance Requirements
**Validation:** Completed (Consensus → Challenge → ThinkDeep)

---

## 1. Epic Overview

**Problem Statement:**
At 1000 WPM (~17 words/second), the current implementation re-rasterizes every word for every frame using `ab_glyph` + `imageproc`. This causes CPU spikes (~1-5ms per word) and inconsistent timing, disrupting the reading flow.

**Solution:**
Implement a Word-Level LRU Cache that stores pre-rendered RGBA buffers keyed by `(word_string, font_size, anchor_position)`. This eliminates redundant rasterization for repeated words, which occur frequently in English text (~20% repetition rate).

**Performance Goals:**
- **Cache hit time:** O(1) lookup (~microseconds)
- **Cache miss time:** O(n) rasterization (~1-5ms)
- **Target cache hit rate:** ~70% with typical English text
- **Net improvement:** Reduce per-frame time from ~3ms to <1ms (on cache hit)

**Big Picture:**
This epic enables the "Performance at 1000+ WPM" requirement from the PRD (Section 6.3). By caching pre-rendered word buffers, we eliminate the most expensive operation in the rendering pipeline (rasterization), making high-speed reading fluid and responsive.

---

## 2. Dependencies

### Completed Dependencies
- ✅ Image-based rendering with `ab_glyph` + `imageproc`
- ✅ `KittyGraphicsRenderer` with sub-pixel OVP anchoring
- ✅ `RsvpRenderer` trait for pluggable backends
- ✅ Font loading system with metrics (JetBrains Mono bundled)
- ✅ `calculate_anchor_position()` function (deterministic anchor calculation)

### Codebase Dependencies
- `src/rendering/kitty/rasterizer.rs` - `rasterize_word()` function to wrap
- `src/rendering/kitty/mod.rs` - `KittyGraphicsRenderer` to integrate
- `src/rendering/font.rs` - Font metrics for cache key construction
- `src/reading/ovp.rs` - `calculate_anchor_position()` for validation

### External Dependencies
- `lru = "0.12"` to be added to Cargo.toml

---

## 3. Technical Specifications

### 3.1 Cache Key Design (Validated Correct)

**Format:** Tuple key `(word: String, font_size: f32, anchor_position: usize)`

**Why This Design is Correct:**
After validation workflow (Consensus → Challenge → ThinkDeep), we confirmed that including `anchor_position` in the cache key is the RIGHT approach because:
1. `anchor_position` is deterministic from `calculate_anchor_position(word)`
2. For word "hello" (5 chars), anchor is ALWAYS at position 1 (2nd letter)
3. Same word + same font = same anchor position always
4. Cache key captures EXACT rendering state for hit-or-miss matching
5. On cache hit: Zero re-rasterization (full benefit realized)

**Example:**
- `CacheKey { word: "hello", font_size: 24.0, anchor_position: 1 }`
- Subsequent "hello" appearances at 24px font ALWAYS use same anchor position
- Cache doesn't fragment - stores exact rendering once, reuses it

### 3.2 Cache Value Design

```rust
pub struct CachedWord {
    buffer: ImageBuffer<Rgba<u8>, Vec<u8>>,  // Pre-rendered RGBA buffer
    width: u32,                              // Image width in pixels
    height: u32,                             // Image height in pixels
}
```

### 3.3 Enhanced WordCache API

```rust
pub struct WordCache {
    cache: LruCache<CacheKey, CachedWord>,  // LRU cache with tuple keys
    font_size: f32,                        // Current font size
    hits: u64,                             // Hit counter for telemetry
    misses: u64,                           // Miss counter for telemetry
    total_cached_bytes: u64,               // Memory tracking
    memory_cap_bytes: u64,                 // 75MB default memory cap
}

impl WordCache {
    pub fn new(capacity: usize) -> Self;
    
    pub fn get_or_render(
        &mut self,
        word: &str,
        anchor_position: usize,
        font: &FontRef,
        metrics: &FontMetrics,
    ) -> Result<&CachedWord, CacheError>;
    
    pub fn clear(&mut self);
    
    pub fn set_font_size(&mut self, font_size: f32);
    
    pub fn get_hit_rate(&self) -> f64;      // hits / (hits + misses)
    
    pub fn get_memory_usage_mb(&self) -> f64;  // total_cached_bytes / 1MB
}
```

### 3.4 Integration Flow

```
render_word(word, anchor_position)
  ↓
word_cache.get_or_render(word, anchor_position, font, metrics)
  ↓
  ├─ Check memory cap, evict if needed
  ├─ Construct cache key (word, font_size, anchor_position)
  ├─ Cache hit: 
  │   - Increment hits counter
  │   - Return reference to CachedWord (~microseconds)
  └─ Cache miss:
      - Increment misses counter
      - Call rasterize_word() (~1-5ms)
      - Store in cache
      - Update total_cached_bytes
      - Return reference
  ↓
encode_image_base64(cached.buffer)  // ~1-2ms
  ↓
transmit_graphics(...)  // ~1-2ms
```

---

## 4. Implementation Details

### 4.1 Cache Capacity and Memory Cap

**Default Capacity:** 1000 entries
**Memory Cap:** 75MB (configurable)

**Rationale:**
- English vocabulary has ~170,000 words, but reading sessions use ~1,000-2,000 unique words
- Memory estimate: ~50-100MB for typical use
- Memory-based eviction prevents issues with long words or large fonts
- Can tune based on real measurements

**Tuning:**
```rust
const DEFAULT_CACHE_CAPACITY: usize = 1000;
const DEFAULT_MEMORY_CAP_BYTES: u64 = 75 * 1024 * 1024; // 75MB
```

### 4.2 Cache Invalidation

**Clear When:**
- Font size changes (call `set_font_size()` which clears cache)
- Font family changes (future feature)
- DPI/scaling changes (if detectable)

**Don't Clear When:**
- WPM changes (speed doesn't affect rendering)
- Terminal resize (cache images are pixel-based)
- Document changes (cache is word-based, document-agnostic)

### 4.3 Cache Statistics

**Tracking:**
- `hits: u64` - Successful cache lookups
- `misses: u64` - Cache misses requiring rasterization
- `total_cached_bytes: u64` - Total memory used by cached entries

**Access:**
- `get_hit_rate()` - Calculate hits / (hits + misses)
- `get_memory_usage_mb()` - Return memory usage in MB

**Debug Logging:**
```rust
#[cfg(debug_assertions)]
{
    log::debug!("Cache hit rate: {:.1}%", cache.get_hit_rate() * 100.0);
    log::debug!("Cache memory: {:.1} MB", cache.get_memory_usage_mb());
}
```

---

## 5. File Structure

### New Files
- `src/rendering/cache.rs` - WordCache implementation with tests

### Modified Files
- `src/rendering/kitty/mod.rs` - Add WordCache to KittyGraphicsRenderer
- `Cargo.toml` - Add `lru = "0.12"` dependency

### No Changes To
- `src/rendering/kitty/rasterizer.rs` - `rasterize_word()` unchanged (cache wraps it)
- `src/rendering/kitty/protocol.rs` - Transmission logic unchanged
- `src/rendering/kitty/positioning.rs` - OVP calculation unchanged
- `src/rendering/renderer.rs` - RsvpRenderer trait unchanged (cache internal)

---

## 6. Testing Strategy

### 6.1 Unit Tests (src/rendering/cache.rs)

**Test Scenarios:**
1. **Basic Cache Operations:**
   - Test cache creation with capacity and memory cap
   - Test cache hit returns cached entry
   - Test cache miss renders new entry
   - Test cache eviction when capacity exceeded

2. **Key Generation:**
   - Test different words produce different keys
   - Test same word with different anchor positions (should produce different keys but rare in practice)
   - Test same word with different font sizes produces different keys

3. **Cache Invalidation:**
   - Test `clear()` empties cache
   - Test `set_font_size()` clears cache and updates size

4. **Memory Tracking:**
   - Test `total_cached_bytes` updates correctly
   - Test memory cap enforcement evicts entries

5. **Statistics:**
   - Test hit/miss counters increment correctly
   - Test `get_hit_rate()` calculation
   - Test `get_memory_usage_mb()` calculation

6. **Edge Cases:**
   - Empty string word
   - Unicode words (e.g., "café", "日本語")
   - Very long words (e.g., 50+ characters)

### 6.2 Integration Tests (tests/cache_integration.rs)

**Test Scenarios:**
1. **Cache Integration:**
   - Render same word twice: Second render should use cached entry
   - Verify hit counter increments
   - Verify miss counter increments on first render

2. **Performance Benchmark:**
   - Measure cache hit rate on typical text (1000-word sample)
   - Verify ≥60% hit rate target is met
   - Compare render time with/without cache

3. **Memory Validation:**
   - Render 1000 unique words, verify memory usage
   - Verify memory cap enforcement

4. **Stress Test:**
   - Render 10,000 words rapidly (simulate 1000 WPM)
   - Verify cache doesn't grow unbounded
   - Verify no memory leaks

### 6.3 Manual Testing

**Scenarios:**
1. Read sample document at 300 WPM (smooth)
2. Increase to 1000 WPM (smooth with cache)
3. Navigate back/forward with j/k keys (words should be instant on re-display)
4. Change font size (cache should clear, no stale words displayed)

---

## 7. Acceptance Criteria

### 7.1 Functional Requirements
- ✅ WordCache correctly caches rendered word buffers
- ✅ Cache lookup returns previously rendered word (O(1) performance)
- ✅ Cache miss triggers rasterization and stores result
- ✅ Cache evicts least-recently-used entry when capacity or memory exceeded
- ✅ Cache clears when font size changes
- ✅ KittyGraphicsRenderer uses cache in `render_word()`

### 7.2 Performance Requirements
- ✅ Cache hit time: <1ms
- ✅ Cache miss time: <5ms (includes rasterization)
- ✅ Cache hit rate: ≥60% on typical English text (target: 70%)
- ✅ Memory usage: <75MB (configurable cap)

### 7.3 Test Coverage
- ✅ All unit tests pass (new tests in cache.rs)
- ✅ All integration tests pass (new tests in cache_integration.rs)
- ✅ Existing tests still pass (no regressions - 178 tests)
- ✅ Performance benchmarks meet targets (hit rate ≥60%)

### 7.4 Documentation Requirements
- ✅ ARCHITECTURE.md updated with WordCache struct
- ✅ ARCHITECTURE.md updated with cache module reference
- ✅ Cargo.toml includes `lru = "0.12"` dependency
- ✅ Code comments explain cache key design and invalidation logic

---

## 8. Risk Assessment

### 8.1 Technical Risks (All Mitigated)

**Memory Usage**
- **Impact:** High (could be excessive on low-memory systems)
- **Probability:** Low (typical desktops have 8GB+ RAM)
- **Mitigation:** Memory-based eviction cap (75MB default) + runtime tracking

**Cache Hit Rate Lower Than Expected**
- **Impact:** Medium (performance improvement less than anticipated)
- **Probability:** Medium (depends on text characteristics)
- **Mitigation:** Hit/miss counters for monitoring; can increase capacity if needed

**Cache Invalidation Issues**
- **Impact:** Medium (stale cached words could display incorrectly)
- **Probability:** Low (simple clear on font changes)
- **Mitigation:** Document clear conditions; add tests; memory tracking

**Performance Regression**
- **Impact:** High (cache overhead could slow rendering)
- **Probability:** Low (LRU crate is O(1), minimal overhead)
- **Mitigation:** Tuple keys avoid String allocation; benchmark before/after

### 8.2 Operational Risks (All Mitigated)

**Timeline Tightness**
- **Impact:** Low (extended to 3-4 days)
- **Probability:** Low (build in buffer)
- **Mitigation:** Extended timeline accounts for edge case debugging

**External Dependency (`lru` crate)**
- **Impact:** Very Low (crate is well-maintained, minimal risk)
- **Probability:** Very Low (crate has been stable for years)
- **Mitigation:** None needed; crate is mature and widely used

---

## 9. Success Metrics

**Primary Success Metric:**
- **Cache Hit Rate:** ≥60% on typical English text (target: 70%)
  - Measured via integration tests with 1000-word sample text
  - Formula: `hits / (hits + misses)`

**Secondary Success Metrics:**
- **Performance Improvement:** ≥30% reduction in average render time
  - Measured via benchmarks comparing cache vs. no-cache
  - Target: Reduce average render time from 3ms to 2ms
- **Memory Usage:** <75MB for 1000-entry cache
  - Measured via `total_cached_bytes` field
- **Test Coverage:** 100% of new code paths tested
  - All unit tests pass
  - All integration tests pass
  - No regression in existing tests (178 tests still passing)

---

## 10. Deliverables

### Code Deliverables
1. `src/rendering/cache.rs` - WordCache implementation with tests
2. `tests/cache_integration.rs` - Integration tests for cache
3. `src/rendering/kitty/mod.rs` - Updated to use WordCache
4. `Cargo.toml` - Added `lru = "0.12"` dependency

### Documentation Deliverables
1. ARCHITECTURE.md - Updated with WordCache struct and module reference
2. Inline code comments - Explaining cache key design and invalidation logic
3. This epic plan document - Detailed specification

### Quality Deliverables
1. All unit tests passing
2. All integration tests passing
3. All existing tests still passing (no regressions)
4. Clippy passes with no warnings
5. Fmt passes with no formatting issues

---

## 11. Timeline Estimate

**Total Effort:** 3-4 days (extended from 2-3 days for buffer)

**Breakdown:**
- **Day 1:** WordCache implementation with memory tracking and counters
- **Day 2:** Integration with KittyGraphicsRenderer
- **Day 3:** Testing (unit + integration) and performance tuning
- **Day 4:** Buffer for edge cases and documentation

**Dependencies:**
- None (all prerequisites completed in previous epics)

---

## 12. Implementation Phases

### Phase 1: WordCache Implementation (Day 1)
1. Create `src/rendering/cache.rs` module
2. Implement `CacheKey` struct with `Hash` trait (tuple key)
3. Implement `CachedWord` struct with `ImageBuffer`, width, height
4. Implement `WordCache` struct with:
   - LRU cache (1000 entries)
   - Hit/miss counters
   - Memory tracking (`total_cached_bytes`, `memory_cap_bytes`)
5. Implement `new()`, `clear()`, `set_font_size()` methods
6. Implement `get_or_render()` with:
   - Memory cap enforcement
   - Cache lookup with hit/miss tracking
   - Fallback to `rasterize_word()` on miss
7. Add `get_hit_rate()` and `get_memory_usage_mb()` helpers
8. Write comprehensive unit tests (hit, miss, eviction, edge cases)

### Phase 2: Renderer Integration (Day 2)
1. Add `WordCache` field to `KittyGraphicsRenderer`
2. Initialize cache in `initialize()` method
3. Clear cache in `cleanup()` method
4. Modify `render_word()` to use `word_cache.get_or_render()`
5. Update `set_font_size()` call to sync cache
6. Test integration with manual rendering

### Phase 3: Testing & Validation (Day 3)
1. Write integration tests in `tests/cache_integration.rs`
2. Test cache hit rate with 1000-word sample (target: ≥60%)
3. Test memory tracking doesn't exceed cap
4. Test cache invalidation on font size changes
5. Run existing test suite (178 tests) to verify no regressions
6. Benchmark with different WPM speeds (300, 500, 1000)

### Phase 4: Polish & Documentation (Day 4)
1. Add debug logging for hit/miss rates (conditional)
2. Update ARCHITECTURE.md with WordCache
3. Update inline code comments
4. Final code review and clippy/fmt checks
5. Prepare handoff notes

---

## 13. Next Epic (Brief Overview Only)

**Epic 4: CPU Compositing with Ghost Words**

**Direction:**
- Implement single-buffer rendering to eliminate flickering
- Add ghost word support (previous/next words at 15% opacity)
- Composite all visual elements before transmission

**Rationale:**
- Eliminates Z-fighting and visual artifacts from multiple image layers
- Improves visual quality and reduces user distraction
- Follows PRD Section 4.3 "Three-Container Model"

---

**Epic Status:** Planning Complete ✅ Validation Passed ✅
**Next Step:** Bead Creation and Implementation
**Confidence Level:** 9.5/10 (Very High)
