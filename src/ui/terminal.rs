use crate::app::{mode::AppMode, App};
use crate::reading::{
    calculate_anchor_position, calculate_anchor_position_from_len, calculate_sentence_progress,
};
use crate::rendering::renderer::{RenderFrame, RsvpRenderer};
use crate::ui::autocomplete::controller::AutocompleteController;
use crate::ui::autocomplete::render::render_autocomplete_popup;
use crate::ui::config_popup::render_config_popup;
use crate::ui::key_handler::{KeyHandlerRegistry, KeyResult};
use crate::ui::key_handlers::{
    create_command_handlers, create_popup_handlers, create_reading_handlers,
};
use crate::ui::parts::background::render_background_cells;
use crate::ui::parts::deck::{render_command_deck, render_wpm};
use crate::ui::parts::word::KittyGraphicsRenderer;
use crate::ui::theme::{set_current_theme, Theme};
use anyhow::Result;
use imageproc::image::Rgba;

use crossterm::{
    cursor::{Hide, Show},
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::Color,
    Terminal,
};

/// Calculate responsive margins based on terminal size
/// Returns (top_margin, bottom_margin, left_margin, right_margin)
fn calculate_margins(area: Rect) -> (u16, u16, u16, u16) {
    const MIN_HEIGHT_FOR_FULL_MARGINS: u16 = 15;
    const TARGET_LEFT_RIGHT: u16 = 1;

    let height = area.height;

    if height >= MIN_HEIGHT_FOR_FULL_MARGINS {
        // Full margins for large terminals
        (1, 1, TARGET_LEFT_RIGHT, TARGET_LEFT_RIGHT)
    } else {
        // Minimal margins for small terminals
        (0, 0, 0, 0)
    }
}
use std::io::{self, Stdout, Write};
use std::time::{Duration, Instant};

/// Cursor blink state machine.
///
/// Stays solid for `PAUSE_AFTER_TYPING` after the last keypress, then blinks on
/// `BLINK_INTERVAL`. Extracted from TuiManager so the terminal layer doesn't own
/// a UI-timing concern inline.
struct CursorBlinker {
    visible: bool,
    last_toggle: Instant,
    last_keypress: Instant,
}

impl CursorBlinker {
    const BLINK_INTERVAL: Duration = Duration::from_millis(500);
    const PAUSE_AFTER_TYPING: Duration = Duration::from_millis(500);

    fn new() -> Self {
        Self {
            visible: true,
            last_toggle: Instant::now(),
            last_keypress: Instant::now(),
        }
    }

    /// Record a keypress: show the cursor immediately and hold off blinking.
    fn on_key(&mut self) {
        self.last_keypress = Instant::now();
        self.visible = true;
    }

    /// Advance blink state. Returns true if visibility changed (caller should redraw).
    fn tick(&mut self) -> bool {
        if self.last_keypress.elapsed() < Self::PAUSE_AFTER_TYPING {
            if !self.visible {
                self.visible = true;
                return true;
            }
            return false;
        }
        if self.last_toggle.elapsed() >= Self::BLINK_INTERVAL {
            self.visible = !self.visible;
            self.last_toggle = Instant::now();
            true
        } else {
            false
        }
    }

    fn visible(&self) -> bool {
        self.visible
    }
}

pub struct TuiManager {
    terminal: Terminal<CrosstermBackend<Stdout>>,
    kitty_renderer: KittyGraphicsRenderer,
    cursor: CursorBlinker,

    // Autocomplete subsystem (state + discovery + cache)
    autocomplete: AutocompleteController,

    // Key handler registry for OCP-compliant key handling
    key_registry: KeyHandlerRegistry,
}

impl TuiManager {
    pub fn new() -> Result<Self> {
        enable_raw_mode()?;
        execute!(io::stdout(), EnterAlternateScreen)?;
        io::stdout().flush()?;

        let backend = CrosstermBackend::new(io::stdout());
        let mut terminal = Terminal::new(backend)?;

        // Force initial draw to initialize terminal size
        terminal.autoresize()?;

        // Initialize Kitty Graphics renderer (always required - no fallback)
        let mut renderer = KittyGraphicsRenderer::new();
        <KittyGraphicsRenderer as crate::rendering::renderer::RsvpRenderer>::initialize(
            &mut renderer,
        )?;

        // Query terminal dimensions and calculate font size
        // Note: This uses CSI escape sequences that may not be supported by all terminals
        // If the query fails, we fall back to estimated dimensions
        let _ = renderer.viewport().query_dimensions();
        if let Some(dims) = renderer.viewport().get_dimensions() {
            // Calculate font size for 5-line height (cell height * 5)
            renderer.calculate_font_size_from_cell_height(dims.cell_size.1);
        }

        // Hide terminal cursor - we use software cursor (█) instead
        execute!(io::stdout(), Hide)?;

        // Initialize key handler registry with all mode handlers
        let mut key_registry = KeyHandlerRegistry::new();
        create_reading_handlers(&mut key_registry);
        create_command_handlers(&mut key_registry);
        create_popup_handlers(&mut key_registry);

        Ok(TuiManager {
            terminal,
            kitty_renderer: renderer,
            cursor: CursorBlinker::new(),
            autocomplete: AutocompleteController::new(),
            key_registry,
        })
    }

