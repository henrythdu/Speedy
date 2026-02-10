use crate::app::mode::AppMode;
use crate::ui::theme::colors;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::Style,
    widgets::{Block, Clear, Paragraph},
    Frame,
};

pub fn render_command_deck(
    frame: &mut Frame,
    area: Rect,
    mode: AppMode,
    command_buffer: &str,
    error_message: Option<&str>,
    cursor_visible: bool, // NEW: Blink state
) {
    // Clear the command area first
    frame.render_widget(Clear, area);

    // Fill entire command area with surface color
    let surface_block = Block::default().style(Style::default().bg(colors::surface()));
    frame.render_widget(surface_block, area);

    // Create layout with left accent bar and input area
    let layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(1), Constraint::Min(1)])
        .split(area);

    // Left accent bar - full height of command section
    let accent_text = "▌\n▌\n▌\n▌\n▌"; // One per row for full 5-cell height
    let accent_bar = Paragraph::new(accent_text)
        .style(Style::default().fg(colors::anchor()).bg(colors::surface()));
    frame.render_widget(accent_bar, layout[0]);

    let content_area = layout[1];

    // Add internal padding (1 cell margin inside command section)
    let padded_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(1), // Left padding
            Constraint::Fill(1),   // Content
            Constraint::Length(1), // Right padding
        ])
        .split(content_area);

    let padded_content = padded_layout[1];

    // Split content area with padding: empty row at top, input, empty row, label at bottom
    let content_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Top padding (empty row)
            Constraint::Min(0),    // Input area
            Constraint::Length(1), // Middle padding (empty row)
            Constraint::Length(1), // Label row
            Constraint::Length(1), // Bottom padding (empty row)
        ])
        .split(padded_content);

    let input_area = content_layout[1];
    let label_area = content_layout[3];

    // Render mode label at bottom
    let mode_label = match mode {
        AppMode::Command => "COMMAND",
        AppMode::Reading => "READING",
        AppMode::Paused => "PAUSED",
        AppMode::Quit => "QUIT",
    };

    let label_widget = Paragraph::new(mode_label)
        .style(Style::default().fg(colors::anchor()).bg(colors::surface()));
    frame.render_widget(label_widget, label_area);

    // Render input above label with blinking cursor
    let cursor_char = if cursor_visible { "█" } else { " " };
    let input_text = if let Some(error) = error_message {
        format!("ERROR: {}{}", error, cursor_char)
    } else if command_buffer.is_empty() {
        format!("Type @file.pdf, @@, or :q{}", cursor_char)
    } else {
        format!("{}{}", command_buffer, cursor_char)
    };

    let text_color = if error_message.is_some() {
        colors::anchor() // Use anchor color (red) for errors
    } else {
        colors::text()
    };

    // Input widget without borders (cleaner look)
    let input_widget =
        Paragraph::new(input_text).style(Style::default().fg(text_color).bg(colors::surface()));

    frame.render_widget(input_widget, input_area);
}

/// Render WPM display in the reading zone
///
/// Shows current words-per-minute in the top-left corner of the reading area.
/// Positioned with small padding (1 cell) from top-left edge.
pub fn render_wpm(frame: &mut Frame, area: Rect, wpm: u32, theme: &crate::ui::theme::Theme) {
    if wpm == 0 {
        return; // Don't render if no active reading session
    }

    // Calculate position: top-left with 1 cell padding
    let wpm_area = Rect::new(
        area.x + 1,
        area.y + 1,
        10, // Width: "999 WPM\n" is max 8 chars
        1,  // Height: single line
    );

    let wpm_text = format!("{} WPM", wpm);
    let wpm_widget =
        Paragraph::new(wpm_text).style(Style::default().fg(theme.text).bg(theme.background));

    frame.render_widget(wpm_widget, wpm_area);
}
