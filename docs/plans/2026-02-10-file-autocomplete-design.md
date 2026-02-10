# Design Document: File Autocomplete Feature

**Date:** 2026-02-10  
**Status:** Review Complete - Ready for Implementation  
**Related:** Inspired by opencode's @ mention feature  
**Consensus Review:** Multi-model review completed (9/10 confidence)

---

## 1. Executive Summary

### Purpose
Implement a file picker autocomplete feature triggered by typing `@` in the command deck. This feature allows users to quickly discover and select supported files (PDF, EPUB) from the current directory and subdirectories without manually typing full paths.

### Scope
- **Trigger:** `@` symbol at the beginning of command input or after whitespace
- **File Discovery:** Recursive scan of current directory + subdirectories using **async threading**
- **Supported Formats:** PDF and EPUB files (matching existing `src/input/` support)
- **Max Depth:** 5 directory levels to prevent excessive scanning
- **Max Files:** 1000 files to prevent UI overflow

### Success Criteria
1. Typing `@` immediately shows a dropdown with matching files (or "Scanning..." indicator)
2. Typing after `@` filters the list in real-time
3. Keyboard navigation works intuitively (arrows, Tab, Enter, Esc)
4. Feature feels responsive - **no UI blocking during file discovery**
5. Works consistently across different directory structures

### Critical Design Changes (Post-Consensus)
- **CHANGED:** File discovery uses `std::thread` + `mpsc` channels (was synchronous)
- **CHANGED:** Per-directory cache keyed by path (was global 30s TTL)
- **ADDED:** "Scanning..." indicator during file discovery
- **ADDED:** Hidden directory exclusion, permission error handling

---

## 2. User Experience Design

### 2.1 Trigger Conditions

The autocomplete activates when:
- User types `@` at the start of the command buffer
- User types `@` after whitespace (e.g., "hello @")

The autocomplete deactivates when:
- User presses `Escape`
- User presses `Enter` to select a file
- User types a non-matching character that produces zero results
- User deletes the `@` symbol

### 2.2 Visual Layout

```
+------------------+
|  Speedy Reader   |
|                  |
|     [Word]       |  <- Reading zone (unchanged)
|                  |
+------------------+
| Scanning...      | <- Popup during discovery
|                  |    (or shows files once ready)
+------------------+
| @file█           |  <- Command deck with @ trigger
| COMMAND          |
+------------------+
```

**After discovery completes:**
```
+------------------+
|  Speedy Reader   |
|                  |
|     [Word]       |
|                  |
+------------------+
| FILES (12 matches)| <- Popup overlay (10 rows max)
| > file1.pdf      |    <- Selected item (anchor color)
|   file2.epub     |
|   docs/book.pdf  |
|   ...            |
| (+3 more)        |    <- Truncated indicator
+------------------+
| @file█           |
| COMMAND          |
+------------------+
```

### 2.3 Interaction Flow

```
[User types '@']
         |
         v
[Spawn background thread]
[Show "Scanning..." indicator]
         |
         v
[Thread scans directories]
[Streams results via mpsc]
         |
         v
[Update popup incrementally]
[User can type to filter during scan]
         |
         +-->[User types filter text]
         |           |
         |           v
         |   [Filter list in real-time]
         |   [Update popup display]
         |
         +-->[User presses Up/Down]
         |           |
         |           v
         |   [Change selected item]
         |   [Update highlight]
         |
         +-->[User presses Tab]
         |           |
         |           v
         |   [Insert selected file path]
         |   [Add trailing space]
         |   [Keep popup open for chaining]
         |
         +-->[User presses Enter]
         |           |
         |           v
         |   [Insert selected file path]
         |   [Close popup]
         |   [Execute command if ready]
         |
         +-->[User presses Escape]
         |           |
         |           v
         |   [Close popup]
         |   [Keep @ and any typed text]
         |
         +-->[User presses Ctrl+R]
                     |
                     v
             [Force cache refresh]
             [Re-scan directories]
```

### 2.4 Keyboard Shortcuts