    pub fn run_event_loop(&mut self, app: &mut App) -> Result<AppMode> {
        let mut last_tick = Instant::now();
        let render_tick = Duration::from_millis(1000 / 60);

        // Force initial render before entering loop
        self.terminal.clear()?;
        self.terminal.flush()?;
        self.render_frame(app)?;

        loop {
            if app.mode() == AppMode::Quit {
                return Ok(AppMode::Quit);
            }

            // Cursor blink — only in Command mode
            let mut needs_redraw = false;
            if app.mode() == AppMode::Command {
                needs_redraw = self.cursor.tick();
            }

            // Short timeout in Command mode for responsive input/cursor;
            // token duration in Reading/Paused drives word auto-advance.
            let poll_timeout = if app.mode() == AppMode::Command {
                Duration::from_millis(50)
            } else {
                Duration::from_millis(app.get_current_token_duration().max(16))
            };

            match event::poll(poll_timeout) {
                Ok(true) => match event::read()? {
                    Event::Key(key) => {
                        // Global shortcuts (every mode)
                        if key.modifiers.contains(event::KeyModifiers::CONTROL) {
                            match key.code {
                                KeyCode::Char('c') => {
                                    app.quit();
                                    return Ok(AppMode::Quit);
                                }
                                KeyCode::Char('p') => {
                                    app.toggle_popup();
                                    continue;
                                }
                                // Ctrl+R refreshes the autocomplete file cache (active session only)
                                KeyCode::Char('r') if self.autocomplete.is_active() => {
                                    self.autocomplete.refresh();
                                    continue;
                                }
                                _ => {}
                            }
                        }

                        match (key.code, app.mode()) {
                            // Command mode: drive autocomplete inline + forward chars to the buffer
                            (KeyCode::Char(c), AppMode::Command) => {
                                let activated = {
                                    let buf = app.command_buffer();
                                    self.autocomplete.try_activate(c, buf, buf.len())
                                };

                                app.push_command_char(c);
                                self.cursor.on_key();

                                // Feed the autocomplete query, except right after @-activation
                                // (the query should stay empty until the next char).
                                if !activated {
                                    self.autocomplete.feed_char(c);
                                }
                            }
                            // Reading/Paused: ':' and '@' open the command deck
                            // directly (matches the deck hint "Type @ for files,
                            // @@, or :q") instead of being swallowed by the
                            // key registry — lets the user quit / change file
                            // without knowing Tab first.
                            (KeyCode::Char(c), AppMode::Reading | AppMode::Paused)
                                if c == ':' || c == '@' =>
                            {
                                app.set_mode(AppMode::Command);
                                let activated = {
                                    let buf = app.command_buffer();
                                    self.autocomplete.try_activate(c, buf, buf.len())
                                };
                                app.push_command_char(c);
                                self.cursor.on_key();
                                if !activated {
                                    self.autocomplete.feed_char(c);
                                }
                            }
                            // Popup / Reading / Paused: route every char through the registry
                            (KeyCode::Char(_), _) => {
                                self.dispatch_key(key.code, app);
                            }
                            (KeyCode::Enter, AppMode::Command) => {
                                if self.autocomplete.is_active() {
                                    self.autocomplete.apply_and_close(app.command_buffer_mut());
                                }
                                // Execute unless the popup is still active (no
                                // selection to apply) — typing @file + Enter must
                                // work in one keypress, not two.
                                if !self.autocomplete.is_active()
                                    && self.dispatch_key(KeyCode::Enter, app)
                                    && app.mode() == AppMode::Quit
                                {
                                    return Ok(AppMode::Quit);
                                }
                            }
                            (KeyCode::Backspace, AppMode::Command) => {
                                if self.autocomplete.is_active() {
                                    self.autocomplete.backspace();
                                }
                                self.dispatch_key(KeyCode::Backspace, app);
                            }
                            (KeyCode::Up, AppMode::Command) => {
                                if self.autocomplete.is_active() {
                                    self.autocomplete.select_previous();
                                }
                            }
                            (KeyCode::Down, AppMode::Command) => {
                                if self.autocomplete.is_active() {
                                    self.autocomplete.select_next();
                                }
                            }
                            (KeyCode::Tab, AppMode::Command) => {
                                if self.autocomplete.is_active() {
                                    self.autocomplete.apply_and_chain(app.command_buffer_mut());
                                } else if app.reading_state().is_some() {
                                    app.set_mode(AppMode::Reading);
                                }
                            }
                            (KeyCode::Tab, AppMode::Reading | AppMode::Paused) => {
                                app.set_mode(AppMode::Command);
                                app.clear_command_buffer();
                            }
                            (KeyCode::Esc, _) => {
                                if self.autocomplete.is_active() {
                                    self.autocomplete.deactivate();
                                } else {
                                    self.dispatch_key(KeyCode::Esc, app);
                                }
                            }
                            // Keys whose effect is Command-only: no-op elsewhere (preserves prior behavior)
                            (
                                KeyCode::Enter
                                | KeyCode::Backspace
                                | KeyCode::Up
                                | KeyCode::Down
                                | KeyCode::Tab,
                                _,
                            ) => {}
                            // Any other key in Reading/Paused/Popup goes through the registry
                            (_, AppMode::Reading | AppMode::Paused | AppMode::Popup) => {
                                self.dispatch_key(key.code, app);
                            }
                            _ => {}
                        }
                    }
                    Event::Resize(cols, rows) => {
                        let _ = self.handle_resize(cols, rows, app);
                    }
                    _ => {}
                },
                Ok(false) => {
                    // Auto-advance only while actively Reading
                    if app.mode() == AppMode::Reading && !app.advance_reading() {
                        app.set_mode(AppMode::Paused);
                    }
                }
                Err(e) => return Err(e.into()),
            }

            // Drain any file-discovery results into the autocomplete state
            self.autocomplete.poll();

            if last_tick.elapsed() >= render_tick || needs_redraw {
                self.render_frame(app)?;
                last_tick = Instant::now();
            }
        }
    }

