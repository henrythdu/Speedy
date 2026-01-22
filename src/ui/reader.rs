use ratatui::{widgets::Paragraph, Frame};

pub fn render_word(frame: &mut Frame, word: &str) {
    let area = frame.area();
    frame.render_widget(Paragraph::new(word), area);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_word_exists() {
        // Verify render_word function exists with correct signature
        let _ = render_word;
    }
}
