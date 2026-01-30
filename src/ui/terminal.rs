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
    widgets::{Block, Clear},
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

        // Query terminal dimensions and set reading zone center
        let _ = renderer.viewport().query_dimensions();
        if let Some(dims) = renderer.viewport().get_dimensions() {
            let center_x = dims.pixel_size.0 / 2;
            let center_y = (dims.pixel_size.1 as f32 * 0.42) as u32; // 42% of screen height per PRD
            renderer.set_reading_zone_center(center_x, center_y);

            // Calculate font size for 5-line height (using cell height * 5)
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
                    if let Event::Key(key) = event::read()? {
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
                                if app.mode() == AppMode::Command && !self.command_buffer.is_empty()
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
                                                    eprintln!("Failed to load clipboard: {}", e);
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
                                if app.mode() == AppMode::Reading || app.mode() == AppMode::Paused {
                                    app.set_mode(AppMode::Command);
                                    self.command_buffer.clear();
                                }
                            }
                            _ => {}
                        }
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
        let render_state = app.get_render_state();

        // Render word via Kitty Graphics Protocol using composite rendering
        // Per Epic 2 composite rendering implementation: Single-image-per-frame approach
        // This uses render_frame() which orchestrates: create_canvas → composite_word → transmit
        if let Some(word) = &render_state.current_word {
            let anchor_pos = crate::reading::calculate_anchor_position(word);

            // Use the new render_frame orchestrator (handles canvas creation, compositing, transmission)
            if let Err(e) = self.kitty_renderer.render_frame(word, anchor_pos) {
                eprintln!("Render error: {}", e);
            }
        }

        // Always render via Ratatui for UI (commands, etc.)
        self.terminal.draw(|frame| {
            let area = frame.area();

            // Split screen: Reading zone (top 85%) + Command deck (bottom 15%)
            let main_layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(85), Constraint::Percentage(15)])
                .split(area);

            let reading_area = main_layout[0];
            let command_area = main_layout[1];

            // Fill reading zone with theme background color
            let theme = Theme::midnight();
            let reading_bg = Block::default().style(Style::default().bg(theme.background));
            frame.render_widget(reading_bg, reading_area);

            // Command deck area
            render_command_deck(frame, command_area, app.mode(), &self.command_buffer);
        })?;

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
