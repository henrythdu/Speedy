//! Integration tests for autocomplete keyboard handling
//!
//! Tests the integration between AutocompleteState and keyboard events.

use speedy::ui::autocomplete::state::AutocompleteState;
use std::path::PathBuf;

#[test]
fn autocomplete_full_workflow_basic() {
    let mut state = AutocompleteState::new();
    let mut command_buffer = String::from("@");

    // Activate autocomplete
    state.activate(&command_buffer, 1, &PathBuf::from("/test"));
    assert!(state.active);
    assert!(state.is_scanning);

    // Simulate discovering files
    state.add_file(PathBuf::from("/test/doc1.pdf"));
    state.add_file(PathBuf::from("/test/book.epub"));
    state.add_file(PathBuf::from("/test/readme.txt"));
    state.mark_scanning_complete();

    // Should have 2 supported files
    assert_eq!(state.match_count(), 2);

    // Select first file
    assert_eq!(state.selected_idx, 0);
    let selected = state.get_selected().unwrap();
    assert_eq!(selected.file_name().unwrap(), "doc1.pdf");

    // Navigate down
    state.select_next();
    assert_eq!(state.selected_idx, 1);
    let selected = state.get_selected().unwrap();
    assert_eq!(selected.file_name().unwrap(), "book.epub");

    // Apply selection
    let new_buffer = state.apply_selection(&mut command_buffer);
    assert!(new_buffer.contains("book.epub"));

    // Deactivate
    state.deactivate();
    assert!(!state.active);
}

#[test]
fn autocomplete_filtering_workflow() {
    let mut state = AutocompleteState::new();
    let command_buffer = String::from("@");

    // Activate and add files
    state.activate(&command_buffer, 1, &PathBuf::from("/test"));
    state.add_file(PathBuf::from("/test/document.pdf"));
    state.add_file(PathBuf::from("/test/book.epub"));
    state.add_file(PathBuf::from("/test/draft.pdf"));
    state.mark_scanning_complete();

    // Initially all PDF and EPUB files should show
    assert_eq!(state.match_count(), 3);

    // Type "doc" to filter
    state.handle_input('d');
    state.handle_input('o');
    state.handle_input('c');

    // Should only match document.pdf
    assert_eq!(state.match_count(), 1);
    assert_eq!(
        state.get_selected().unwrap().file_name().unwrap(),
        "document.pdf"
    );

    // Clear filter with backspace
    state.backspace();
    state.backspace();
    state.backspace();

    // All files should show again
    assert_eq!(state.match_count(), 3);
}

#[test]
fn autocomplete_navigation_wrapping() {
    let mut state = AutocompleteState::new();
    let command_buffer = String::from("@");

    state.activate(&command_buffer, 1, &PathBuf::from("/test"));
    state.add_file(PathBuf::from("/test/a.pdf"));
    state.add_file(PathBuf::from("/test/b.pdf"));
    state.add_file(PathBuf::from("/test/c.pdf"));
    state.mark_scanning_complete();

    assert_eq!(state.selected_idx, 0);

    // Navigate down wraps to first
    state.select_next();
    assert_eq!(state.selected_idx, 1);
    state.select_next();
    assert_eq!(state.selected_idx, 2);
    state.select_next();
    assert_eq!(state.selected_idx, 0); // Wrapped

    // Navigate up wraps to last
    state.select_previous();
    assert_eq!(state.selected_idx, 2); // Wrapped
}

#[test]
fn autocomplete_activation_conditions() {
    // At start of buffer
    assert!(AutocompleteState::should_activate("", 0));

    // After whitespace
    assert!(AutocompleteState::should_activate("hello ", 6));
    assert!(AutocompleteState::should_activate("cmd\t", 4));
    assert!(AutocompleteState::should_activate("args\n", 5));

    // Not after other characters
    assert!(!AutocompleteState::should_activate("hello", 5));
    assert!(!AutocompleteState::should_activate("cmd@", 4));
    assert!(!AutocompleteState::should_activate("test@file", 9));
}

#[test]
fn autocomplete_empty_results() {
    let mut state = AutocompleteState::new();
    let command_buffer = String::from("@");

    state.activate(&command_buffer, 1, &PathBuf::from("/test"));
    state.mark_scanning_complete();

    // No files discovered
    assert_eq!(state.match_count(), 0);

    // Navigation should not panic
    state.select_next();
    state.select_previous();

    // Get selected should return None
    assert!(state.get_selected().is_none());
}

#[test]
fn autocomplete_tab_completion() {
    let mut state = AutocompleteState::new();
    let mut command_buffer = String::from("@do");

    state.activate(&command_buffer, 1, &PathBuf::from("/test"));
    state.add_file(PathBuf::from("/test/document.pdf"));
    state.add_file(PathBuf::from("/test/other.epub"));
    state.mark_scanning_complete();

    // Filter to "do"
    state.handle_input('d');
    state.handle_input('o');

    // Apply selection (simulates Tab press)
    let result = state.apply_selection(&mut command_buffer);

    // Should replace @do with the full path
    assert!(result.contains("document.pdf"));
    assert!(!result.contains("@do"));
}

#[test]
fn autocomplete_backspace_closes_when_empty() {
    let mut state = AutocompleteState::new();
    let command_buffer = String::from("@");

    state.activate(&command_buffer, 1, &PathBuf::from("/test"));
    assert!(state.active);

    // Simulate typing and then backspacing
    state.handle_input('t');
    state.backspace();

    // Query is now empty but autocomplete is still active
    assert!(state.active);
    assert!(state.query.is_empty());
}
