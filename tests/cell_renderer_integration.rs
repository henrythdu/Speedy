//! Integration tests for CellRenderer

use speedy::rendering::cell::CellRenderer;
use speedy::rendering::renderer::RsvpRenderer;
use unicode_width::UnicodeWidthStr;

#[test]
fn test_cell_renderer_lifecycle() {
    let mut renderer = CellRenderer::new();

    // Initialize
    assert!(renderer.initialize().is_ok());

    // Render some words
    assert!(renderer.render_word("hello", 0).is_ok());
    assert!(renderer.render_word("world", 2).is_ok());
    assert!(renderer.render_word("rust", 1).is_ok());

    // Clear
    assert!(renderer.clear().is_ok());

    // Cleanup
    assert!(renderer.cleanup().is_ok());
}

#[test]
fn test_cell_renderer_ovp_calculations() {
    let mut renderer = CellRenderer::new();
    renderer.update_terminal_size(80, 24);

    // Test various word lengths and anchor positions
    let test_cases = vec![
        ("hello", 2, 38), // Center at 40, anchor at 2, start at 38
        ("a", 0, 40),     // Single char centered
        ("ab", 0, 40),    // Two chars, anchor at first
        ("world", 0, 40), // Five chars, anchor at first
        ("test", 1, 39),  // Four chars, anchor at position 1
    ];

    for (word, anchor, expected_start) in test_cases {
        let start = renderer.calculate_start_column(word, anchor).unwrap();
        assert_eq!(
            start, expected_start,
            "Word '{}' with anchor {} should start at column {}",
            word, anchor, expected_start
        );
    }
}

#[test]
fn test_cell_renderer_different_terminal_sizes() {
    let mut renderer = CellRenderer::new();

    // Test with various terminal sizes
    let sizes = vec![(80, 24), (120, 40), (60, 20), (200, 60)];

    for (width, height) in sizes {
        renderer.update_terminal_size(width, height);

        let center_row = renderer.get_center_row();
        assert_eq!(center_row, height / 2);

        let start_col = renderer.calculate_start_column("test", 1).unwrap();
        let expected_center = width / 2;
        assert_eq!(start_col, expected_center - 1);
    }
}

#[test]
fn test_cell_renderer_error_handling() {
    let mut renderer = CellRenderer::new();
    renderer.initialize().unwrap();

    // Test out of bounds anchor
    let result = renderer.render_word("hi", 5);
    assert!(result.is_err());

    // Test valid anchors
    assert!(renderer.render_word("a", 0).is_ok());
    assert!(renderer.render_word("ab", 0).is_ok());
    assert!(renderer.render_word("ab", 1).is_ok());
}

#[test]
fn test_cell_renderer_does_not_support_subpixel() {
    let renderer = CellRenderer::new();
    assert!(!renderer.supports_subpixel_ovp());
}

#[test]
fn test_cell_renderer_with_unicode_emojis() {
    let mut renderer = CellRenderer::new();
    renderer.update_terminal_size(80, 24);

    // Test with emoji
    assert!(renderer.render_word("hi😊", 2).is_ok());
    assert_eq!(renderer.get_current_word(), Some("hi😊"));

    // Calculate start column with emoji
    let start = renderer.calculate_start_column("hi😊", 2).unwrap();
    // 80 cols, center=40, anchor=2, start=38
    assert_eq!(start, 38);
}

#[test]
fn test_cell_renderer_with_cjk_characters() {
    let mut renderer = CellRenderer::new();
    renderer.update_terminal_size(80, 24);

    // Test with CJK characters
    assert!(renderer.render_word("你好", 1).is_ok());
    assert_eq!(renderer.get_current_word(), Some("你好"));

    // Calculate start column with CJK
    let start = renderer.calculate_start_column("你好", 1).unwrap();
    // 80 cols, center=40, prefix "你" has width 2, start=40-2=38
    assert_eq!(start, 38);
}

#[test]
fn test_cell_renderer_mixed_content() {
    let mut renderer = CellRenderer::new();
    renderer.update_terminal_size(100, 30);

    // Render different types of content
    assert!(renderer.render_word("test123", 3).is_ok());
    assert_eq!(renderer.get_current_word(), Some("test123"));

    renderer.clear().unwrap();
    assert!(renderer.get_current_word().is_none());

    // Re-render after clear
    assert!(renderer.render_word("final", 2).is_ok());
    assert_eq!(renderer.get_current_word(), Some("final"));
}
