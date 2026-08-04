/// Token struct for RSVP reading
#[derive(Debug, PartialEq, Clone)]
pub struct Token {
    text: String,
    /// Trailing punctuation characters (e.g., ['?', '!'] for "word?!") per PRD Section 3.2 max stacking rule.
    punctuation: Vec<char>,
    /// Indicates if this token starts a new sentence (PRD Section 3.3).
    is_sentence_start: bool,
    /// Character count of the token text.
    char_count: usize,
    /// Index of this token within its sentence (0-based).
    sentence_index: usize,
    /// Total number of tokens in the sentence this token belongs to.
    sentence_length: usize,
}

impl Token {
    /// Creates a new Token with all fields.
    pub fn new(
        text: String,
        punctuation: Vec<char>,
        is_sentence_start: bool,
        char_count: usize,
        sentence_index: usize,
        sentence_length: usize,
    ) -> Self {
        Self {
            text,
            punctuation,
            is_sentence_start,
            char_count,
            sentence_index,
            sentence_length,
        }
    }

    /// Returns a reference to the token's text content.
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Returns a reference to the token's trailing punctuation characters.
    pub fn punctuation(&self) -> &[char] {
        &self.punctuation
    }

    /// Returns true if this token starts a new sentence.
    pub fn is_sentence_start(&self) -> bool {
        self.is_sentence_start
    }

    /// Returns the character count of the token text.
    pub fn char_count(&self) -> usize {
        self.char_count
    }

    /// Returns the index of this token within its sentence.
    pub fn sentence_index(&self) -> usize {
        self.sentence_index
    }

    /// Returns the total number of tokens in the sentence.
    pub fn sentence_length(&self) -> usize {
        self.sentence_length
    }
}
