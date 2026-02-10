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

/// Calculate responsive margins based on terminal size
/// Returns (top_margin, bottom_margin, left_margin, right_margin)
fn calculate_margins(area: Rect) -> (u16, u16, u16, u16) {
    const MIN_HEIGHT_FOR_FULL_MARGINS: u16 = 15;
    const TARGET_TOP_BOTTOM: u16 = 1;
    const TARGET_LEFT_RIGHT: u16 = 1;

    let height = area.height;

    if height >= MIN_HEIGHT_FOR_FULL_MARGINS {
        // Full margins for large terminals
        (
            TARGET_TOP_BOTTOM,
            TARGET_TOP_BOTTOM,
            TARGET_LEFT_RIGHT,
            TARGET_LEFT_RIGHT,
        )
    } else if height >= 10 {
        // Reduced margins for medium terminals
        (1, 1, TARGET_LEFT_RIGHT, TARGET_LEFT_RIGHT)
    } else {
        // Minimal margins for small terminals
        (0, 0, 0, 0)
    }
}
use std::io::{self, Stdout, Write};
use std::thread;
use std::time::{Duration, Instant};

pub struct TuiManager {
    terminal: Terminal<CrosstermBackend<Stdout>>,
    command_buffer: String,
    kitty_renderer: KittyGraphicsRenderer,
    cursor_visible: bool,        // Blink state
    last_cursor_toggle: Instant, // Last toggle time
    last_keypress: Instant,      // For pause-on-type
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
            cursor_visible: true, // Start visible
            last_cursor_toggle: Instant::now(),
            last_keypress: Instant::now(),
        })
    }

    pub fn run_event_loop(&mut self, app: &mut App) -> io::Result<AppMode> {
        let mut last_tick = Instant::now();
        let render_tick = Duration::from_millis(1000 / 60);

        // Cursor blink timing constants
        const CURSOR_BLINK_INTERVAL: Duration = Duration::from_millis(500);
        const CURSOR_PAUSE_AFTER_TYPING: Duration = Duration::from_millis(500);

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

            // Cursor blink management (only in Command mode)
            let mut needs_redraw = false;
            if app.mode() == AppMode::Command {
                let time_since_keypress = self.last_keypress.elapsed();

                if time_since_keypress >= CURSOR_PAUSE_AFTER_TYPING {
                    // Time to resume blinking
                    if self.last_cursor_toggle.elapsed() >= CURSOR_BLINK_INTERVAL {
                        self.cursor_visible = !self.cursor_visible;
                        self.last_cursor_toggle = Instant::now();
                        needs_redraw = true;
                    }
                } else {
                    // Recently typed - keep cursor visible
                    if !self.cursor_visible {
                        self.cursor_visible = true;
                        needs_redraw = true;
                    }
                }
            }

            // Use fixed short timeout in Command mode for responsive input
            // Use token duration in Reading/Paused modes for word timing
            let poll_timeout = if app.mode() == AppMode::Command {
                Duration::from_millis(50) // 50ms for responsive cursor blink and input
            } else {
                let timeout_ms = app.get_current_token_duration();
                Duration::from_millis(timeout_ms.max(16))
            };

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
                                        self.last_keypress = Instant::now(); // Reset blink pause
                                        self.cursor_visible = true; // Show cursor immediately
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
                            // Handle terminal resize - always update viewport
                            let _ = self.handle_resize(cols, rows, app);
                        }
                        _ => {}
                    }
                }
                Ok(false) => {
                    // Only auto-advance in Reading mode, not Paused
                    if app.mode() == AppMode::Reading && !app.advance_reading() {
                        app.set_mode(AppMode::Paused);
                    }
                }
                Err(e) => {
                    // Propagate I/O errors instead of ignoring them
                    return Err(e);
                }
            }

            if last_tick.elapsed() >= render_tick || needs_redraw {
                self.render_frame(app)?;
                last_tick = Instant::now();
            }
        }
    }

    /// Calculate the reader area (terminal minus command section and margins)
    fn calculate_reader_area(&mut self) -> Rect {
        if let Some(dims) = self.kitty_renderer.viewport().get_dimensions() {
            let command_height = (dims.cell_size.1 * 5.0) as u32;

            // Calculate margins in pixels based on terminal height
            let terminal_height_cells = (dims.pixel_size.1 as f32 / dims.cell_size.1) as u16;
            let (top_margin_cells, bottom_margin_cells, left_margin_cells, right_margin_cells) =
                calculate_margins(Rect::new(
                    0,
                    0,
                    dims.pixel_size.0 as u16,
                    terminal_height_cells,
                ));

            // Convert cell margins to pixels
            let top_margin_px = (top_margin_cells as f32 * dims.cell_size.1) as u32;
            let bottom_margin_px = (bottom_margin_cells as f32 * dims.cell_size.1) as u32;
            let left_margin_px = (left_margin_cells as f32 * dims.cell_size.0) as u32;
            let right_margin_px = (right_margin_cells as f32 * dims.cell_size.0) as u32;

            Rect::new(
                left_margin_px as u16,
                top_margin_px as u16,
                (dims.pixel_size.0 - left_margin_px - right_margin_px) as u16,
                (dims.pixel_size.1 - command_height - top_margin_px - bottom_margin_px) as u16,
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
            let theme = Theme::midnight();

            // Fill entire viewport with background color first
            let full_bg = Block::default().style(Style::default().bg(theme.background));
            frame.render_widget(full_bg, area);

            // Calculate responsive margins based on terminal size
            let (top, bottom, left, right) = calculate_margins(area);

            // Apply vertical margins
            let vertical_margins = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(top),
                    Constraint::Fill(1),
                    Constraint::Length(bottom),
                ])
                .split(area);

            let content_area = vertical_margins[1];

            // Apply horizontal margins
            let horizontal_margins = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Length(left),
                    Constraint::Fill(1),
                    Constraint::Length(right),
                ])
                .split(content_area);

            let inner_area = horizontal_margins[1];

            // Split inner area into reading and command sections
            let main_layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Fill(1), Constraint::Length(5)])
                .split(inner_area);

            let reading_area = main_layout[0];
            let command_area = main_layout[1];

            // Content areas use same background as margins for seamless look
            // (Background already filled entire viewport above)

            // Render WPM display in top-left of reading area
            let wpm = app.get_wpm();
            render_wpm(frame, reading_area, wpm, &theme);

            render_command_deck(
                frame,
                command_area,
                app.mode(),
                &self.command_buffer,
                app.get_error(),
                self.cursor_visible && app.mode() == AppMode::Command, // Only blink cursor in Command mode
            );
        })?;

        // Flush stdout to ensure ratatui output appears immediately
        io::stdout().flush()?;

        // Render word via Kitty Graphics Protocol
        // Always clear previous graphics first to prevent artifacts when switching modes
        if let Err(e) = RsvpRenderer::clear(&mut self.kitty_renderer) {
            app.set_error(format!("Failed to clear previous word: {}", e));
        }

        // Skip rendering pure whitespace/newline tokens to avoid blank screens
        if let Some(word) = app.get_current_word() {
            let trimmed = word.trim();
            if !trimmed.is_empty() && trimmed != "\n" && trimmed != "\r\n" {
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
                        }
                        // Always increment image ID to maintain ID sequence
                        self.kitty_renderer.current_image_id += 1;

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
                        }
                        // Always increment image ID to maintain ID sequence
                        self.kitty_renderer.current_image_id += 1;
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
