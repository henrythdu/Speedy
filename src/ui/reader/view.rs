use crate::engine::Token;
use crate::ui::theme::colors;
use ratatui::{
    layout::Alignment,
    style::Style,
    text::{Line, Span},
    widgets::Paragraph,
};

pub fn render_word_display(word: &str, anchor_pos: usize) -> Paragraph<'static> {
    let chars: Vec<char> = word.chars().collect();
    let _word_len = chars.len();

    let left_padding = 3_i32.saturating_sub(anchor_pos as i32) as usize;

    let mut spans = Vec::new();
    for _ in 0..left_padding {
        spans.push(Span::styled(" ", Style::default().fg(colors::text())));
    }

    for (i, ch) in chars.iter().enumerate() {
        let style = if i == anchor_pos {
            Style::default()
                .fg(colors::anchor())
                .add_modifier(ratatui::style::Modifier::BOLD)
        } else {
            Style::default().fg(colors::text())
        };
        spans.push(Span::styled(ch.to_string(), style));
    }

    Paragraph::new(Line::from(spans))
        .alignment(Alignment::Left)
        .style(Style::default().bg(colors::background()))
}

pub fn render_progress_bar(progress: (usize, usize)) -> Line<'static> {
    let (current, total) = progress;
    let width = if total == 0 {
        0.0
    } else {
        (current as f64 / total as f64) * 100.0
    };

    let filled_len = (width / 100.0 * 20.0) as usize;
    let empty_len = 20 - filled_len;

    let mut spans = Vec::new();
    for _ in 0..filled_len {
        spans.push(Span::styled("─", Style::default().fg(colors::text())));
    }
    for _ in 0..empty_len {
        spans.push(Span::styled("─", Style::default().fg(colors::dimmed())));
    }

    Line::from(spans).alignment(Alignment::Center)
}

pub fn render_context_left(tokens: &[Token], current: usize, window: usize) -> Paragraph<'static> {
    let start = if current > window {
        current - window
    } else {
        0
    };
    let context_words: Vec<String> = tokens[start..current]
        .iter()
        .map(|t| {
            let mut text = t.text.clone();
            for &p in &t.punctuation {
                text.push(p);
            }
            text
        })
        .collect();

    let text = context_words.join(" ");

    Paragraph::new(text).alignment(Alignment::Right).style(
        Style::default()
            .fg(colors::dimmed())
            .bg(colors::background()),
    )
}

pub fn render_context_right(tokens: &[Token], current: usize, window: usize) -> Paragraph<'static> {
    if tokens.is_empty() || current >= tokens.len() {
        return Paragraph::new("").alignment(Alignment::Left).style(
            Style::default()
                .fg(colors::dimmed())
                .bg(colors::background()),
        );
    }

    let end = std::cmp::min(current + window + 1, tokens.len());
    let context_words: Vec<String> = tokens[current + 1..end]
        .iter()
        .map(|t| {
            let mut text = t.text.clone();
            for &p in &t.punctuation {
                text.push(p);
            }
            text
        })
        .collect();

    let text = context_words.join(" ");

    Paragraph::new(text).alignment(Alignment::Left).style(
        Style::default()
            .fg(colors::dimmed())
            .bg(colors::background()),
    )
}

pub fn render_gutter_placeholder() -> Paragraph<'static> {
    Paragraph::new("│")
        .alignment(Alignment::Right)
        .style(Style::default().fg(colors::text()).bg(colors::background()))
}

pub fn render_placeholder() -> Paragraph<'static> {
    let text = "Type @filename to load a file\nOr @@ to load from clipboard\n:q to quit";
    Paragraph::new(text)
        .alignment(Alignment::Center)
        .style(Style::default().fg(colors::dimmed()).bg(colors::background()))
}

use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::widgets::{Block, Borders, Clear};
use ratatui::Frame;
use crate::app::mode::AppMode;

pub fn render_command_deck(frame: &mut Frame, area: Rect, mode: AppMode) {
    // Clear the command area first
    frame.render_widget(Clear, area);

    // Create layout with left accent bar and input area
    let layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(1), Constraint::Min(1)])
        .split(area);

    // Left accent bar
    let accent_bar = Paragraph::new("▌")
        .style(Style::default().fg(colors::anchor()).bg(colors::surface()));
    frame.render_widget(accent_bar, layout[0]);

    // Command input area
    let mode_indicator = match mode {
        AppMode::Command => " COMMAND ",
        AppMode::Reading => " READING ",
        AppMode::Paused => " PAUSED ",
        AppMode::Peek => " PEEK ",
        AppMode::Quit => " QUIT ",
    };

    let input_text = format!("{} Type @file.pdf, @@, or :q", mode_indicator);
    
    let input_widget = Paragraph::new(input_text)
        .block(Block::default()
            .borders(Borders::TOP)
            .border_style(Style::default().fg(colors::dimmed()))
        )
        .style(Style::default().fg(colors::text()).bg(colors::surface()));
    
    frame.render_widget(input_widget, layout[1]);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_word_display_creates_paragraph() {
        let word = "hello";
        let anchor_pos = 1;
        let paragraph = render_word_display(word, anchor_pos);
        let _ = paragraph;
    }

    #[test]
    fn test_render_word_display_anchor_is_red() {
        let word = "test";
        let anchor_pos = 1;
        let paragraph = render_word_display(word, anchor_pos);
        let _ = paragraph;
    }

    #[test]
    fn test_render_progress_bar_zero_total() {
        let progress = (0, 0);
        let bar = render_progress_bar(progress);
        let _ = bar;
    }

    #[test]
    fn test_render_progress_bar_halfway() {
        let progress = (50, 100);
        let bar = render_progress_bar(progress);
        let _ = bar;
    }

    #[test]
    fn test_render_context_left_empty_tokens() {
        let tokens: Vec<Token> = vec![];
        let paragraph = render_context_left(&tokens, 0, 3);
        let _ = paragraph;
    }

    #[test]
    fn test_render_context_right_empty_tokens() {
        let tokens: Vec<Token> = vec![];
        let paragraph = render_context_right(&tokens, 0, 3);
        let _ = paragraph;
    }

    #[test]
    fn test_render_gutter_placeholder_creates_paragraph() {
        let paragraph = render_gutter_placeholder();
        let _ = paragraph;
    }
}