    /// Forward a key to the registry; log handler errors to `app`. Returns true if consumed.
    fn dispatch_key(&mut self, code: KeyCode, app: &mut App) -> bool {
        let mode = app.mode();
        match self.key_registry.dispatch(code, mode, app) {
            Some(Ok(KeyResult::Consumed)) => true,
            None => false,
            Some(Err(e)) => {
                app.set_error(format!("Key handler error: {}", e));
                false
            }
        }
    }

    /// Non-blocking drain of the file-discovery channel into autocomplete state.
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

    pub fn render_frame(&mut self, app: &mut App) -> Result<()> {
        // Rounded background card: one opaque rounded rect behind the text
        // (z=-1), re-transmitted only when dims or theme change.
        let theme = Theme::get_by_name(app.theme_name());
        if let Color::Rgb(r, g, b) = theme.background {
            let _ = self.kitty_renderer.render_background(Rgba([r, g, b, 255]));
        }

        // Reader area (pixel rect) is shared with the KGP layer for the macro gutter.
        let reader_area = self.calculate_reader_area();
        // Kitty layer FIRST, TUI layer LAST: ratatui's draw() flushes stdout once
        // at the end, so all kitty placements/deletes reach the terminal in ONE
        // write batch → one repaint per frame → no word flicker.
        self.render_kitty_layer(app, reader_area)?;
        self.render_tui_layer(app)?;
        Ok(())
    }

    /// Ratatui widget tree: background, margins, WPM, command deck, autocomplete
    /// and config popups. Pure cell-based rendering via the crossterm backend.
    fn render_tui_layer(&mut self, app: &mut App) -> Result<()> {
        self.terminal.draw(|frame| {
            let area = frame.area();
            set_current_theme(app.theme_name());
            let theme = Theme::get_by_name(app.theme_name());

            // Background fill with rounded corners: the kitty layer placed an
            // opaque rounded rect BEHIND the text (z=i32::MIN), so ratatui must
            // leave the 4 screen-corner cells unpainted or it would square them.
            render_background_cells(frame, area, theme.background);

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

            // Render WPM display in top-left of reading area
            let wpm = app.get_wpm();
            render_wpm(frame, reading_area, wpm, &theme);

            render_command_deck(
                frame,
                command_area,
                app.mode(),
                app.command_buffer(),
                app.get_error(),
                self.cursor.visible() && app.mode() == AppMode::Command, // Only blink cursor in Command mode
                // Active = Command (typing) or Paused (deck is the interaction
                // surface — ':' / '@' open it). Dimmed only while Reading.
                matches!(app.mode(), AppMode::Command | AppMode::Paused),
            );
            // Render autocomplete popup if active
            let terminal_height = frame.area().height;
            render_autocomplete_popup(
                frame,
                self.autocomplete.state(),
                command_area,
                terminal_height,
            );

            // Render config popup if open
            render_config_popup(frame, &app.config_popup, command_area, terminal_height);
        })?;

        // No explicit flush here: Terminal::draw flushes the buffer when it
        // completes, and render_frame runs the kitty layer first so this single
        // flush pushes the whole frame out in one batch.
        Ok(())
    }

