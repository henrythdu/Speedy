use crate::app::{mode::AppMode, App};
use crate::engine::wpm_to_milliseconds;
use crate::rendering::kitty::KittyGraphicsRenderer;
use crate::rendering::renderer::RsvpRenderer;
use crate::ui::reader::view::render_command_deck;
use crate::ui::theme::Theme;

use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::Style,
    widgets::Block,
    Terminal,
};
use std::io::{self, Stdout};
use std::time::{Duration, Instant};

pub struct TuiManager {
    terminal: Terminal<CrosstermBackend<Stdout>>,
    command_buffer: String,
    kitty_renderer: KittyGraphicsRenderer,
}

impl TuiManager {
    pub fn new() -> Result<Self, io::Error> {
        enable_raw_mode()?;
        execute!(io::stdout(), EnterAlternateScreen)?;

        let backend = CrosstermBackend::new(io::stdout());
        let terminal = Terminal::new(backend)?;

        // Initialize Kitty Graphics renderer (always required - no fallback)
        let mut renderer = KittyGraphicsRenderer::new();
        <KittyGraphicsRenderer as crate::rendering::renderer::RsvpRenderer>::initialize(
            &mut renderer,
        )
        .expect("Failed to initialize KittyGraphicsRenderer");

        // Query terminal dimensions and calculate font size
        let _ = renderer.viewport().query_dimensions();
        if let Some(dims) = renderer.viewport().get_dimensions() {
            // Calculate font size for 5-line height (cell height * 5)
            renderer.calculate_font_size_from_cell_height(dims.cell_size.1);
        }

        Ok(TuiManager {
            terminal,
            command_buffer: String::new(),
            kitty_renderer: renderer,
        })
    }

    pub fn run_event_loop(&mut self, app: &mut App) -> io::Result<AppMode> {
        let mut last_tick = Instant::now();
        let render_tick = Duration::from_millis(1000 / 60);

        loop {
            let current_mode = app.mode();
            if current_mode == AppMode::Quit {
                return Ok(current_mode);
            }
            // Command, Reading, and Paused all stay in TUI
            // Command mode shows the command deck for input
            // Reading and Paused modes show the RSVP display

            let wpm = app.get_wpm();
            let timeout_ms = wpm_to_milliseconds(wpm);
            let poll_timeout = Duration::from_millis(timeout_ms);

            match event::poll(poll_timeout) {
                Ok(true) => {
                    match event::read()? {
                        Event::Key(key) => {
                            // Handle Ctrl+C to quit
                            if key.code == KeyCode::Char('c')
                                && key.modifiers.contains(event::KeyModifiers::CONTROL)
                            {
                                app.set_mode(AppMode::Quit);
                                return Ok(AppMode::Quit);
                            }

                            match key.code {
                                KeyCode::Char(c) => {
                                    if app.mode() == AppMode::Command {
                                        // In command mode, collect input
                                        self.command_buffer.push(c);
                                    } else {
                                        // In reading/paused mode, use app key handling
                                        app.handle_keypress(c);
                                    }
                                }
                                KeyCode::Enter => {
                                    if app.mode() == AppMode::Command
                                        && !self.command_buffer.is_empty()
                                    {
                                        // Execute the command
                                        let command = self.command_buffer.clone();
                                        self.command_buffer.clear();

                                        // Parse and execute
                                        use crate::ui::command::{parse_command, Command};
                                        match parse_command(&command) {
                                            Command::LoadFile(path) => {
                                                // Load the file using input module
                                                use crate::input::pdf;
                                                match pdf::load(&path) {
                                                    Ok(doc) => {
                                                        let text: String = doc
                                                            .tokens
                                                            .iter()
                                                            .map(|t| {
                                                                let mut s = t.text.clone();
                                                                for p in &t.punctuation {
                                                                    s.push(*p);
                                                                }
                                                                s
                                                            })
                                                            .collect::<Vec<_>>()
                                                            .join(" ");
                                                        app.start_reading(&text, 300);
                                                    }
                                                    Err(e) => {
                                                        eprintln!("Failed to load file: {}", e);
                                                    }
                                                }
                                            }
                                            Command::LoadClipboard => {
                                                // Load from clipboard
                                                use crate::input::clipboard;
                                                match clipboard::load() {
                                                    Ok(doc) => {
                                                        let text: String = doc
                                                            .tokens
                                                            .iter()
                                                            .map(|t| {
                                                                let mut s = t.text.clone();
                                                                for p in &t.punctuation {
                                                                    s.push(*p);
                                                                }
                                                                s
                                                            })
                                                            .collect::<Vec<_>>()
                                                            .join(" ");
                                                        app.start_reading(&text, 300);
                                                    }
                                                    Err(e) => {
                                                        eprintln!(
                                                            "Failed to load clipboard: {}",
                                                            e
                                                        );
                                                    }
                                                }
                                            }
                                            Command::Quit => {
                                                app.set_mode(AppMode::Quit);
                                                return Ok(AppMode::Quit);
                                            }
                                            Command::Help => {
                                                // Show help - for now just stay in command mode
                                            }
                                            Command::Unknown(_) => {
                                                // Invalid command - could show error in UI
                                                eprintln!("Unknown command: {}", command);
                                            }
                                        }
                                    }
                                }
                                KeyCode::Backspace => {
                                    if app.mode() == AppMode::Command {
                                        self.command_buffer.pop();
                                    }
                                }
                                KeyCode::Esc => {
                                    if app.mode() == AppMode::Reading
                                        || app.mode() == AppMode::Paused
                                    {
                                        app.set_mode(AppMode::Command);
                                        self.command_buffer.clear();
                                    }
                                }
                                _ => {}
                            }
                        }
                        Event::Resize(cols, rows) => {
                            // Handle terminal resize with minimum size enforcement
                            if cols >= 80 && rows >= 24 {
                                let _ = self.handle_resize(cols, rows, app);
                            }
                        }
                        _ => {}
                    }
                }
                Ok(false) => {
                    // Only auto-advance in Reading mode, not Paused
                    if app.mode() == AppMode::Reading {
                        app.advance_reading();
                    }
                }
                Err(e) => {
                    // Propagate I/O errors instead of ignoring them
                    return Err(e);
                }
            }

            if last_tick.elapsed() >= render_tick {
                self.render_frame(app)?;
                last_tick = Instant::now();
            }
        }
    }

