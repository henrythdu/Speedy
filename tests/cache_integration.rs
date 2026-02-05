//! Cache Integration Tests
//!
//! Tests the integration of WordCache with KittyGraphicsRenderer
//! to ensure proper caching behavior and performance characteristics.

use speedy::rendering::cache::{WordCache, DEFAULT_CACHE_CAPACITY};
use speedy::rendering::font::{get_font, get_font_metrics};
use speedy::rendering::kitty::KittyGraphicsRenderer;
use speedy::rendering::renderer::RsvpRenderer;

/// Setup helper to create a renderer with initialized cache
fn setup_renderer_with_cache() -> KittyGraphicsRenderer {
    let mut renderer = KittyGraphicsRenderer::new();
    renderer
        .initialize()
        .expect("Failed to initialize renderer");
    renderer
}

#[test]
fn test_cache_integration_same_word_uses_cached_entry() {
    let mut renderer = setup_renderer_with_cache();

    // Render the same word twice
    let word = "hello";
    let anchor = 1; // For 5-char word, anchor is at position 1

    // First render - should be a cache miss
    let result1 = renderer.render_word(word, anchor);
    assert!(result1.is_ok(), "First render should succeed");

    // Second render - should be a cache hit
    let result2 = renderer.render_word(word, anchor);
    assert!(result2.is_ok(), "Second render should succeed");

    // Note: We can't directly access the cache hit rate from the renderer,
    // but this test verifies the integration works without errors
}

#[test]
fn test_cache_integration_different_words_no_interference() {
    let mut renderer = setup_renderer_with_cache();

    // Render multiple different words
    let words = vec![
        ("hello", 1),
        ("world", 1),
        ("the", 1),
        ("quick", 1),
        ("brown", 1),
    ];

    // First pass - render all words (cache misses)
    for (word, anchor) in &words {
        let result = renderer.render_word(word, *anchor);
        assert!(result.is_ok(), "Render should succeed for word: {}", word);
    }

    // Second pass - render all words again (cache hits)
    for (word, anchor) in &words {
        let result = renderer.render_word(word, *anchor);
        assert!(
            result.is_ok(),
            "Second render should succeed for word: {}",
            word
        );
    }
}

#[test]
fn test_cache_integration_repeated_words_high_hit_rate() {
    let mut renderer = setup_renderer_with_cache();

    // Simulate reading with repeated words (common in English)
    let text = "the quick brown fox jumps over the lazy dog the quick brown fox";
    let words: Vec<&str> = text.split_whitespace().collect();

    // Render all words
    for word in &words {
        let anchor = calculate_expected_anchor(word);
        let result = renderer.render_word(word, anchor);
        assert!(result.is_ok(), "Render should succeed for word: {}", word);
    }

    // Render them again (simulating going back in text)
    for word in words.iter().rev() {
        let anchor = calculate_expected_anchor(word);
        let result = renderer.render_word(word, anchor);
        assert!(
            result.is_ok(),
            "Reverse render should succeed for word: {}",
            word
        );
    }
}

#[test]
fn test_cache_integration_different_anchor_positions() {
    let mut renderer = setup_renderer_with_cache();

    // Same word with different anchor positions should be different cache entries
    // (though in practice, anchor is deterministic based on word length)
    let word = "test";

    // Render with different anchors
    for anchor in 0..word.len() {
        let result = renderer.render_word(word, anchor);
        assert!(
            result.is_ok(),
            "Render should succeed for anchor: {}",
            anchor
        );
    }
}

#[test]
fn test_cache_integration_unicode_words() {
    let mut renderer = setup_renderer_with_cache();

    // Test with unicode words
    let unicode_words = vec![("café", 1), ("naïve", 1), ("résumé", 2), ("über", 1)];

    for (word, anchor) in unicode_words {
        let result = renderer.render_word(word, anchor);
        assert!(
            result.is_ok(),
            "Render should succeed for unicode word: {}",
            word
        );
    }
}

#[test]
fn test_cache_integration_long_words() {
    let mut renderer = setup_renderer_with_cache();

    // Test with long words (should still work)
    let long_words = vec![
        "supercalifragilisticexpialidocious",
        "Antidisestablishmentarianism",
        "Pneumonoultramicroscopicsilicovolcanoconiosis",
    ];

    for word in long_words {
        let anchor = calculate_expected_anchor(word);
        let result = renderer.render_word(word, anchor);
        assert!(result.is_ok(), "Render should succeed for long word");
    }
}

#[test]
fn test_cache_integration_special_characters() {
    let mut renderer = setup_renderer_with_cache();

    // Test words with special characters
    let special_words = vec![
        ("can't", 1),
        ("won't", 1),
        ("it's", 1),
        ("don't", 1),
        ("hello!", 1),
        ("test?", 1),
    ];

    for (word, anchor) in special_words {
        let result = renderer.render_word(word, anchor);
        assert!(
            result.is_ok(),
            "Render should succeed for word with special chars: {}",
            word
        );
    }
}

