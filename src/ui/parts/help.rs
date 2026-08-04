//! Help part — the `:help` / `:h` reference overlay.
//!
//! A static, rounded popup (mirroring the config popup) holding everything a
//! new user needs to find: the keybindings, what the two progress indicators
//! mean, how to load files, and where the theme settings live. Esc/Enter/q
//! dismiss it (see the Help arm in terminal.rs' event loop — Help is a mode,
//! so the word freezes while it's open).

use crate::ui::theme::colors;
use ratatui::{
    layout::{Alignment, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph},
    Frame,
};

/// Render the help popup above the command deck.
pub fn render_help_popup(frame: &mut Frame, command_area: Rect, terminal_height: u16) {
    let area = calculate_help_area(command_area, terminal_height);

    let block = Block::default()
        .title(" HELP ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(colors::accent()))
        .style(Style::default().bg(colors::surface()));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let mut lines: Vec<Line> = Vec::new();
    section(&mut lines, "READING");
    info(&mut lines, "Space / p", "pause / resume");
    info(&mut lines, "j / k", "previous / next sentence");
    info(&mut lines, "[ / ]", "slower / faster");
    blank(&mut lines);
    section(&mut lines, "DECK — active when paused");
    info(&mut lines, "@ path", "open a file, pick with ↑↓");
    info(&mut lines, "@@", "paste from clipboard");
    info(&mut lines, ":q", "quit");
    info(&mut lines, ":h", "this help");
    info(&mut lines, "Esc", "back to reading");
    blank(&mut lines);
    section(&mut lines, "PROGRESS");
    info(&mut lines, "bar", "progress in current sentence");
    info(&mut lines, "right gutter", "progress in whole document");
    blank(&mut lines);
    section(&mut lines, "THEMES");
    info(&mut lines, "Ctrl+P", "settings (theme, WPM, ghosts)");

    let paragraph = Paragraph::new(lines).alignment(Alignment::Left);
    frame.render_widget(paragraph, inner);
}

/// Position above the command deck, clamped to the available space.
/// Width fits the content (~52) instead of spanning the whole command area,
/// so no line can run into the right border on any terminal.
fn calculate_help_area(command_area: Rect, terminal_height: u16) -> Rect {
    const POPUP_HEIGHT: u16 = 20;
    const POPUP_WIDTH: u16 = 52;

    let popup_width = command_area.width.clamp(40, POPUP_WIDTH);
    let space_above = command_area.y;
    let y = if space_above >= POPUP_HEIGHT {
        command_area.y.saturating_sub(POPUP_HEIGHT)
    } else {
        0
    };
    let _ = terminal_height; // clamped to top; deck is at the bottom on small terms
    Rect::new(
        command_area.x,
        y,
        popup_width,
        POPUP_HEIGHT.min(space_above.max(8)),
    )
}

/// A dim section header line.
fn section(lines: &mut Vec<Line>, title: &str) {
    lines.push(Line::from(Span::styled(
        format!("  {}", title),
        Style::default()
            .fg(colors::accent())
            .add_modifier(Modifier::BOLD),
    )));
}

/// One `key + description` line, keys accented.
fn info(lines: &mut Vec<Line>, key: &str, desc: &str) {
    lines.push(Line::from(vec![
        Span::styled(
            format!("    {:16}", key),
            Style::default().fg(colors::accent()),
        ),
        Span::styled(desc.to_string(), Style::default().fg(colors::text())),
    ]));
}

fn blank(lines: &mut Vec<Line>) {
    lines.push(Line::from(""));
}