| Key | Action |
|-----|--------|
| `@` | Activate autocomplete (if at start or after whitespace) |
| `Up` / `Down` | Navigate through file list (wraps around) |
| `Tab` | Insert selected file path + space, keep popup open |
| `Enter` | Insert selected file path, close popup |
| `Escape` | Close popup without selection |
| `Backspace` | Delete character, update filter (close if @ deleted) |
| `Ctrl+R` | Force cache refresh and re-scan |
| Any character | Add to filter, update matches |

### 2.5 Visual Styling

**Popup:**
- Position: Above command deck (5 lines above), or below if insufficient space
- Width: Command area width minus 2 cells (1-cell padding each side)
- Max height: 12 lines (1 header + 10 items + 1 optional scrollbar indicator)
- Background: Surface color from theme
- Border: Single-line border with text color

**Header:**
- Text: "FILES (N matches)" or "Scanning..."
- Color: Text color
- Style: Bold

**File Items:**
- Unselected: Text color with prefix ([PDF], [EPUB])
- Selected: Anchor color background, text color foreground
- Path display: Relative to current directory, truncated if too long

**Scroll/Truncation Indicator:**
- Show "(+N more)" or "..." if more than 10 files match
- Position: Footer line of popup

---

## 3. Technical Architecture

### 3.1 Module Structure

```
src/ui/
├── mod.rs              # Existing - export new modules
├── terminal.rs         # Modified - event handling, thread mgmt
├── command.rs          # Existing - command parsing (unchanged)
├── command_executor.rs # Existing - command execution (unchanged)
├── reader/
│   └── view.rs         # Modified - add render_autocomplete_popup()
└── autocomplete/       # NEW MODULE
    ├── mod.rs          # Public exports and constants
    ├── discovery.rs    # File scanning with threading
    ├── cache.rs        # Per-directory cache management
    ├── state.rs        # AutocompleteState struct and methods
    └── render.rs       # Popup rendering logic
```

### 3.2 Component Interactions

```
┌─────────────────┐
│   TuiManager    │ Owns AutocompleteState, spawns threads
│   (terminal.rs) │ Receives results via mpsc channel
└────────┬────────┘
         │
         │ 1. Detects @ trigger
         │ 2. Spawns discovery thread
         │ 3. Updates state.active = true
         │ 4. Handles keyboard navigation
         v
┌─────────────────┐
│  Autocomplete   │
│     State       │ Manages selection, filtering, scroll
│   (state.rs)    │ Receives files from channel
└────────┬────────┘
         │
         │ Queries filtered files
         v
┌─────────────────┐
│  File Discovery │ Runs in separate thread
│  (discovery.rs) │ Streams results via mpsc
└────────┬────────┘
         │
         │ Uses
         v
┌─────────────────┐
│  Per-Dir Cache  │ Cache keyed by directory path
│   (cache.rs)    │ path -> (files, timestamp)
└─────────────────┘
         │
         v
┌─────────────────┐
│    Renderer     │
│   (render.rs)   │ Renders popup via ratatui
└─────────────────┘
```

### 3.3 Integration Points

**1. TuiManager (`terminal.rs`)**
- Add field: `autocomplete_state: AutocompleteState`
- Add field: `discovery_receiver: Option<Receiver<PathBuf>>`
- Modify event loop: Handle @ detection and navigation keys
- Check receiver for new files each frame
- Pass state to render_frame for popup rendering

**2. Event Loop Changes**
```rust
// In run_event_loop()
match key.code {
    KeyCode::Char('@') => {
        if should_trigger_autocomplete(&command_buffer) {
            autocomplete_state.activate(&command_buffer, cursor_pos);
            // Spawn discovery thread
            spawn_discovery_thread(&autocomplete_state);
        }
    }
    KeyCode::Char(c) if autocomplete_state.active => {
        autocomplete_state.handle_input(c);
    }
    KeyCode::Up if autocomplete_state.active => {
        autocomplete_state.select_previous();
    }
    // ... other navigation keys
    KeyCode::Char('r') if key.modifiers.contains(KeyModifiers::CONTROL) => {
        autocomplete_state.force_refresh();
    }
}

// Each frame, check for new files from discovery thread
if let Some(ref receiver) = discovery_receiver {
    while let Ok(file) = receiver.try_recv() {
        autocomplete_state.add_file(file);
    }
}
```

