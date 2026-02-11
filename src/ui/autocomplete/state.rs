//! Autocomplete state management
//!
//! Manages the state of the autocomplete popup including discovered files,
//! current selection, filtering, and scroll position.

use std::path::{Path, PathBuf};

use super::is_supported_file;

/// State for the autocomplete popup
#[derive(Debug)]
pub struct AutocompleteState {
    /// Whether the popup is currently visible
    active: bool,

    /// Text after @ used for filtering (e.g., "file" from "@file")
    query: String,

    /// Position of @ in command_buffer (for replacement)
    anchor_idx: usize,

    /// All discovered files (incrementally populated)
    files: Vec<PathBuf>,

    /// Indices into files that match current query
    filtered_indices: Vec<usize>,

    /// Currently selected item index (into filtered_indices)
    selected_idx: usize,

    /// Scroll offset for viewing items beyond viewport
    scroll_offset: usize,

    /// Whether discovery is currently running
    is_scanning: bool,

    /// Root directory being scanned
    scan_root: PathBuf,
}

impl AutocompleteState {
    /// Create a new inactive autocomplete state
    pub fn new() -> Self {
        Self {
            active: false,
            query: String::new(),
            anchor_idx: 0,
            files: Vec::new(),
            filtered_indices: Vec::new(),
            selected_idx: 0,
            scroll_offset: 0,
            is_scanning: false,
            scan_root: PathBuf::new(),
        }
    }

    /// Check if the popup is currently visible
    pub fn is_active(&self) -> bool {
        self.active
    }

    /// Get the current filter query
    pub fn query(&self) -> &str {
        &self.query
    }

    /// Get the anchor index for replacement
    #[allow(dead_code)]
    pub fn anchor_idx(&self) -> usize {
        self.anchor_idx
    }

    /// Get the list of discovered files
    pub fn files(&self) -> &Vec<PathBuf> {
        &self.files
    }

    /// Get the filtered indices
    #[allow(dead_code)]
    pub fn filtered_indices(&self) -> &Vec<usize> {
        &self.filtered_indices
    }

    /// Get the currently selected index
    pub fn selected_idx(&self) -> usize {
        self.selected_idx
    }

    /// Get the scroll offset
    pub fn scroll_offset(&self) -> usize {
        self.scroll_offset
    }

    /// Check if discovery is currently running
    pub fn is_scanning(&self) -> bool {
        self.is_scanning
    }

    /// Get the root directory being scanned
    #[allow(dead_code)]
    pub fn scan_root(&self) -> &PathBuf {
        &self.scan_root
    }

    /// Activate autocomplete when @ is typed
    pub fn activate(&mut self, _command_buffer: &str, cursor_pos: usize, root: &Path) {
        self.active = true;
        self.anchor_idx = cursor_pos;
        self.query = String::new();
        self.files.clear();
        self.filtered_indices.clear();
        self.selected_idx = 0;
        self.scroll_offset = 0;
        self.is_scanning = true;
        self.scan_root = root.to_path_buf();
    }

    /// Add a file received from discovery thread
    pub fn add_file(&mut self, file: PathBuf) {
        // Only add supported file types
        if !is_supported_file(&file) {
            return;
        }

        // Check if file matches current query
        if self.matches_query(&file) {
            self.filtered_indices.push(self.files.len());
        }
        self.files.push(file);
    }

    /// Handle character input while active
    pub fn handle_input(&mut self, c: char) {
        self.query.push(c);
        self.refilter();
        self.selected_idx = 0;
        self.scroll_offset = 0;
    }

    /// Remove last character from query (backspace)
    pub fn backspace(&mut self) {
        self.query.pop();
        self.refilter();
        self.selected_idx = 0;
        self.scroll_offset = 0;
    }

    /// Navigate up in list (wraps around)
    pub fn select_previous(&mut self) {
        if self.filtered_indices.is_empty() {
            return;
        }
        if self.selected_idx == 0 {
            self.selected_idx = self.filtered_indices.len() - 1;
        } else {
            self.selected_idx -= 1;
        }
        self.update_scroll();
    }

    /// Navigate down in list (wraps around)
    pub fn select_next(&mut self) {
        if self.filtered_indices.is_empty() {
            return;
        }
        self.selected_idx = (self.selected_idx + 1) % self.filtered_indices.len();
        self.update_scroll();
    }

    /// Get currently selected file path
    pub fn get_selected(&self) -> Option<&PathBuf> {
        self.filtered_indices
            .get(self.selected_idx)
            .and_then(|&idx| self.files.get(idx))
    }

