use crate::app::mode::AppMode;
use crate::engine::Token;

/// Render state for UI components
pub struct RenderState {
    pub mode: AppMode,
    pub current_word: Option<String>,
    pub tokens: Vec<Token>,
    pub current_index: usize,
    pub context_left: Vec<String>,
    pub context_right: Vec<String>,
    pub progress: (usize, usize),
}

impl RenderState {
    /// Create an empty render state for when no document is loaded
    pub fn empty(mode: AppMode) -> Self {
        Self {
            mode,
            current_word: None,
            tokens: vec![],
            current_index: 0,
            context_left: vec![],
            context_right: vec![],
            progress: (0, 0),
        }
    }

    /// Create render state from reading state data
    pub fn from_reading_state(
        mode: AppMode,
        tokens: Vec<Token>,
        current_index: usize,
        context_window: usize,
    ) -> Self {
        // Calculate total before moving tokens
        let total = tokens.len();

        // Get context words before current
        let start = if current_index > context_window {
            current_index - context_window
        } else {
            0
        };
        let context_left: Vec<String> = tokens[start..current_index]
            .iter()
            .map(|t| t.text.clone())
            .collect();

        // Get context words after current
        let end = std::cmp::min(current_index + context_window + 1, tokens.len());
        let context_right: Vec<String> = tokens[current_index + 1..end]
            .iter()
            .map(|t| t.text.clone())
            .collect();

        Self {
            mode: mode.clone(),
            current_word: tokens.get(current_index).map(|t| t.text.clone()),
            tokens,
            current_index,
            context_left,
            context_right,
            progress: (current_index, total),
        }
    }
}
