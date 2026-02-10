use crate::app::{mode::AppMode, App};
use crate::rendering::kitty::KittyGraphicsRenderer;
use crate::rendering::renderer::RsvpRenderer;
use crate::ui::reader::view::{render_command_deck, render_wpm};
use crate::ui::theme::Theme;

use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::Style,
    widgets::Block,
    Terminal,
};
use std::io::{self, Stdout, Write};
use std::thread;
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
        io::stdout().flush()?;
        
        // Give terminal time to switch to alternate screen
        thread::sleep(Duration::from_millis(100));

        let backend = CrosstermBackend::new(io::stdout());
        let mut terminal = Terminal::new(backend)?;
        
        // Force initial draw to initialize terminal size
        terminal.autoresize()?;

        // Initialize Kitty Graphics renderer (always required - no fallback)
        let mut renderer = KittyGraphicsRenderer::new();
        <KittyGraphicsRenderer as crate::rendering::renderer::RsvpRenderer>::initialize(
            &mut renderer,
        )
        .expect("Failed to initialize KittyGraphicsRenderer");

        // Query terminal dimensions and calculate font size
        // Note: This uses CSI escape sequences that may not be supported by all terminals
        // If the query fails, we fall back to estimated dimensions
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

        // Force initial render before entering loop
        self.terminal.clear()?;
        self.terminal.flush()?;
        self.render_frame(app)?;

        loop {
            let current_mode = app.mode();
            if current_mode == AppMode::Quit {
                return Ok(current_mode);
            }
            // Command, Reading, and Paused all stay in TUI
            // Command mode shows the command deck for input
            // Reading and Paused modes show the RSVP display

            let timeout_ms = app.get_current_token_duration();
            // Ensure minimum timeout to prevent busy-waiting and allow render tick to fire
            let poll_timeout = Duration::from_millis(timeout_ms.max(16));

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
                                        use crate::ui::command_executor::{
                                            execute_command, CommandResult,
                                        };
                                        match execute_command(app, &command)? {
                                            CommandResult::Continue => {}
                                            CommandResult::Exit(mode) => return Ok(mode),
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
                    if app.mode() == AppMode::Reading
                        && !app.advance_reading() {
                            app.set_mode(AppMode::Paused);
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

    /// Calculate the reader area (terminal minus command section)
    fn calculate_reader_area(&mut self) -> Rect {
        if let Some(dims) = self.kitty_renderer.viewport().get_dimensions() {
            let command_height = (dims.cell_size.1 * 5.0) as u32;
            Rect::new(
                0,
                0,
                dims.pixel_size.0 as u16,
                (dims.pixel_size.1 - command_height) as u16,
            )
        } else {
            // Fallback dimensions
            Rect::new(0, 0, 800, 600)
        }
    }

    pub fn render_frame(&mut self, app: &mut App) -> io::Result<()> {
        // Calculate reader area once at the beginning (for all rendering steps)
        let reader_area = self.calculate_reader_area();

        // Render background via Ratatui
        self.terminal.draw(|frame| {
            let area = frame.area();

            let main_layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Fill(1), Constraint::Length(5)])
                .split(area);

            let reading_area = main_layout[0];
            let command_area = main_layout[1];

            let theme = Theme::midnight();
            let reading_bg = Block::default().style(Style::default().bg(theme.background));
            frame.render_widget(reading_bg, reading_area);

            // Render WPM display in top-left of reading area
            let wpm = app.get_wpm();
            render_wpm(frame, reading_area, wpm, &theme);

            render_command_deck(
                frame,
                command_area,
                app.mode(),
                &self.command_buffer,
                app.get_error(),
            );
        })?;
        
        // Flush stdout to ensure ratatui output appears immediately
        io::stdout().flush()?;

        // Render word via Kitty Graphics Protocol
        // Skip rendering pure whitespace/newline tokens to avoid blank screens
        if let Some(word) = app.get_current_word() {
            let trimmed = word.trim();
            if !trimmed.is_empty() && trimmed != "\n" && trimmed != "\r\n" {
                // Clear previous word only when rendering a new one
                if let Err(e) = RsvpRenderer::clear(&mut self.kitty_renderer) {
                    app.set_error(format!("Failed to clear previous word: {}", e));
                }
                let anchor_pos = crate::reading::calculate_anchor_position(&word);

                // Render word
                if let Err(e) =
                    RsvpRenderer::render_word(&mut self.kitty_renderer, &word, anchor_pos)
                {
                    app.set_error(format!("Render error: {}", e));
                }

                // Render progress bar and macro gutter
                if let Some(reading_state) = &app.reading_state {
                    // Extract values we need to avoid borrow issues
                    let current_index = reading_state.current_index;
                    let total_tokens = reading_state.tokens.len();
                    let app_mode = app.mode();
                    
                    let progress = crate::reading::calculate_sentence_progress(
                        current_index,
                        &reading_state.tokens,
                    );
                    let word_y = self.kitty_renderer.get_vertical_center().unwrap_or(0);
                    if let Ok(word_height) =
                        self.kitty_renderer.calculate_word_height(&word, anchor_pos)
                    {
                        // Render micro bar (sentence progress)
                        let bar_image_id = self.kitty_renderer.current_image_id;
                        if let Err(e) = self.kitty_renderer.render_bar(
                            word_y,
                            word_height,
                            progress,
                            &app_mode,
                            bar_image_id,
                        ) {
                            app.set_error(format!("Bar render error: {}", e));
                        } else {
                            // Increment image ID after successful bar render
                            self.kitty_renderer.current_image_id += 1;
                        }

                        // Render macro gutter (document progress)
                        let gutter_id = self.kitty_renderer.current_image_id;
                        if let Err(e) = self.kitty_renderer.render_macro_gutter(
                            current_index,
                            total_tokens,
                            reader_area,
                            app_mode,
                            gutter_id,
                        ) {
                            app.set_error(format!("Gutter render error: {}", e));
                        } else {
                            self.kitty_renderer.current_image_id += 1;
                        }
                    }
                }
            }
            // If word is newline/whitespace, don't clear or render - keep previous word visible
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
            app.set_error(format!("Failed to clear during resize: {}", e));
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
