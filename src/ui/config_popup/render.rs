//! Config popup rendering
//!
//! Renders the config picker popup (Ctrl+P) for editing default WPM, theme, and ghost words.

use super::state::ConfigPopupState;
use crate::ui::theme::colors;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Paragraph},
    Frame,
};

/// Render the config popup
///
/// # Arguments
/// * `frame` - The ratatui frame to render to
/// * `state` - The current config popup state
/// * `command_area` - The area of the command deck (for positioning)
/// * `terminal_height` - Total terminal height (for dynamic placement)
pub fn render_config_popup(
    frame: &mut Frame,
    state: &ConfigPopupState,
    command_area: Rect,
    terminal_height: u16,
) {
    if !state.is_open {
        return;
    }

    let popup_area = calculate_popup_area(command_area, terminal_height);

    // Clear the popup area first
    frame.render_widget(Clear, popup_area);

    // Render the popup content
    render_popup_content(frame, state, popup_area);
}

/// Calculate the popup area based on available space
///
/// Positions the popup above the command deck, with a fixed height of 7 lines:
/// - 1 border top
/// - 1 header
/// - 3 config rows
/// - 1 footer
/// - 1 border bottom
fn calculate_popup_area(command_area: Rect, terminal_height: u16) -> Rect {
    const POPUP_HEIGHT: u16 = 7;
    const MIN_WIDTH: u16 = 40;

    let popup_width = command_area.width.max(MIN_WIDTH);

    // Try to position above command deck
    let space_above = command_area.y;
    let space_below = terminal_height.saturating_sub(command_area.y + command_area.height);

    let y = if space_above >= POPUP_HEIGHT {
        // Position above
        command_area.y.saturating_sub(POPUP_HEIGHT)
    } else if space_below >= POPUP_HEIGHT {
        // Position below
        command_area.y + command_area.height
    } else {
        // Default to above, clamp to available space
        command_area.y.saturating_sub(space_above.min(POPUP_HEIGHT))
    };

    Rect::new(command_area.x, y, popup_width, POPUP_HEIGHT)
}

/// Render the popup content with header, rows, and footer
fn render_popup_content(frame: &mut Frame, state: &ConfigPopupState, area: Rect) {
    let block = Block::default()
        .title(" CONFIG ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(colors::accent()))
        .style(Style::default().bg(colors::surface()));

    // Layout: border padding (implicit) + content
    // Content: header (1) + rows (3) + footer (1)
    let inner_area = block.inner(area);
    frame.render_widget(block, area);

    // Split inner area into header, rows, and footer
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Header
            Constraint::Length(3), // Rows (3 lines)
            Constraint::Length(1), // Footer
        ])
        .split(inner_area);

    // Render header
    render_header(frame, chunks[0]);

    // Split rows area into 3 equal parts
    let row_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .split(chunks[1]);

    // Render config rows
    render_row(
        frame,
        row_chunks[0],
        "Default WPM:",
        &format!("{}", state.temp_default_wpm),
        state.selected_row == 0,
    );
    render_row(
        frame,
        row_chunks[1],
        "Theme:",
        state.current_theme(),
        state.selected_row == 1,
    );
    render_row(
        frame,
        row_chunks[2],
        "Ghost Words:",
        if state.temp_ghost_words { "on" } else { "off" },
        state.selected_row == 2,
    );

    // Render footer
    render_footer(frame, chunks[2]);
}

/// Render the header line
fn render_header(frame: &mut Frame, area: Rect) {
    let header_text = Line::from(vec![
        Span::styled(
            "Edit settings with ",
            Style::default()
                .fg(colors::text())
                .bg(colors::surface())
                .add_modifier(Modifier::DIM),
        ),
        Span::styled(
            "←→",
            Style::default()
                .fg(colors::accent())
                .bg(colors::surface())
                .add_modifier(Modifier::BOLD),
        ),
    ]);

    let header = Paragraph::new(header_text).alignment(Alignment::Left);

    frame.render_widget(header, area);
}

/// Render a single config row
///
/// # Arguments
/// * `frame` - The ratatui frame
/// * `area` - The area for this row
/// * `label` - The row label (e.g., "Default WPM:")
/// * `value` - The current value display
/// * `selected` - Whether this row is currently selected
fn render_row(frame: &mut Frame, area: Rect, label: &str, value: &str, selected: bool) {
    let (fg_color, bg_color) = if selected {
        (Color::Black, colors::accent())
    } else {
        (colors::text(), colors::surface())
    };

    let style = if selected {
        Style::default()
            .fg(fg_color)
            .bg(bg_color)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(fg_color).bg(bg_color)
    };

    // Create row content with label and value
    let row_text = Line::from(vec![
        Span::styled(format!("  {:14}", label), style),
        Span::styled(value.to_string(), style),
    ]);

    let row = Paragraph::new(row_text).alignment(Alignment::Left);

    frame.render_widget(row, area);
}

/// Render the footer with navigation hints
fn render_footer(frame: &mut Frame, area: Rect) {
    let footer_text = "↑↓ navigate  •  Enter save  •  Esc cancel";
    let footer_style = Style::default()
        .fg(colors::text())
        .bg(colors::surface())
        .add_modifier(Modifier::DIM);

    let footer = Paragraph::new(footer_text)
        .alignment(Alignment::Center)
        .style(footer_style);

    frame.render_widget(footer, area);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_popup_area_above() {
        let command_area = Rect::new(0, 10, 80, 5);
        let terminal_height = 24;

        let popup = calculate_popup_area(command_area, terminal_height);

        // Should position above command deck
        assert_eq!(popup.y, 3); // 10 - 7
        assert_eq!(popup.height, 7);
        assert_eq!(popup.x, command_area.x);
    }

    #[test]
    fn test_calculate_popup_area_below() {
        let command_area = Rect::new(0, 2, 80, 5);
        let terminal_height = 24;

        let popup = calculate_popup_area(command_area, terminal_height);

        // Should position below command deck (not enough space above)
        assert_eq!(popup.y, 7); // 2 + 5
        assert_eq!(popup.height, 7);
    }

    #[test]
    fn test_calculate_popup_area_min_width() {
        let command_area = Rect::new(0, 10, 20, 5);
        let terminal_height = 24;

        let popup = calculate_popup_area(command_area, terminal_height);

        // Should use minimum width of 40
        assert_eq!(popup.width, 40);
    }
}
