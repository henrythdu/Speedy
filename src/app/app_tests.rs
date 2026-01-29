use crate::app::mode::AppMode;
use crate::app::{App, AppEvent, RenderState};
use crate::reading::state::ReadingState;

#[test]
fn test_app_event_enum_exists() {
    let _load_file = AppEvent::LoadFile("test.txt".to_string());
    let _load_clipboard = AppEvent::LoadClipboard;
    let _quit = AppEvent::Quit;
    let _help = AppEvent::Help;
}

#[test]
fn test_app_handle_event_quit() {
    let mut app = App::new();
    app.handle_event(AppEvent::Quit);
    assert_eq!(app.mode, AppMode::Quit);
}

#[test]
fn test_app_handle_event_help() {
    let mut app = App::new();
    app.handle_event(AppEvent::Help);
}

#[test]
fn test_app_get_render_state_returns_correct_initial_state() {
    let app = App::new();
    let state = app.get_render_state();
    assert_eq!(state.mode, AppMode::Command);
    assert_eq!(state.current_word, None);
    let _render_state: RenderState = state;
}
