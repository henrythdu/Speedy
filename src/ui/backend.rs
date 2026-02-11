//! Terminal backend abstraction for testability
//!
//! This module provides a trait-based abstraction over terminal I/O operations,
//! enabling unit testing of the TUI event loop without requiring an actual terminal.
//!
//! ## Architecture
//!
//! - `TerminalBackend`: Core trait defining terminal operations
//! - `CrosstermBackend`: Production implementation using crossterm
//! - `MockBackend`: Test implementation that records operations for verification
//!
//! ## Example
//!
//! ```rust
//! // Production code
//! let backend = CrosstermBackend::new()?;
//! let mut manager = TuiManager::with_backend(backend);
//!
//! // Test code
//! let backend = MockBackend::new()
//!     .with_events(vec![Event::Key(key_event)]);
//! let mut manager = TuiManager::with_backend(backend);
//! ```

use crate::app::App;
use crossterm::event::{Event, KeyEvent};
use std::io::{self, Write};
use std::time::Duration;

/// Backend trait for terminal I/O operations
///
/// This trait abstracts over the actual terminal implementation, allowing
/// the TUI to be tested with mock backends that simulate user input.
///
/// Implementations:
/// - `CrosstermBackend`: Production backend using crossterm
/// - `MockBackend`: Test backend that replays recorded events
pub trait TerminalBackend {
    /// Read the next event from the terminal
    ///
    /// Blocks until an event is available or an error occurs.
    fn read_event(&mut self) -> io::Result<Event>;

    /// Poll for an event with a timeout
    ///
    /// Returns `Ok(Some(event))` if an event is available,
    /// `Ok(None)` if the timeout expires, or an error if polling fails.
    fn poll_event(&mut self, timeout: Duration) -> io::Result<Option<Event>>;

    /// Render the current application state
    ///
    /// Updates the terminal display based on the provided app state.
    fn render(&mut self, app: &App) -> io::Result<()>;

    /// Clear the terminal screen
    fn clear(&mut self) -> io::Result<()>;

    /// Flush any pending output
    fn flush(&mut self) -> io::Result<()>;
}

/// Production backend implementation using crossterm
///
/// This is the default backend for actual terminal interaction.
/// It uses crossterm for cross-platform terminal control.
pub struct CrosstermBackend {
    terminal: ratatui::Terminal<ratatui::backend::CrosstermBackend<std::io::Stdout>>,
}

impl CrosstermBackend {
    /// Create a new crossterm backend
    ///
    /// Initializes the terminal in raw mode with alternate screen.
    pub fn new() -> io::Result<Self> {
        use crossterm::{
            execute,
            terminal::{enable_raw_mode, EnterAlternateScreen},
        };

        enable_raw_mode()?;
        execute!(std::io::stdout(), EnterAlternateScreen)?;
        std::io::stdout().flush()?;

        let backend = ratatui::backend::CrosstermBackend::new(std::io::stdout());
        let terminal = ratatui::Terminal::new(backend)?;

        Ok(Self { terminal })
    }

    /// Get a reference to the underlying terminal
    pub fn terminal(
        &self,
    ) -> &ratatui::Terminal<ratatui::backend::CrosstermBackend<std::io::Stdout>> {
        &self.terminal
    }

    /// Get a mutable reference to the underlying terminal
    pub fn terminal_mut(
        &mut self,
    ) -> &mut ratatui::Terminal<ratatui::backend::CrosstermBackend<std::io::Stdout>> {
        &mut self.terminal
    }
}

impl TerminalBackend for CrosstermBackend {
    fn read_event(&mut self) -> io::Result<Event> {
        crossterm::event::read()
    }

    fn poll_event(&mut self, timeout: Duration) -> io::Result<Option<Event>> {
        if crossterm::event::poll(timeout)? {
            Ok(Some(crossterm::event::read()?))
        } else {
            Ok(None)
        }
    }

    fn render(&mut self, _app: &App) -> io::Result<()> {
        // Rendering is handled by TuiManager directly for now
        // This method exists for the trait contract
        Ok(())
    }

    fn clear(&mut self) -> io::Result<()> {
        self.terminal.clear()?;
        Ok(())
    }

    fn flush(&mut self) -> io::Result<()> {
        std::io::stdout().flush()
    }
}

impl Drop for CrosstermBackend {
    fn drop(&mut self) {
        use crossterm::{
            cursor::Show,
            execute,
            terminal::{disable_raw_mode, LeaveAlternateScreen},
        };

        let _ = execute!(std::io::stdout(), Show);
        let _ = disable_raw_mode();
        let _ = execute!(std::io::stdout(), LeaveAlternateScreen);
    }
}

/// Mock backend for testing
///
/// Records all operations for verification and replays predefined events.
/// This allows testing TUI logic without an actual terminal.
#[derive(Debug)]
pub struct MockBackend {
    /// Events to be returned by read_event() in sequence
    events: Vec<Event>,
    /// Current position in the events vector
    event_index: usize,
    /// Record of all render calls
    render_calls: Vec<AppSnapshot>,
    /// Record of all clear calls
    clear_calls: usize,
    /// Record of all flush calls
    flush_calls: usize,
}

