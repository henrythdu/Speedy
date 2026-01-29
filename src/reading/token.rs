/// Token struct for RSVP reading
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Token {
    pub text: String,
    /// Trailing punctuation characters (e.g., ['?', '!'] for "word?!") per PRD Section 3.2 max stacking rule.
    pub punctuation: Vec<char>,
    /// Indicates if this token starts a new sentence (PRD Section 3.3).
    pub is_sentence_start: bool,
}
