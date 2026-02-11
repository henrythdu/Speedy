use speedy::app::mode::AppMode;
use speedy::app::App;
use speedy::reading::ovp::calculate_anchor_position;
use speedy::reading::state::ReadingState;
use speedy::reading::timing::wpm_to_milliseconds;
use speedy::reading::tokenization::tokenize_text;
use std::fs::{self, File};
use std::io::{Read, Write};

// RAII guard to ensure test file cleanup even on panic
struct TestFileGuard<'a> {
    path: &'a str,
}

impl<'a> TestFileGuard<'a> {
    fn new(path: &'a str) -> Self {
        Self { path }
    }
}

impl<'a> Drop for TestFileGuard<'a> {
    fn drop(&mut self) {
        let _ = fs::remove_file(self.path);
    }
}

#[test]
fn end_to_end_reading() {
    let test_file = "test_e2e.txt";
    let _guard = TestFileGuard::new(test_file); // Ensures cleanup on drop
    let content = "Hello world! This is a test of the RSVP reader.";

    let mut file = File::create(test_file).unwrap();
    file.write_all(content.as_bytes()).unwrap();

    // Load file content directly
    let mut loaded_file = File::open(test_file).unwrap();
    let mut loaded_content = String::new();
    loaded_file.read_to_string(&mut loaded_content).unwrap();
    assert_eq!(loaded_content, content);

    let tokens = tokenize_text(&loaded_content);
    assert!(!tokens.is_empty(), "Should have tokens");
    assert_eq!(tokens[0].text(), "Hello");
    assert_eq!(tokens[1].text(), "world");

    let mut state = ReadingState::new_with_default_config(tokens, 300);
    assert!(state.current_token().is_some());
    assert_eq!(state.current_token().unwrap().text(), "Hello");

    state.advance();
    assert_eq!(state.current_token().unwrap().text(), "world");

    state.adjust_wpm(50);
    assert_eq!(state.wpm(), 350);
    // File cleanup handled by guard
}

#[test]
fn tui_workflow_reading_progress() {
    let content = "one two three four five six seven eight nine ten";

    let mut app = App::new();
    app.start_reading(content, 300);

    // Check initial state
    assert_eq!(app.get_current_word(), Some("one".to_string()));

    app.advance_reading();
    assert_eq!(app.get_current_word(), Some("two".to_string()));
}

#[test]
fn tui_workflow_ovp_anchoring() {
    assert_eq!(calculate_anchor_position("a"), 0);
    assert_eq!(calculate_anchor_position("hello"), 1);
    assert_eq!(calculate_anchor_position("worldwide"), 2);
    assert_eq!(calculate_anchor_position("extraordinary"), 3);
}

#[test]
fn tui_workflow_timing_precision() {
    assert_eq!(wpm_to_milliseconds(300), 200);
    assert_eq!(wpm_to_milliseconds(165), 364);
    assert_eq!(wpm_to_milliseconds(600), 100);
}

#[test]
fn tui_workflow_app_mode_transitions() {
    let mut app = App::new();

    assert_eq!(app.mode(), AppMode::Command);

    app.set_mode(AppMode::Paused);
    assert_eq!(app.mode(), AppMode::Paused);

    app.set_mode(AppMode::Reading);
    assert_eq!(app.mode(), AppMode::Reading);

    app.set_mode(AppMode::Command);
    assert_eq!(app.mode(), AppMode::Command);
}

#[test]
fn tui_workflow_reading_advance() {
    let content = "word1 word2 word3 word4 word5";

    let mut app = App::new();
    app.start_reading(content, 300);

    // Check initial state
    assert_eq!(app.get_current_word(), Some("word1".to_string()));

    let advanced = app.advance_reading();
    assert!(advanced);
    assert_eq!(app.get_current_word(), Some("word2".to_string()));

    app.advance_reading();
    app.advance_reading();
    assert_eq!(app.get_current_word(), Some("word4".to_string()));

    app.advance_reading();
    assert_eq!(app.get_current_word(), Some("word5".to_string()));

    let no_more = app.advance_reading();
    assert!(!no_more);
}
