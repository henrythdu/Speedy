// UI reader component - word rendering with ratatui

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_word_centered() {
        // For MVP, verify word is included in output
        let word = "hello";
        let output = render_word(word);
        assert!(output.contains(word));
    }
}