**3. View Rendering (`view.rs`)**
- Add function: `render_autocomplete_popup(frame, area, state)`
- Call from `render_command_deck` when state.active is true
- Handle "Scanning..." vs file list display

---

## 4. Implementation Details

### 4.1 Data Structures

**AutocompleteState (src/ui/autocomplete/state.rs)**
```rust
pub struct AutocompleteState {
    /// Whether the popup is currently visible
    pub active: bool,
    
    /// Text after @ used for filtering (e.g., "file" from "@file")
    pub query: String,
    
    /// Position of @ in command_buffer (for replacement)
    pub anchor_idx: usize,
    
    /// All discovered files (incrementally populated)
    pub files: Vec<PathBuf>,
    
    /// Indices into files that match current query
    pub filtered_indices: Vec<usize>,
    
    /// Currently selected item index (into filtered_indices)
    pub selected_idx: usize,
    
    /// Scroll offset for viewing items beyond viewport
    pub scroll_offset: usize,
    
    /// Whether discovery is currently running
    pub is_scanning: bool,
    
    /// Root directory being scanned
    pub scan_root: PathBuf,
}

impl AutocompleteState {
    /// Activate autocomplete when @ is typed
    pub fn activate(&mut self, command_buffer: &str, cursor_pos: usize, root: &Path);
    
    /// Add a file received from discovery thread
    pub fn add_file(&mut self, file: PathBuf);
    
    /// Handle character input while active
    pub fn handle_input(&mut self, c: char);
    
    /// Navigate up in list
    pub fn select_previous(&mut self);
    
    /// Navigate down in list  
    pub fn select_next(&mut self);
    
    /// Get currently selected file path
    pub fn get_selected(&self) -> Option<&PathBuf>;
    
    /// Insert selected file into command buffer
    pub fn apply_selection(&self, command_buffer: &mut String) -> String;
    
    /// Close popup
    pub fn deactivate(&mut self);
    
    /// Force cache refresh and re-scan
    pub fn force_refresh(&mut self);
    
    /// Check if character should trigger activation
    pub fn should_activate(command_buffer: &str, cursor_pos: usize) -> bool;
}
```

**PerDirectoryCache (src/ui/autocomplete/cache.rs)**
```rust
pub struct PerDirectoryCache {
    /// Cache entries: directory path -> (files, timestamp)
    entries: HashMap<PathBuf, CacheEntry>,
    
    /// Cache validity duration
    ttl: Duration,
}

struct CacheEntry {
    files: Vec<PathBuf>,
    timestamp: Instant,
}

impl PerDirectoryCache {
    /// Get cached files for directory (returns None if expired/missing)
    pub fn get(&self, dir: &Path) -> Option<&[PathBuf]>;
    
    /// Store files for directory
    pub fn put(&mut self, dir: PathBuf, files: Vec<PathBuf>);
    
    /// Invalidate entry for directory
    pub fn invalidate(&mut self, dir: &Path);
    
    /// Check if entry is expired
    fn is_expired(&self, entry: &CacheEntry) -> bool;
}
```

### 4.2 File Discovery (Threaded)

