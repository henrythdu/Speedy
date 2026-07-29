//! Autocomplete orchestration: owns the popup state, the file-discovery
//! channel, and the per-directory cache, and exposes the high-level actions
//! the command-deck key loop needs (@-activation, query feeding, selection,
//! refresh, non-blocking result drain).
//!
//! Lives in the autocomplete cell so the FSM is cohesive and testable instead
//! of inlined in the terminal event loop.

use std::path::PathBuf;
use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex};

use super::cache::PerDirectoryCache;
use super::discovery::spawn_discovery_thread;
use super::state::AutocompleteState;

pub struct AutocompleteController {
    state: AutocompleteState,
    receiver: Option<Receiver<PathBuf>>,
    cache: Arc<Mutex<PerDirectoryCache>>,
}

impl Default for AutocompleteController {
    fn default() -> Self {
        Self::new()
    }
}

impl AutocompleteController {
    pub fn new() -> Self {
        Self {
            state: AutocompleteState::new(),
            receiver: None,
            cache: Arc::new(Mutex::new(PerDirectoryCache::new())),
        }
    }

    pub fn is_active(&self) -> bool {
        self.state.is_active()
    }

    /// Borrow the popup state for rendering.
    pub fn state(&self) -> &AutocompleteState {
        &self.state
    }

    /// Activate the popup on an `@` trigger. Returns true if this char activated it.
    pub fn try_activate(&mut self, c: char, command_buffer: &str, cursor_pos: usize) -> bool {
        if c == '@' && AutocompleteState::should_activate(command_buffer, cursor_pos) {
            self.state
                .activate(command_buffer, cursor_pos, &current_dir());
            self.spawn();
            true
        } else {
            false
        }
    }

    /// Feed a typed char to the active query (no-op if inactive).
    pub fn feed_char(&mut self, c: char) {
        if self.state.is_active() {
            self.state.handle_input(c);
        }
    }

    /// Backspace the query; deactivates when the query empties.
    pub fn backspace(&mut self) {
        self.state.backspace();
        if self.state.query().is_empty() {
            self.state.deactivate();
        }
    }

    pub fn select_previous(&mut self) {
        self.state.select_previous();
    }

    pub fn select_next(&mut self) {
        self.state.select_next();
    }

    /// Apply the selected file to the command buffer and close the popup.
    pub fn apply_and_close(&mut self, command_buffer: &mut String) {
        self.state.apply_selection(command_buffer);
        self.state.deactivate();
    }

    /// Apply the selected file and keep the popup open for chained `@`-completions.
    pub fn apply_and_chain(&mut self, command_buffer: &mut String) {
        self.state.apply_selection(command_buffer);
        command_buffer.push(' ');
    }

    pub fn deactivate(&mut self) {
        self.state.deactivate();
    }

    /// Ctrl+R: invalidate the current directory's cache and restart discovery.
    pub fn refresh(&mut self) {
        let dir = current_dir();
        if let Ok(mut cache) = self.cache.lock() {
            cache.invalidate(&dir);
        }
        self.state.clear_files();
        self.spawn();
    }

    /// Non-blocking drain of discovery results into the popup state.
    pub fn poll(&mut self) {
        let mut disconnected = false;
        if let Some(receiver) = self.receiver.as_ref() {
            loop {
                match receiver.try_recv() {
                    Ok(file) => self.state.add_file(file),
                    Err(std::sync::mpsc::TryRecvError::Empty) => break,
                    Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                        self.state.mark_scanning_complete();
                        disconnected = true;
                        break;
                    }
                }
            }
        }
        if disconnected {
            self.receiver = None;
        }
    }

    fn spawn(&mut self) {
        let cache = Arc::clone(&self.cache);
        let handle = spawn_discovery_thread(current_dir(), cache);
        self.receiver = Some(handle.receiver);
    }
}

fn current_dir() -> PathBuf {
    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn try_activate_only_triggers_on_at_when_eligible() {
        let mut ctrl = AutocompleteController::new();
        // Empty buffer + '@' at position 0 should_activate returns true
        let buf = String::new();
        assert!(ctrl.try_activate('@', &buf, 0));
        assert!(ctrl.is_active());

        // A second activation char on a non-eligible buffer does not re-trigger
        let mut ctrl2 = AutocompleteController::new();
        assert!(!ctrl2.try_activate('a', "a", 1));
        assert!(!ctrl2.is_active());
    }

    #[test]
    fn feed_char_is_noop_when_inactive() {
        let mut ctrl = AutocompleteController::new();
        ctrl.feed_char('x'); // must not panic, stays inactive
        assert!(!ctrl.is_active());
    }

    #[test]
    fn backspace_deactivates_when_query_empties() {
        let mut ctrl = AutocompleteController::new();
        ctrl.try_activate('@', "", 0);
        assert!(ctrl.is_active());
        // query is empty already post-activation; backspace deactivates
        ctrl.backspace();
        assert!(!ctrl.is_active());
    }
}