    /// Kitty Graphics Protocol layer: place the current word (with ghost
    /// context) via ab_glyph rasterization, the sentence-progress micro-bar and
    /// the document-progress macro gutter, then delete the PREVIOUS frame's
    /// images. Place-then-delete in one write batch = one repaint per frame.
    fn render_kitty_layer(&mut self, app: &mut App, reader_area: Rect) -> Result<()> {
        // Skip rendering (and clearing) pure whitespace/newline tokens so the
        // previous word stays visible — words run together instead of blanking.
        let word = app.get_current_word();
        let should_render = word
            .as_ref()
            .map(|w| !w.trim().is_empty() && w.trim() != "\n" && w.trim() != "\r\n")
            .unwrap_or(false);

        if !should_render {
            return Ok(());
        }

        let word = word.expect("should_render implies a word");

        // Use precomputed char_count from token for O(1) anchor calculation
        // instead of O(n) chars().count() on the word string
        let (anchor_pos, ghost_prev, ghost_next) = if let Some(reading_state) = app.reading_state()
        {
            let anchor = if let Some(token) = reading_state.current_token() {
                calculate_anchor_position_from_len(token.char_count())
            } else {
                calculate_anchor_position(&word)
            };
            // Get ghost context only if enabled in config
            let (prev, next) = if app.ghost_words_enabled() {
                reading_state.ghost_context()
            } else {
                (None, None)
            };
            (anchor, prev, next)
        } else {
            (calculate_anchor_position(&word), None, None)
        };

        // Create render frame with ghost context
        let frame = RenderFrame::with_ghosts(&word, anchor_pos, ghost_prev, ghost_next);

        // Render frame (current word + optional ghost words)
        if let Err(e) = RsvpRenderer::render_frame(&mut self.kitty_renderer, &frame) {
            app.set_error(format!("Render error: {}", e));
        }

        // Render progress bar and macro gutter
        if let Some(reading_state) = app.reading_state() {
            let current_index = reading_state.current_index();
            let total_tokens = reading_state.tokens().len();
            let paused = matches!(app.mode(), AppMode::Paused);

            let progress = calculate_sentence_progress(current_index, reading_state.tokens());
            let word_y = self.kitty_renderer.get_vertical_center().unwrap_or(0);
            // Constant line height (not per-word glyph height): the bar's Y
            // position must not jitter between words of different heights.
            let word_height = self.kitty_renderer.word_line_height();
            // Render micro bar (sentence progress) + macro gutter (document progress)
            if let Err(e) = self.kitty_renderer.render_progress(
                word_y,
                word_height,
                progress,
                paused,
                current_index,
                total_tokens,
                reader_area,
            ) {
                app.set_error(format!("Progress render error: {}", e));
            }
        }

        // Delete the PREVIOUS frame's images (the current frame is already
        // placed on top of them, so the swap is invisible). Skipped entirely
        // for blank tokens, keeping the last word on screen.
        if let Err(e) = RsvpRenderer::clear(&mut self.kitty_renderer) {
            app.set_error(format!("Failed to clear previous word: {}", e));
        }

        Ok(())
    }

    /// Handle terminal resize events
    ///
    /// Updates viewport dimensions and redraws the current word at the new center position.
    /// Auto-pauses reading during resize to prevent visual artifacts (per Design Doc Section 8.1).
    fn handle_resize(&mut self, cols: u16, rows: u16, app: &mut App) -> Result<()> {
        // Auto-pause if currently reading to prevent visual artifacts
        let was_reading = app.mode() == AppMode::Reading;
        if was_reading {
            app.toggle_pause();
        }

        // Update ratatui's internal terminal size to match actual terminal
        // This ensures frame.area() returns correct dimensions for layout calculations
        self.terminal.autoresize()?;

        // Update viewport dimensions using the resize event data.
        // Cell size (px per cell) is constant across resizes — only the number of
        // cols/rows changes — so extrapolate pixel size from the previous cell
        // size instead of querying the terminal (stdin queries deadlock the
        // crossterm event reader and are never used).
        self.kitty_renderer
            .viewport()
            .update_from_resize(cols, rows);

        // Force immediate redraw (render_frame places the new frame and clears
        // the previous one internally).
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

        // Ensure cursor is visible on exit
        let _ = execute!(io::stdout(), Show);

        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen);
    }
}