**Discovery Thread (src/ui/autocomplete/discovery.rs)**
```rust
pub fn spawn_discovery_thread(
    root: PathBuf,
    sender: Sender<PathBuf>,
    cache: Arc<Mutex<PerDirectoryCache>>
) -> JoinHandle<()> {
    std::thread::spawn(move || {
        // Check cache first
        if let Some(cached) = cache.lock().unwrap().get(&root) {
            // Send cached files
            for file in cached {
                let _ = sender.send(file.clone());
            }
            return;
        }
        
        // Scan directories
        let files = scan_directories(&root);
        
        // Update cache
        cache.lock().unwrap().put(root, files.clone());
        
        // Send files
        for file in files {
            let _ = sender.send(file);
        }
    })
}

fn scan_directories(root: &Path) -> Vec<PathBuf> {
    const MAX_DEPTH: usize = 5;
    const MAX_FILES: usize = 1000;
    const SUPPORTED_EXTENSIONS: &[&str] = &["pdf", "epub"];
    
    let mut files = Vec::new();
    let mut stack = vec![(root.to_path_buf(), 0)];
    
    while let Some((dir, depth)) = stack.pop() {
        if depth > MAX_DEPTH || files.len() >= MAX_FILES {
            continue;
        }
        
        let entries = match fs::read_dir(&dir) {
            Ok(entries) => entries,
            Err(_) => continue, // Skip directories we can't read
        };
        
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            let file_name = path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("");
            
            // Skip hidden directories and files
            if file_name.starts_with('.') {
                continue;
            }
            
            if path.is_dir() {
                stack.push((path, depth + 1));
            } else if is_supported_file(&path, SUPPORTED_EXTENSIONS) {
                files.push(path);
            }
        }
    }
    
    // Sort for consistent ordering
    files.sort();
    files
}
```

### 4.3 Filtering Logic

```rust
fn filter_files(files: &[PathBuf], query: &str) -> Vec<usize> {
    let query_lower = query.to_lowercase();
    
    files
        .iter()
        .enumerate()
        .filter(|(_, path)| {
            let filename = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_lowercase();
            
            let path_str = path
                .to_string_lossy()
                .to_lowercase();
            
            // Match filename or full path
            filename.contains(&query_lower) || 
            path_str.contains(&query_lower)
        })
        .map(|(idx, _)| idx)
        .collect()
}
```

### 4.4 Keyboard Handling Matrix

| State | Key | Action |
|-------|-----|--------|
| Inactive | `@` at start/after space | Activate, spawn discovery thread, show "Scanning..." |
| Active | Character | Append to query, refilter |
| Active | Backspace | Delete last char, refilter (deactivate if @ deleted) |
| Active | Up | selected_idx = (selected_idx - 1) % count |
| Active | Down | selected_idx = (selected_idx + 1) % count |
| Active | Tab | Apply selection + space, keep active (for chaining) |
| Active | Enter | Apply selection, deactivate |
| Active | Escape | Deactivate, keep @query text |
| Active | Ctrl+R | Force cache refresh, re-scan |
| Active | Any other | Deactivate, process key normally |

### 4.5 Rendering Calculations

**Popup Position (Dynamic)**
```rust
fn calculate_popup_position(command_area: Rect, terminal_height: u16) -> Rect {
    const MAX_HEIGHT: u16 = 12;
    const MIN_HEIGHT: u16 = 3;
    
    let popup_height = min(MAX_HEIGHT, filtered_count as u16 + 2);
    let popup_height = max(MIN_HEIGHT, popup_height);
    let popup_width = command_area.width.saturating_sub(2);
    
    // Check if there's room above command deck
    let space_above = command_area.y;
    let space_below = terminal_height - command_area.y - command_area.height;
    
    if space_above >= popup_height {
        // Position above
        let y = command_area.y.saturating_sub(popup_height);
        Rect::new(command_area.x + 1, y, popup_width, popup_height)
    } else if space_below >= popup_height {
        // Position below
        let y = command_area.y + command_area.height;
        Rect::new(command_area.x + 1, y, popup_width, popup_height)
    } else {
        // Default to above, clamp height
        let y = command_area.y.saturating_sub(min(popup_height, space_above));
        Rect::new(command_area.x + 1, y, popup_width, min(popup_height, space_above))
    }
}
```