    pub fn render_frame(&mut self, app: &App) -> io::Result<()> {
        // Step 1: Render via Ratatui FIRST to establish background (reading zone + command deck)
        // This draws the persistent UI that never changes
        self.terminal.draw(|frame| {
            let area = frame.area();

            // Split screen: Reading zone (dynamic) + Command deck (fixed 5 lines)
            // This must match the calculation in KittyGraphicsRenderer::calculate_reading_zone_center
            let main_layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Fill(1), Constraint::Length(5)])
                .split(area);

            let reading_area = main_layout[0];
            let command_area = main_layout[1];

            // Fill reading zone with theme background color
            let theme = Theme::midnight();
            let reading_bg = Block::default().style(Style::default().bg(theme.background));
            frame.render_widget(reading_bg, reading_area);

            // Command deck area - this NEVER goes away
            render_command_deck(frame, command_area, app.mode(), &self.command_buffer);
        })?;

        // Step 2: Clear previous word from Kitty Graphics Protocol
        // This must happen BEFORE rendering the new word to prevent stacking
        if let Err(e) = RsvpRenderer::clear(&mut self.kitty_renderer) {
            eprintln!("Warning: Failed to clear previous word: {}", e);
        }

        // Step 3: Render word via Kitty Graphics Protocol ON TOP of Ratatui background
        // Only word is transmitted (transparent background), not a full canvas
        // This creates a smooth RSVP experience where only the word changes at WPM rate
        if let Some(word) = app.get_current_word() {
            let anchor_pos = crate::reading::calculate_anchor_position(word);

            // Use render_word which only transmits word image (not full canvas)
            // The word has transparent background so Ratatui background shows through
            if let Err(e) = RsvpRenderer::render_word(&mut self.kitty_renderer, word, anchor_pos) {
                eprintln!("Render error: {}", e);
            }
        }

        Ok(())
    }

    /// Handle terminal resize events
    ///
    /// Updates viewport dimensions and redraws the current word at the new center position.
    /// Auto-pauses reading during resize to prevent visual artifacts (per Design Doc Section 8.1).
    fn handle_resize(&mut self, _cols: u16, _rows: u16, app: &mut App) -> io::Result<()> {
        // Auto-pause if currently reading to prevent visual artifacts
        let was_reading = app.mode() == AppMode::Reading;
        if was_reading {
            app.toggle_pause();
        }

        // Update viewport dimensions by re-querying terminal
        // This updates the viewport dimensions which are now used dynamically
        // for both X and Y center calculations
        let _ = self.kitty_renderer.viewport().query_dimensions();

        // Clear previous graphics and redraw at new position
        if let Err(e) = RsvpRenderer::clear(&mut self.kitty_renderer) {
            eprintln!("Warning: Failed to clear during resize: {}", e);
        }

        // Force immediate redraw
        self.render_frame(app)?;

        // Resume if we were reading
        if was_reading {
            app.toggle_pause();
        }

        Ok(())
    }
}

impl Drop for TuiManager {
    fn drop(&mut self) {
        // Cleanup Kitty graphics before exiting
        let _ = RsvpRenderer::cleanup(&mut self.kitty_renderer);

        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen);
    }
}
