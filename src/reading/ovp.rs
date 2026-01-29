/// OVP (Optimal Viewing Position) Anchor Calculation
///
/// Per PRD Section 3.1:
/// Words are horizontally shifted so that the anchor letter remains at a fixed vertical coordinate.
/// The anchor position is calculated based on word length:
/// - 1 char word → position 0 (1st letter)
/// - 2-5 char words → position 1 (2nd letter)
/// - 6-9 char words → position 2 (3rd letter)
/// - 10-13 char words → position 3 (4th letter)
/// - 14+ char words → position 3 (capped at 4th for MVP)
///
/// Calculates the anchor position for a word based on its length.
///
/// Returns the 0-based index of the character that should be the anchor.
/// This allows the word to be positioned so the anchor letter appears
/// at a consistent vertical coordinate in the TUI.
pub fn calculate_anchor_position(word: &str) -> usize {
    let len = word.chars().count();
    match len {
        0..=1 => 0,
        2..=5 => 1,
        6..=9 => 2,
        _ => 3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_anchor_position_single_char() {
        let result = calculate_anchor_position("I");
        assert_eq!(result, 0, "Single char word should return position 0");
    }

    #[test]
    fn test_calculate_anchor_position_two_to_five_chars() {
        assert_eq!(calculate_anchor_position("He"), 1);
        assert_eq!(calculate_anchor_position("Hello"), 1);
        assert_eq!(calculate_anchor_position("world"), 1);
    }

    #[test]
    fn test_calculate_anchor_position_six_to_nine_chars() {
        assert_eq!(calculate_anchor_position("reading"), 2);
        assert_eq!(calculate_anchor_position("sentence"), 2);
        assert_eq!(calculate_anchor_position("anchored"), 2);
    }

    #[test]
    fn test_calculate_anchor_position_ten_to_thirteen_chars() {
        assert_eq!(calculate_anchor_position("extraordinary"), 3);
        assert_eq!(calculate_anchor_position("fascinating"), 3);
    }

    #[test]
    fn test_calculate_anchor_position_fourteen_plus_chars() {
        assert_eq!(calculate_anchor_position("extraordinarily"), 3);
        assert_eq!(calculate_anchor_position("Antidisestablishmentarianism"), 3);
    }

    #[test]
    fn test_calculate_anchor_position_empty_string() {
        let result = calculate_anchor_position("");
        assert_eq!(result, 0, "Empty string should return position 0");
    }

    #[test]
    fn test_calculate_anchor_position_two_char() {
        assert_eq!(calculate_anchor_position("am"), 1);
    }

    #[test]
    fn test_calculate_anchor_position_five_char() {
        assert_eq!(calculate_anchor_position("hello"), 1);
    }

    #[test]
    fn test_calculate_anchor_position_six_char() {
        assert_eq!(calculate_anchor_position("worlds"), 2);
    }

    #[test]
    fn test_calculate_anchor_position_nine_char() {
        assert_eq!(calculate_anchor_position("beautiful"), 2);
    }

    #[test]
    fn test_calculate_anchor_position_ten_char() {
        assert_eq!(calculate_anchor_position("government"), 3);
    }

    #[test]
    fn test_calculate_anchor_position_thirteen_char() {
        assert_eq!(calculate_anchor_position("characterize"), 3);
    }
}