**Scroll Logic**
```rust
fn calculate_visible_range(
    selected: usize, 
    total: usize, 
    viewport_height: usize
) -> (usize, usize) {
    let half_viewport = viewport_height / 2;
    
    let start = if selected < half_viewport {
        0
    } else if selected > total.saturating_sub(half_viewport) {
        total.saturating_sub(viewport_height)
    } else {
        selected.saturating_sub(half_viewport)
    };
    
    let end = min(start + viewport_height, total);
    (start, end)
}
```

---

## 5. Testing Strategy

### 5.1 Unit Tests

**discovery.rs**
- Test file scanning with mock directory structure
- Test max depth limiting
- Test max files limiting
- Test supported extensions filtering
- Test hidden directory exclusion
- Test permission error handling
- Test non-UTF8 filename handling

**cache.rs**
- Test cache hit/miss
- Test TTL expiration
- Test per-directory isolation
- Test cache invalidation

**state.rs**
- Test activation/deactivation
- Test filtering with various queries
- Test navigation wrapping
- Test selection application
- Test incremental file addition

### 5.2 Integration Tests

**terminal.rs changes**
- Test @ trigger detection
- Test keyboard navigation in event loop
- Test interaction with command buffer
- Test thread spawning and channel communication

**view.rs changes**
- Test popup renders in correct position
- Test "Scanning..." indicator display
- Test scroll behavior
- Test styling (selected vs unselected)

### 5.3 Manual Testing Scenarios

1. **Basic Flow**
   - Type `@` in empty buffer -> see "Scanning..." then file list
   - Type filter -> list narrows in real-time
   - Press Down -> selection moves
   - Press Enter -> file inserted

2. **Threading**
   - Type `@` in large directory -> UI remains responsive
   - Type filter while scanning -> filtering works
   - Verify no UI freezing during discovery

3. **Navigation**
   - Test Up/Down wrapping (first to last, last to first)
   - Test Tab inserts file + space
   - Test Escape closes without insertion
   - Test Ctrl+R forces refresh

4. **Edge Cases**
   - Empty directory -> show "No files found"
   - Very long filenames -> truncate with ellipsis
   - Many files (>10) -> verify scroll indicator
   - Special characters in paths -> handle correctly
   - Permission denied directories -> skip gracefully
   - Hidden directories -> excluded from scan

5. **Cache Behavior**
   - Type `@` twice in same directory -> second time uses cache (instant)
   - Change directory -> cache miss, new scan
   - Press Ctrl+R -> forces cache refresh

6. **Performance**
   - Large directory tree (1000+ files)
   - Measure time from @ to first file display
   - Ensure UI responsiveness during scan
   - Test on network mount (slow FS)

---

## 6. Open Questions & Decisions

### 6.1 Decisions Made (Post-Consensus)

1. **File Discovery Method**
   - **DECISION:** Use `std::thread::spawn` + `mpsc` channels
   - **Rationale:** Prevents UI blocking, allows incremental display
   - **Alternative Rejected:** Synchronous scanning (causes UI lag)

2. **Cache Strategy**
   - **DECISION:** Per-directory cache keyed by path
   - **Rationale:** Prevents stale data when changing directories
   - **Alternative Rejected:** Global 30s TTL (shows wrong files after cd)

3. **State Management**
   - **DECISION:** Keep AutocompleteState in TuiManager
   - **Rationale:** Simpler implementation, consistent with existing patterns
   - **Note:** Can refactor to dedicated controller later if needed

4. **Hidden Directories**
   - **DECISION:** Exclude hidden directories (starting with `.`)
   - **Rationale:** Avoids scanning .git, node_modules, etc.
   - **Future:** Could make this configurable

### 6.2 Future Enhancements

- **Fuzzy Matching:** Support fuzzy search (e.g., "fb" matches "foo bar.pdf")
- **Recent Files:** Show recently opened files at top of list
- **Bookmarks:** Allow @bookmark syntax for frequently accessed paths
- **Directory Navigation:** Support @../ to autocomplete parent directories
- **File Preview:** Show file size/metadata in popup
- **Configuration:** Make depth limit, file extensions, hidden dir exclusion configurable

---

## 7. Dependencies

