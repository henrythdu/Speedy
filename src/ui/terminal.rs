use crate::app::{mode::AppMode, App};
use crate::engine::wpm_to_milliseconds;
use crate::ui::reader::view::{
    render_context_left, render_context_right, render_gutter_placeholder, render_progress_bar,
    render_word_display,
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
            if current_mode == AppMode::Quit || current_mode == AppMode::Command {
                return Ok(current_mode);
            }
            // Reading and Paused both stay in TUI

            let wpm = app.get_wpm();
            let timeout_ms = wpm_to_milliseconds(wpm);
            let poll_timeout = Duration::from_millis(timeout_ms);

            match event::poll(poll_timeout) {
                Ok(true) => {
                    if let Event::Key(key) = event::read()? {
                        if let KeyCode::Char(c) = key.code {
                            app.handle_keypress(c);
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

            // Main content area (top 90%)
            let main_area = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(90), Constraint::Length(1)])
                .split(area)[0];

            // Split main content: left context, word, right context
            let content_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(40),
                    Constraint::Length(30),
                    Constraint::Percentage(40),
                ])
                .split(main_area);

            let left_context =
                render_context_left(&render_state.tokens, render_state.current_index, 1);
            frame.render_widget(left_context, content_chunks[0]);

            if let Some(word) = &render_state.current_word {
                let anchor_pos = crate::reading::calculate_anchor_position(word);
                let word_display = render_word_display(word, anchor_pos);
                frame.render_widget(word_display, content_chunks[1]);
            }

            let right_context =
                render_context_right(&render_state.tokens, render_state.current_index, 1);
            frame.render_widget(right_context, content_chunks[2]);

            // Progress bar at bottom of main area
            let progress_bar = render_progress_bar(render_state.progress);
            let progress_area = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(1), Constraint::Length(1)])
                .split(main_area)[1];
            frame.render_widget(progress_bar, progress_area);

            // Gutter on far right (3% of full width)
            let gutter = render_gutter_placeholder();
            let gutter_area = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(97), Constraint::Percentage(3)])
                .split(area)[1];
            frame.render_widget(gutter, gutter_area);
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
