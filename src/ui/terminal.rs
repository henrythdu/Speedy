use crate::app::{mode::AppMode, App};
use crate::engine::wpm_to_milliseconds;
use crate::ui::reader::view::{
    render_command_deck, render_context_left, render_context_right, render_gutter_placeholder,
    render_placeholder, render_progress_bar, render_word_display,
};
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    Terminal,
};
use std::io::{self, Stdout};
use std::time::{Duration, Instant};

pub struct TuiManager {
    terminal: Terminal<CrosstermBackend<Stdout>>,
}

impl TuiManager {
    pub fn new() -> Result<Self, io::Error> {
        enable_raw_mode()?;
        execute!(io::stdout(), EnterAlternateScreen)?;

        let backend = CrosstermBackend::new(io::stdout());
        let terminal = Terminal::new(backend)?;

        Ok(TuiManager { terminal })
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
                        if key.code == KeyCode::Char('c') && key.modifiers.contains(event::KeyModifiers::CONTROL) {
                            app.set_mode(AppMode::Quit);
                            return Ok(AppMode::Quit);
                        }
                        
                        match key.code {
                            KeyCode::Char(c) => {
                                app.handle_keypress(c);
                            }
                            KeyCode::Enter => {
                                // TODO: Execute command when in Command mode
                            }
                            KeyCode::Esc => {
                                if app.mode() == AppMode::Reading || app.mode() == AppMode::Paused {
                                    app.set_mode(AppMode::Command);
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

        self.terminal.draw(|frame| {
            let area = frame.area();

            // Split screen: Reading zone (top 85%) + Command deck (bottom 15%)
            let main_layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(85), Constraint::Percentage(15)])
                .split(area);

            let reading_area = main_layout[0];
            let command_area = main_layout[1];

            // Reading zone: Left context | Center word | Right context | Gutter
            let reading_layout = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(35),  // Left context
                    Constraint::Percentage(30),  // Center word
                    Constraint::Percentage(32),  // Right context
                    Constraint::Percentage(3),   // Gutter
                ])
                .split(reading_area);

            // Render left context
            let left_context =
                render_context_left(&render_state.tokens, render_state.current_index, 3);
            frame.render_widget(left_context, reading_layout[0]);

            // Render center word with OVP anchoring
            if let Some(word) = &render_state.current_word {
                let anchor_pos = crate::reading::calculate_anchor_position(word);
                let word_display = render_word_display(word, anchor_pos);
                frame.render_widget(word_display, reading_layout[1]);
            } else {
                // Show placeholder when no content loaded
                let placeholder = render_placeholder();
                frame.render_widget(placeholder, reading_layout[1]);
            }

            // Render right context
            let right_context =
                render_context_right(&render_state.tokens, render_state.current_index, 3);
            frame.render_widget(right_context, reading_layout[2]);

            // Render gutter
            let gutter = render_gutter_placeholder();
            frame.render_widget(gutter, reading_layout[3]);

            // Command deck area
            render_command_deck(frame, command_area, app.mode());
        })?;

        Ok(())
    }
}

impl Drop for TuiManager {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen);
    }
}