#[test]
fn test_word_cache_basic_operations() {
    let font = get_font().expect("Font should be available");
    let metrics = get_font_metrics(&font, 24.0);
    let mut cache = WordCache::new(100);

    // Test cache miss
    let result1 = cache.get_or_render("hello", 1, &font, &metrics);
    assert!(result1.is_ok(), "First render should succeed");
    assert_eq!(cache.misses(), 1, "Should have 1 miss");
    assert_eq!(cache.hits(), 0, "Should have 0 hits");

    // Test cache hit
    let result2 = cache.get_or_render("hello", 1, &font, &metrics);
    assert!(result2.is_ok(), "Second render should succeed");
    assert_eq!(cache.misses(), 1, "Should still have 1 miss");
    assert_eq!(cache.hits(), 1, "Should have 1 hit");

    // Verify hit rate
    let hit_rate = cache.get_hit_rate();
    assert!(
        (hit_rate - 0.5).abs() < 0.01,
        "Hit rate should be 50% (1 hit / 2 total)"
    );
}

#[test]
fn test_word_cache_memory_tracking() {
    let font = get_font().expect("Font should be available");
    let metrics = get_font_metrics(&font, 24.0);
    let mut cache = WordCache::new(100);

    // Initial memory should be 0
    assert_eq!(cache.total_cached_bytes(), 0, "Initial memory should be 0");

    // Add a word
    let _ = cache.get_or_render("hello", 1, &font, &metrics);

    // Memory should have increased
    assert!(
        cache.total_cached_bytes() > 0,
        "Memory should increase after adding word"
    );

    // Memory usage should be reasonable (< 1MB for one word)
    assert!(
        cache.total_cached_bytes() < 1_000_000,
        "Memory for one word should be < 1MB"
    );
}

#[test]
fn test_word_cache_font_size_change_clears_cache() {
    let font = get_font().expect("Font should be available");
    let metrics = get_font_metrics(&font, 24.0);
    let mut cache = WordCache::new(100);

    // Add some words
    let _ = cache.get_or_render("hello", 1, &font, &metrics);
    let _ = cache.get_or_render("world", 1, &font, &metrics);

    assert!(cache.len() > 0, "Cache should have entries");

    // Change font size
    cache.set_font_size(48.0);

    // Cache should be cleared
    assert_eq!(
        cache.len(),
        0,
        "Cache should be cleared after font size change"
    );
    assert_eq!(cache.total_cached_bytes(), 0, "Memory should be reset");
    assert_eq!(cache.hits(), 0, "Stats should be reset");
}

#[test]
fn test_word_cache_capacity_enforcement() {
    let font = get_font().expect("Font should be available");
    let metrics = get_font_metrics(&font, 24.0);
    let mut cache = WordCache::new(5); // Small capacity

    // Add more words than capacity
    for i in 0..10 {
        let word = format!("word{}", i);
        let _ = cache.get_or_render(&word, 1, &font, &metrics);
    }

    // Cache should not exceed capacity
    assert!(cache.len() <= 5, "Cache should not exceed capacity");
}

#[test]
fn test_word_cache_different_words_different_keys() {
    let font = get_font().expect("Font should be available");
    let metrics = get_font_metrics(&font, 24.0);
    let mut cache = WordCache::new(100);

    // Add different words
    let _ = cache.get_or_render("hello", 1, &font, &metrics);
    let _ = cache.get_or_render("world", 1, &font, &metrics);
    let _ = cache.get_or_render("test", 0, &font, &metrics);

    // Should have 3 entries
    assert_eq!(cache.len(), 3, "Should have 3 different cache entries");
    assert_eq!(cache.misses(), 3, "Should have 3 misses");
}

/// Helper function to calculate expected anchor position based on word length
/// Mirrors the logic in calculate_anchor_position()
fn calculate_expected_anchor(word: &str) -> usize {
    let len = word.chars().count();
    match len {
        0..=1 => 0,
        2..=5 => 1,
        6..=9 => 2,
        _ => 3,
    }
}

#[test]
fn test_cache_hit_rate_calculation() {
    let font = get_font().expect("Font should be available");
    let metrics = get_font_metrics(&font, 24.0);
    let mut cache = WordCache::new(100);

    // No lookups yet - hit rate should be 0
    assert_eq!(cache.get_hit_rate(), 0.0, "Initial hit rate should be 0");

    // Add word (miss)
    let _ = cache.get_or_render("test", 1, &font, &metrics);
    assert_eq!(
        cache.get_hit_rate(),
        0.0,
        "Hit rate after 1 miss should be 0"
    );

    // Get same word (hit)
    let _ = cache.get_or_render("test", 1, &font, &metrics);
    assert!(
        (cache.get_hit_rate() - 0.5).abs() < 0.001,
        "Hit rate should be 50%"
    );

    // Get same word again (hit)
    let _ = cache.get_or_render("test", 1, &font, &metrics);
    assert!(
        (cache.get_hit_rate() - 0.666).abs() < 0.01,
        "Hit rate should be 66.7%"
    );
}

#[test]
fn test_memory_usage_mb_calculation() {
    let font = get_font().expect("Font should be available");
    let metrics = get_font_metrics(&font, 24.0);
    let mut cache = WordCache::new(100);

    // Initial memory should be 0 MB
    assert_eq!(
        cache.get_memory_usage_mb(),
        0.0,
        "Initial memory usage should be 0"
    );

    // Add a word
    let _ = cache.get_or_render("test", 1, &font, &metrics);

    // Memory should be > 0 and < 1 MB for one word
    let mb = cache.get_memory_usage_mb();
    assert!(mb > 0.0, "Memory usage should be > 0 MB");
    assert!(mb < 1.0, "Memory usage for one word should be < 1 MB");
}
