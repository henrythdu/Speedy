use speedy::app::mode::AppMode;
use speedy::app::{App, AppEvent};
use speedy::engine::error::load_file_safe;
use speedy::engine::ovp::calculate_anchor_position;
use speedy::engine::state::ReadingState;
use speedy::engine::timing::{tokenize_text, wpm_to_milliseconds};
use std::fs::{self, File};
use std::io::Write;

#[test]
fn end_to_end_reading() {
    let test_file = "test_e2e.txt";
    let content = "Hello world! This is a test of the RSVP reader.";

    let mut file = File::create(test_file).unwrap();
    file.write_all(content.as_bytes()).unwrap();

    let loaded_content = load_file_safe(test_file).expect("Should load file successfully");
    assert_eq!(loaded_content, content);

    let tokens = tokenize_text(&loaded_content);
    assert!(!tokens.is_empty(), "Should have tokens");
    assert_eq!(tokens[0].text, "Hello");
    assert_eq!(tokens[1].text, "world");

    let mut state = ReadingState::new_with_default_config(tokens, 300);
    assert!(state.current_token().is_some());
    assert_eq!(state.current_token().unwrap().text, "Hello");

    state.advance();
    assert_eq!(state.current_token().unwrap().text, "world");

    state.adjust_wpm(50);
    assert_eq!(state.wpm, 350);

    fs::remove_file(test_file).unwrap();
}

#[test]
fn tui_workflow_repl_to_reading_mode() {
    let mut app = App::new();

    assert_eq!(app.mode, AppMode::Repl);

    app.handle_event(AppEvent::LoadFile("nonexistent.txt".to_string()));

    assert_eq!(app.mode, AppMode::Repl);
}

#[test]
fn tui_workflow_render_state_generation() {
    let content = "one two three four five six seven eight nine ten";

    let mut app = App::new();
    app.start_reading(content, 300);

    let render_state = app.get_render_state();
    assert!(render_state.current_word.is_some());
    assert_eq!(render_state.progress, (0, 10));

    app.advance_reading();
    let render_state = app.get_render_state();
    assert_eq!(render_state.progress, (1, 10));
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

    assert_eq!(app.mode, AppMode::Repl);

    app.set_mode(AppMode::Paused);
    assert_eq!(app.mode, AppMode::Paused);

    app.set_mode(AppMode::Reading);
    assert_eq!(app.mode, AppMode::Reading);

    app.set_mode(AppMode::Repl);
    assert_eq!(app.mode, AppMode::Repl);
}

#[test]
fn tui_workflow_reading_advance() {
    let content = "word1 word2 word3 word4 word5";

    let mut app = App::new();
    app.start_reading(content, 300);

    let initial_state = app.get_render_state();
    assert_eq!(initial_state.current_word, Some("word1".to_string()));

    let advanced = app.advance_reading();
    assert!(advanced);

    let after_advance = app.get_render_state();
    assert_eq!(after_advance.current_word, Some("word2".to_string()));

    app.advance_reading();
    app.advance_reading();
    let third_advance = app.get_render_state();
    assert_eq!(third_advance.current_word, Some("word4".to_string()));

    app.advance_reading();
    let final_advance = app.get_render_state();
    assert_eq!(final_advance.current_word, Some("word5".to_string()));

    let no_more = app.advance_reading();
    assert!(!no_more);
}
