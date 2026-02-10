//! Autocomplete popup rendering
//!
//! Renders the file autocomplete popup using ratatui widgets.

use super::state::AutocompleteState;
use super::{get_file_prefix, MAX_VISIBLE_ITEMS};
use crate::ui::theme::colors;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph},
    Frame,
};
use std::path::Path;

/// Render the autocomplete popup
///
/// # Arguments
/// * `frame` - The ratatui frame to render to
/// * `state` - The current autocomplete state
/// * `command_area` - The area of the command deck (for positioning)
/// * `terminal_height` - Total terminal height (for dynamic placement)
pub fn render_autocomplete_popup(
    frame: &mut Frame,
    state: &AutocompleteState,
    command_area: Rect,
    terminal_height: u16,
) {
    if !state.active {
        return;
    }

    let popup_area = calculate_popup_area(state, command_area, terminal_height);

    // Clear the popup area first
    frame.render_widget(Clear, popup_area);

    // Render the popup content
    if state.is_scanning && state.files.is_empty() {
        render_scanning_indicator(frame, popup_area);
    } else {
        render_file_list(frame, state, popup_area);
    }
}

/// Calculate the popup area based on state and available space
fn calculate_popup_area(
    state: &AutocompleteState,
    command_area: Rect,
    terminal_height: u16,
) -> Rect {
    const MIN_HEIGHT: u16 = 3;
    const MAX_HEIGHT: u16 = 12;

    let match_count = state.match_count();
    let popup_height = if (state.is_scanning && state.files.is_empty()) || match_count == 0 {
        // MIN_HEIGHT: borders (2) + content (1) + help text (1) = 4, but let's use 5 for comfort
        5
    } else {
        // List: borders (2) + items (N) + footer (1 or 2)
        let remaining_count = match_count.saturating_sub(MAX_VISIBLE_ITEMS);
        let footer_height = if remaining_count > 0 { 2 } else { 1 };
        let content_height = (match_count.min(MAX_VISIBLE_ITEMS) + 2 + footer_height) as u16;
        content_height.clamp(MIN_HEIGHT, MAX_HEIGHT)
    };

    let popup_width = command_area.width.saturating_sub(2);

    // Try to position above command deck first
    let space_above = command_area.y;
    let space_below = terminal_height.saturating_sub(command_area.y + command_area.height);

    let y = if space_above >= popup_height {
        // Position above
        command_area.y.saturating_sub(popup_height)
    } else if space_below >= popup_height {
        // Position below
        command_area.y + command_area.height
    } else {
        // Default to above, clamp height
        command_area.y.saturating_sub(space_above.min(popup_height))
    };

    Rect::new(
        command_area.x + 1,
        y,
        popup_width,
        popup_height.min(space_above.max(space_below)),
    )
}

/// Render a "Scanning..." indicator
fn render_scanning_indicator(frame: &mut Frame, area: Rect) {
    let block = Block::default()
        .title("FILES")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(colors::text()))
        .style(Style::default().bg(colors::surface()));

    let paragraph = Paragraph::new("Scanning...")
        .block(block)
        .alignment(Alignment::Center)
        .style(Style::default().fg(colors::text()).bg(colors::surface()));

    frame.render_widget(paragraph, area);
}

