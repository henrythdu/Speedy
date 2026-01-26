use speedy::engine::config::TimingConfig;
use speedy::engine::error::load_file_safe;
use speedy::engine::state::ReadingState;
use speedy::engine::timing::tokenize_text;
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