**New Dependencies Required:**
- None - using only std library for threading and channels

**Existing Dependencies Used:**
- `std::fs` - File system operations
- `std::path` - Path manipulation
- `std::time` - Cache timing
- `std::thread` - Background discovery
- `std::sync::mpsc` - Thread communication
- `ratatui` - Popup rendering
- `crossterm` - Keyboard events

---

## 8. Risk Assessment

| Risk | Severity | Likelihood | Mitigation |
|------|----------|------------|------------|
| **Threading complexity** | Medium | Medium | Well-scoped thread with clear lifetime, mpsc channel |
| **UI blocking during scan** | Low | Low | Async discovery prevents blocking, even on slow FS |
| **Cache stale data** | Low | Low | Per-directory cache, Ctrl+R for manual refresh |
| **Special character issues** | Low | Low | Use PathBuf/String for all path handling |
| **Terminal size too small** | Low | Medium | Dynamic placement (above/below), clamps to min size |
| **Thread panics** | Low | Low | Use Result types, proper error handling in thread |

---

## 9. Implementation Checklist

### Phase 1: Core Infrastructure
- [ ] Create `src/ui/autocomplete/mod.rs` with module exports
- [ ] Create `src/ui/autocomplete/cache.rs` with per-directory cache
- [ ] Create `src/ui/autocomplete/discovery.rs` with threaded scanning
- [ ] Create `src/ui/autocomplete/state.rs` with state management
- [ ] Add unit tests for cache module
- [ ] Add unit tests for discovery module

### Phase 2: Integration
- [ ] Add `AutocompleteState` to `TuiManager`
- [ ] Add `discovery_receiver` channel to `TuiManager`
- [ ] Modify event loop in `terminal.rs` for @ detection and threading
- [ ] Add Ctrl+R handler for cache refresh
- [ ] Add integration tests for keyboard handling

### Phase 3: UI
- [ ] Create `src/ui/autocomplete/render.rs` with popup rendering
- [ ] Modify `view.rs` to call render function
- [ ] Add "Scanning..." indicator
- [ ] Add styling (colors, borders, selection highlight)
- [ ] Test visual appearance in terminal

### Phase 4: Polish
- [ ] Add error handling (permissions, non-UTF8, etc.)
- [ ] Optimize performance if needed
- [ ] Add help text for the feature
- [ ] Final integration testing

---

## 10. Appendix

### A. Similar Implementations for Reference

**opencode @ mention feature:**
- Triggers on `@` character
- Shows dropdown with context-appropriate options
- Real-time filtering as you type
- Keyboard navigation with visual feedback

**VS Code file picker:**
- Quick file navigation with fuzzy matching
- Shows file icons and paths
- Recent files prioritized

**fzf:**
- Async file discovery
- Streaming results
- Highly responsive even on large directories

### B. Code Examples

**File Type Prefix:**
```rust
fn get_file_prefix(path: &Path) -> &'static str {
    match path.extension().and_then(|e| e.to_str()) {
        Some("pdf") => "[PDF]",
        Some("epub") => "[EPUB]",
        _ => "[FILE]",
    }
}
```

**Path Truncation:**
```rust
fn format_path_for_display(path: &Path, max_width: usize) -> String {
    let path_str = path.to_string_lossy();
    if path_str.len() <= max_width {
        path_str.to_string()
    } else {
        // Truncate from start, keep end
        let start_idx = path_str.len().saturating_sub(max_width - 3);
        format!("...{}", &path_str[start_idx..])
    }
}
```

**Thread Safety:**
```rust
// Wrap cache in Arc<Mutex<>> for thread-safe sharing
let cache = Arc::new(Mutex::new(PerDirectoryCache::new()));
let cache_clone = Arc::clone(&cache);

std::thread::spawn(move || {
    // Use cache_clone in thread
    let mut cache = cache_clone.lock().unwrap();
    // ...
});
```

---

**Document Status:** Review Complete - Consensus Achieved (9/10 confidence)  
**Next Step:** Implementation ready - proceed with Phase 1
