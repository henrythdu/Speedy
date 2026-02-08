use crate::app::mode::AppMode;
use crate::ui::theme::colors;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::Style,
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

pub fn render_command_deck(
    frame: &mut Frame,
    area: Rect,
    mode: AppMode,
    command_buffer: &str,
    error_message: Option<&str>,
) {
    // Clear the command area first
    frame.render_widget(Clear, area);

    // Create layout with left accent bar and input area
    let layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(1), Constraint::Min(1)])
        .split(area);

    // Left accent bar
    let accent_bar =
        Paragraph::new("▌").style(Style::default().fg(colors::anchor()).bg(colors::surface()));
    frame.render_widget(accent_bar, layout[0]);

    // Command input area
    let mode_indicator = match mode {
        AppMode::Command => " COMMAND ",
        AppMode::Reading => " READING ",
        AppMode::Paused => " PAUSED ",
        AppMode::Quit => " QUIT ",
    };

    let input_text = if let Some(error) = error_message {
        // Show error in red
        format!("{} ERROR: {}", mode_indicator, error)
    } else if command_buffer.is_empty() {
        format!("{} Type @file.pdf, @@, or :q", mode_indicator)
    } else {
        format!("{} {}", mode_indicator, command_buffer)
    };

    let text_color = if error_message.is_some() {
        colors::anchor() // Use anchor color (red) for errors
    } else {
        colors::text()
    };

    let input_widget = Paragraph::new(input_text)
        .block(
            Block::default()
                .borders(Borders::TOP)
                .border_style(Style::default().fg(colors::dimmed())),
        )
        .style(Style::default().fg(text_color).bg(colors::surface()));

    frame.render_widget(input_widget, layout[1]);
}