    /// Insert selected file into command buffer
    /// Returns the new command buffer content
    pub fn apply_selection(&self, command_buffer: &mut String) -> String {
        if let Some(file) = self.get_selected() {
            // Replace @query with @filepath (keep the @ prefix)
            let file_str = file.to_string_lossy();
            command_buffer.replace_range(self.anchor_idx.., &format!("@{}", file_str));
        }
        command_buffer.clone()
    }

    /// Close popup without changing command buffer
    pub fn deactivate(&mut self) {
        self.active = false;
        self.is_scanning = false;
    }

    /// Mark discovery as complete
    pub fn mark_scanning_complete(&mut self) {
        self.is_scanning = false;
    }

    /// Clear files for cache refresh
    pub fn clear_files(&mut self) {
        self.files.clear();
        self.filtered_indices.clear();
    }

    /// Check if a character should trigger autocomplete activation
    pub fn should_activate(command_buffer: &str, cursor_pos: usize) -> bool {
        if cursor_pos == 0 {
            // At start of buffer
            return true;
        }

        // Check if @ is typed after whitespace
        if let Some(prev_char) = command_buffer.chars().nth(cursor_pos - 1) {
            return prev_char.is_whitespace();
        }

        false
    }

    /// Get the number of filtered matches
    pub fn match_count(&self) -> usize {
        self.filtered_indices.len()
    }

    /// Get the file at a filtered index
    pub fn get_filtered_file(&self, filtered_idx: usize) -> Option<&PathBuf> {
        self.filtered_indices
            .get(filtered_idx)
            .and_then(|&idx| self.files.get(idx))
    }

    /// Check if a file matches the current query
    fn matches_query(&self, file: &Path) -> bool {
        if self.query.is_empty() {
            return true;
        }

        let query_lower = self.query.to_lowercase();
        let filename = file
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_lowercase();

        let path_str = file.to_string_lossy().to_lowercase();

        filename.contains(&query_lower) || path_str.contains(&query_lower)
    }

    /// Refilter files based on current query
    fn refilter(&mut self) {
        self.filtered_indices.clear();
        for (idx, file) in self.files.iter().enumerate() {
            if self.matches_query(file) {
                self.filtered_indices.push(idx);
            }
        }
    }

    /// Update scroll offset to keep selected item visible
    fn update_scroll(&mut self) {
        const VISIBLE_ITEMS: usize = 10;

        if self.selected_idx < self.scroll_offset {
            self.scroll_offset = self.selected_idx;
        } else if self.selected_idx >= self.scroll_offset + VISIBLE_ITEMS {
            self.scroll_offset = self.selected_idx.saturating_sub(VISIBLE_ITEMS - 1);
        }
    }
}

impl Default for AutocompleteState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_activate_at_start() {
        assert!(AutocompleteState::should_activate("", 0));
    }

    #[test]
    fn test_should_activate_after_whitespace() {
        assert!(AutocompleteState::should_activate("hello ", 6));
        assert!(AutocompleteState::should_activate("cmd  ", 4));
    }

    #[test]
    fn test_should_not_activate_after_char() {
        assert!(!AutocompleteState::should_activate("hello", 5));
        assert!(!AutocompleteState::should_activate("cmd@", 4));
    }

    #[test]
    fn test_state_activation() {
        let mut state = AutocompleteState::new();
        state.activate("", 0, Path::new("/test"));

        assert!(state.active);
        assert!(state.is_scanning);
        assert_eq!(state.anchor_idx, 0);
    }

    #[test]
    fn test_navigation_wrapping() {
        let mut state = AutocompleteState::new();
        state.activate("", 0, Path::new("/test"));
        state.files = vec![PathBuf::from("a"), PathBuf::from("b"), PathBuf::from("c")];
        state.filtered_indices = vec![0, 1, 2];

        assert_eq!(state.selected_idx, 0);

        state.select_previous(); // Should wrap to last
        assert_eq!(state.selected_idx, 2);

        state.select_next(); // Should wrap to first
        assert_eq!(state.selected_idx, 0);
    }

    #[test]
    fn test_query_filtering() {
        let mut state = AutocompleteState::new();
        state.activate("", 0, Path::new("/test"));

        // Use unique directories to avoid path matching issues
        state.files = vec![
            PathBuf::from("/docs/document.pdf"),
            PathBuf::from("/books/book.epub"),
            PathBuf::from("/text/readme.txt"),
        ];
        state.filtered_indices = vec![0, 1, 2];

        // Filter by "doc" - should only match document.pdf
        state.handle_input('d');
        state.handle_input('o');
        state.handle_input('c');
        assert_eq!(state.match_count(), 1);
        assert_eq!(
            state.get_filtered_file(0).unwrap().file_name().unwrap(),
            "document.pdf"
        );
    }
}