/// Snapshot of application state for test verification
#[derive(Debug, Clone)]
pub struct AppSnapshot {
    pub mode: String,
    pub current_word: Option<String>,
    pub wpm: u32,
}

impl AppSnapshot {
    /// Create a snapshot from an App instance
    pub fn from_app(app: &App) -> Self {
        Self {
            mode: format!("{:?}", app.mode()),
            current_word: app.get_current_word().map(|s| s.to_string()),
            wpm: app.get_wpm(),
        }
    }
}

impl MockBackend {
    /// Create a new mock backend with no events
    pub fn new() -> Self {
        Self {
            events: Vec::new(),
            event_index: 0,
            render_calls: Vec::new(),
            clear_calls: 0,
            flush_calls: 0,
        }
    }

    /// Configure the mock with a sequence of events to replay
    pub fn with_events(mut self, events: Vec<Event>) -> Self {
        self.events = events;
        self
    }

    /// Add a single event to the sequence
    pub fn add_event(&mut self, event: Event) {
        self.events.push(event);
    }

    /// Get all recorded render calls
    pub fn render_calls(&self) -> &[AppSnapshot] {
        &self.render_calls
    }

    /// Get the number of clear calls
    pub fn clear_calls(&self) -> usize {
        self.clear_calls
    }

    /// Get the number of flush calls
    pub fn flush_calls(&self) -> usize {
        self.flush_calls
    }

    /// Check if all events have been consumed
    pub fn all_events_consumed(&self) -> bool {
        self.event_index >= self.events.len()
    }

    /// Get remaining events count
    pub fn remaining_events(&self) -> usize {
        self.events.len().saturating_sub(self.event_index)
    }

    /// Reset all counters and records
    pub fn reset(&mut self) {
        self.event_index = 0;
        self.render_calls.clear();
        self.clear_calls = 0;
        self.flush_calls = 0;
    }
}

impl Default for MockBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl TerminalBackend for MockBackend {
    fn read_event(&mut self) -> io::Result<Event> {
        if self.event_index < self.events.len() {
            let event = self.events[self.event_index].clone();
            self.event_index += 1;
            Ok(event)
        } else {
            // Return a synthetic key event when out of events
            Ok(Event::Key(KeyEvent::from(crossterm::event::KeyCode::Null)))
        }
    }

    fn poll_event(&mut self, _timeout: Duration) -> io::Result<Option<Event>> {
        if self.event_index < self.events.len() {
            let event = self.events[self.event_index].clone();
            self.event_index += 1;
            Ok(Some(event))
        } else {
            Ok(None)
        }
    }

    fn render(&mut self, app: &App) -> io::Result<()> {
        self.render_calls.push(AppSnapshot::from_app(app));
        Ok(())
    }

    fn clear(&mut self) -> io::Result<()> {
        self.clear_calls += 1;
        Ok(())
    }

    fn flush(&mut self) -> io::Result<()> {
        self.flush_calls += 1;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyEvent};

    #[test]
    fn test_mock_backend_new() {
        let backend = MockBackend::new();
        assert!(backend.all_events_consumed());
        assert_eq!(backend.render_calls().len(), 0);
    }

    #[test]
    fn test_mock_backend_with_events() {
        let events = vec![
            Event::Key(KeyEvent::from(KeyCode::Char('a'))),
            Event::Key(KeyEvent::from(KeyCode::Char('b'))),
        ];
        let mut backend = MockBackend::new().with_events(events);

        assert!(!backend.all_events_consumed());
        assert_eq!(backend.remaining_events(), 2);

        let event = backend.read_event().unwrap();
        assert!(matches!(event, Event::Key(_)));
        assert_eq!(backend.remaining_events(), 1);
    }

    #[test]
    fn test_mock_backend_poll_event() {
        let events = vec![Event::Key(KeyEvent::from(KeyCode::Char('x')))];
        let mut backend = MockBackend::new().with_events(events);

        let event = backend.poll_event(Duration::from_millis(0)).unwrap();
        assert!(event.is_some());

        let event = backend.poll_event(Duration::from_millis(0)).unwrap();
        assert!(event.is_none());
    }

    #[test]
    fn test_mock_backend_records_operations() {
        let mut backend = MockBackend::new();

        // Create a minimal app for testing
        // Note: This test verifies the recording mechanism works
        backend.clear().unwrap();
        backend.flush().unwrap();

        assert_eq!(backend.clear_calls(), 1);
        assert_eq!(backend.flush_calls(), 1);
    }

    #[test]
    fn test_mock_backend_reset() {
        let events = vec![Event::Key(KeyEvent::from(KeyCode::Char('a')))];
        let mut backend = MockBackend::new().with_events(events);

        // Consume the event
        let _ = backend.read_event();
        backend.clear().unwrap();
        backend.flush().unwrap();

        // Reset
        backend.reset();

        assert!(backend.all_events_consumed()); // No events after reset
        assert_eq!(backend.clear_calls(), 0);
        assert_eq!(backend.flush_calls(), 0);
    }
}