/// Render the file list
fn render_file_list(frame: &mut Frame, state: &AutocompleteState, area: Rect) {
    let match_count = state.match_count();

    // Build title
    let title = if state.is_scanning {
        format!("FILES ({} matches, scanning...)", match_count)
    } else {
        format!("FILES ({} matches)", match_count)
    };

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(colors::text()))
        .style(Style::default().bg(colors::surface()));

    if match_count == 0 {
        // Split area for empty state: main content + help text
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(1), Constraint::Length(1)])
            .split(area);
        
        let paragraph = Paragraph::new("No files found")
            .block(block)
            .alignment(Alignment::Center)
            .style(Style::default().fg(colors::text()).bg(colors::surface()));
        frame.render_widget(paragraph, chunks[0]);
        
        // Render help text in dedicated area
        render_help_text(frame, chunks[1]);
        return;
    }

    // Calculate footer height needed
    let remaining_count = match_count.saturating_sub(MAX_VISIBLE_ITEMS);
    let footer_height = if remaining_count > 0 { 2 } else { 1 };

    // Split area into list section and footer section
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(footer_height)])
        .split(area);

    let list_area = chunks[0];
    let footer_area = chunks[1];

    // Build list items
    let items: Vec<ListItem> = (state.scroll_offset..)
        .take(MAX_VISIBLE_ITEMS)
        .filter_map(|filtered_idx| state.get_filtered_file(filtered_idx))
        .map(|file| render_file_item(file))
        .collect();

    // Calculate which visible item is selected (0-based index within visible items)
    let visible_selected_idx = state.selected_idx.saturating_sub(state.scroll_offset);
    let mut list_state = ListState::default();
    list_state.select(Some(visible_selected_idx));

    let list = List::new(items)
        .block(block)
        .highlight_style(
            Style::default()
                .bg(colors::anchor())
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("> ");

    frame.render_stateful_widget(list, list_area, &mut list_state);

    // Render footer content (truncation indicator + help text)
    render_footer(frame, footer_area, remaining_count);
}

/// Render the footer with truncation indicator and help text
fn render_footer(frame: &mut Frame, area: Rect, remaining_count: usize) {
    if remaining_count > 0 {
        // Split footer into two lines: truncation indicator + help text
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(1), Constraint::Length(1)])
            .split(area);

        // Truncation indicator
        let truncation_text = format!("(+{} more)", remaining_count);
        let truncation_style = Style::default()
            .fg(colors::text())
            .bg(colors::surface())
            .add_modifier(Modifier::ITALIC);

        let truncation_widget = Paragraph::new(truncation_text)
            .alignment(Alignment::Right)
            .style(truncation_style);
        frame.render_widget(truncation_widget, chunks[0]);

        // Help text
        render_help_text(frame, chunks[1]);
    } else {
        // Just help text
        render_help_text(frame, area);
    }
}

/// Render the help text
fn render_help_text(frame: &mut Frame, area: Rect) {
    let help_text = "↑↓ navigate • Enter select • Tab complete • Esc close • Ctrl+R refresh";
    let help_style = Style::default()
        .fg(colors::text())
        .bg(colors::surface())
        .add_modifier(Modifier::DIM);

    let help_widget = Paragraph::new(help_text)
        .alignment(Alignment::Center)
        .style(help_style);

    frame.render_widget(help_widget, area);
}

/// Render a single file item
fn render_file_item(file: &Path) -> ListItem<'_> {
    let prefix = get_file_prefix(file);
    let display_name = format_file_name(file);

    let content = format!("{} {}", prefix, display_name);
    let style = Style::default().fg(colors::text());

    ListItem::new(Line::from(vec![Span::styled(content, style)]))
}

/// Format a file path for display
///
/// Shows the relative path, truncating if too long
fn format_file_name(file: &Path) -> String {
    const MAX_WIDTH: usize = 60;

    // Try to get relative path from current directory
    let display_path = if let Ok(current_dir) = std::env::current_dir() {
        file.strip_prefix(&current_dir)
            .unwrap_or(file)
            .to_string_lossy()
            .to_string()
    } else {
        file.to_string_lossy().to_string()
    };

    if display_path.len() <= MAX_WIDTH {
        display_path
    } else {
        // Truncate from the start, keep the end
        let start = display_path.len().saturating_sub(MAX_WIDTH - 3);
        format!("...{}", &display_path[start..])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_file_name_short() {
        let path = Path::new("doc.pdf");
        assert_eq!(format_file_name(path), "doc.pdf");
    }

    #[test]
    fn test_format_file_name_truncates_long() {
        let long_name = "a".repeat(100);
        let path = Path::new(&long_name);
        let formatted = format_file_name(path);
        assert!(formatted.len() <= 63); // MAX_WIDTH + 3 for "..."
        assert!(formatted.starts_with("..."));
    }
}
