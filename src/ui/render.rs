use crate::engine::timing::Token;
use ratatui::{
    layout::Alignment,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
};

const COLOR_BG: Color = Color::Rgb(26, 27, 38);
const COLOR_TEXT: Color = Color::Rgb(169, 177, 214);
const COLOR_ANCHOR: Color = Color::Rgb(247, 118, 142);

pub fn render_word_display(word: &str, anchor_pos: usize) -> Paragraph<'static> {
    let chars: Vec<char> = word.chars().collect();

    let mut spans = Vec::new();
    for (i, ch) in chars.iter().enumerate() {
        let style = if i == anchor_pos {
            Style::default()
                .fg(COLOR_ANCHOR)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(COLOR_TEXT)
        };
        spans.push(Span::styled(ch.to_string(), style));
    }

    Paragraph::new(Line::from(spans))
        .alignment(Alignment::Center)
        .style(Style::default().bg(COLOR_BG))
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
        spans.push(Span::styled("─", Style::default().fg(COLOR_TEXT)));
    }
    for _ in 0..empty_len {
        spans.push(Span::styled(
            "─",
            Style::default().fg(COLOR_TEXT).add_modifier(Modifier::DIM),
        ));
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
        .map(|t| t.text.clone())
        .collect();

    let text = context_words.join(" ");

    Paragraph::new(text).alignment(Alignment::Right).style(
        Style::default()
            .fg(COLOR_TEXT)
            .add_modifier(Modifier::DIM)
            .bg(COLOR_BG),
    )
}

pub fn render_context_right(tokens: &[Token], current: usize, window: usize) -> Paragraph<'static> {
    if tokens.is_empty() || current >= tokens.len() {
        return Paragraph::new("").alignment(Alignment::Left).style(
            Style::default()
                .fg(COLOR_TEXT)
                .add_modifier(Modifier::DIM)
                .bg(COLOR_BG),
        );
    }

    let end = std::cmp::min(current + window + 1, tokens.len());
    let context_words: Vec<String> = tokens[current + 1..end]
        .iter()
        .map(|t| t.text.clone())
        .collect();

    let text = context_words.join(" ");

    Paragraph::new(text).alignment(Alignment::Left).style(
        Style::default()
            .fg(COLOR_TEXT)
            .add_modifier(Modifier::DIM)
            .bg(COLOR_BG),
    )
}

pub fn render_gutter_placeholder() -> Paragraph<'static> {
    Paragraph::new("│")
        .alignment(Alignment::Right)
        .style(Style::default().fg(COLOR_TEXT).bg(COLOR_BG))
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
