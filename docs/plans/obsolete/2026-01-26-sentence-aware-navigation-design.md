# Sentence-Aware Navigation Design

**Status:** Design Complete ✓  
**Date:** 2026-01-26  
**Scope:** MVP - Basic sentence detection and navigation (defer abbreviations/decimals to v1.1)

---

## Overview

Implement sentence-aware navigation (j/k keys) that always land at sentence beginnings to prevent users from starting mid-sentence. This is a core reading experience feature (PRD Section 3.3) that builds directly on the existing timing engine.

---

## Architecture

### Module Responsibilities

#### timing.rs - Tokenization & Detection
**Add to Token struct:**
```rust
pub struct Token {
    pub text: String,
    pub punctuation: Vec<char>,
    pub is_sentence_start: bool,  // NEW FIELD
}
```

**Add pure detection function:**
```rust
pub fn detect_sentence_boundary(prev_token: Option<&Token>, current_word: &str) -> bool
```
- Returns `true` if:
  1. Previous token had `.`, `?`, `!`, OR `\n`
  2. Current word starts with capital letter (A-Z)
- First token of text always returns `true` (PRD requirement)

**Modify tokenize_text():**
- Iterate tokens with 1-token lookbehind
- Call `detect_sentence_boundary()` for each token
- Set `is_sentence_start` flag accordingly

#### state.rs - Navigation Logic
**Add to ReadingState:**
```rust
impl ReadingState {
    fn find_next_sentence_start(&self) -> Option<usize>
    fn find_previous_sentence_start(&self) -> Option<usize>
    pub fn jump_to_next_sentence(&mut self) -> bool
    pub fn jump_to_previous_sentence(&mut self) -> bool
}
```

**Edge cases:**
- At first sentence start → `find_previous()` returns 0 (stay put)
- At last sentence start → `find_next()` returns `len - 1` (stay put)
- Empty token stream → Both methods return None or no-op
- Current token is sentence start → Navigate to NEXT/PREVIOUS sentence start

#### app.rs - Future Wiring (separate bead)
- Bind `j` key to `reading_state.jump_to_next_sentence()`
- Bind `k` key to `reading_state.jump_to_previous_sentence()`
- Update UI state on navigation events
- (Phase 2) Render micro-progress line with `▁` character

### Data Flow
```
Text Input → tokenize_text() → Tokens with is_sentence_start flags
                                           ↓
                                   ReadingState manages current_index
                                           ↓
                      Navigation (j/k) → Find sentence start → Update current_index
```

---

## Error Handling

**Philosophy: Pure functions, no panics**

- Return `Option<usize>` for navigation results (None if no sentence start found)
- Use explicit `if let` instead of `expect()` or `unwrap()`
- Handle edge cases gracefully (empty stream, single token, at boundaries)

**No external failures:**
- Sentence detection has no I/O or external dependencies
- Navigation operates on in-memory token indices
- Only potential failure: invalid index access (prevent with bounds checking)

---

## Testing Strategy (TDD)

### Speedy-hmc (Detection Algorithm) Tests
- `test_sentence_boundary_basic_period()` - "Hello. World" → second token flagged
- `test_sentence_boundary_question_mark()` - "What? Yes" → second token flagged
- `test_sentence_boundary_exclamation()` - "Stop! Go" → second token flagged
- `test_sentence_boundary_newline()` - "Hello\nWorld" → both tokens flagged
- `test_first_token_always_start()` - Single token gets `is_sentence_start = true`
- `test_consecutive_newlines()` - "Hello\n\nWorld" → all three flagged
- `test_capital_required()` - "hello. world" → NO flag (lowercase after period)
- `test_no_punctuation()` - "hello world" → only first token flagged

### Speedy-ui3 (Tokenization) Tests
- All existing 49 timing tests must pass (regression check)
- `test_is_sentence_start_set_correctly()` - Verify flags match detection logic
- `test_tokenize_single_sentence()` - Multi-word sentence
- `test_tokenize_multiple_sentences()` - Multiple sentences in text

### Speedy-58j (Forward Navigation) Tests
- `test_jump_to_next_middle()` - Jump from middle to next start
- `test_jump_to_next_at_boundary()` - Jump from sentence start to next
- `test_jump_to_next_last_sentence()` - At end, stays put
- `test_jump_to_next_empty_stream()` - Empty tokens, no-op
- `test_find_next_no_more_sentences()` - No next sentence found

### Speedy-f1q (Backward Navigation) Tests
- `test_jump_to_prev_middle()` - Jump from middle to previous start
- `test_jump_to_prev_at_boundary()` - Jump from sentence start to previous
- `test_jump_to_prev_first_sentence()` - At beginning, stays put
- `test_jump_to_prev_empty_stream()` - Empty tokens, no-op
- `test_find_prev_no_more_sentences()` - No previous sentence found

---

## Implementation Plan

### 1. Speedy-hmc (Sentence Detection Algorithm)
**Priority: P2**  
**Estimated: 2-3 hours**

**Tasks:**
- Add `detect_sentence_boundary(prev_token: Option<&Token>, current_word: &str) -> bool` to timing.rs
- Write TDD tests for basic detection, newline boundaries, consecutive newlines, capital detection
- Implement `detect_sentence_boundary()` to make tests pass
- Run `cargo test` to verify

**Acceptance Criteria:**
- All detection tests pass
- Function is pure (no side effects)
- Matches PRD Section 3.3 MVP rules

### 2. Speedy-ui3 (Tokenization Update)
**Priority: P2**  
**Estimated: 1-2 hours**

**Tasks:**
- Add `is_sentence_start: bool` field to Token struct
- Modify `tokenize_text()` to iterate tokens with lookbehind context
- Call `detect_sentence_boundary()` for each token
- Set `true` for first token (PRD requirement)
- Write TDD tests for correct flag setting
- Ensure all existing 49 timing tests pass

**Acceptance Criteria:**
- First token always marked as sentence start
- Flags set correctly for all boundary conditions
- All 49 existing timing tests still pass
- Tokenization is still deterministic

### 3. Speedy-58j (Forward Navigation)
**Priority: P2**  
**Estimated: 1-2 hours**

**Tasks:**
- Add `find_next_sentence_start()` to ReadingState (scans forward)
- Add `jump_to_next_sentence()` method
- Write TDD tests for forward navigation edge cases
- Run `cargo test` to verify

**Acceptance Criteria:**
- Jumps to next sentence start after current position
- Stays at end if at last sentence start
- Handles empty stream gracefully
- Navigates to NEXT sentence if current token is already a sentence start

### 4. Speedy-f1q (Backward Navigation)
**Priority: P2**  
**Estimated: 1-2 hours**

**Tasks:**
- Add `find_previous_sentence_start()` to ReadingState (scans backward)
- Add `jump_to_previous_sentence()` method
- Write TDD tests for backward navigation edge cases
- Run `cargo test` to verify

**Acceptance Criteria:**
- Jumps to previous sentence start before current position
- Stays at beginning if at first sentence start
- Handles empty stream gracefully
- Navigates to PREVIOUS sentence if current token is already a sentence start

---

## Future Work (Phase 2)

### Not in Scope for MVP
- Abbreviation detection (Dr., Mr., Mrs., e.g., i.e., etc.) - deferred to v1.1
- Decimal number handling (3.14, 2.5, 1,000) - deferred to v1.1
- UI event binding (j/k keys) - separate bead after engine API is stable
- Micro-progress rendering (`▁` character) - part of gutter implementation
- Sentence progress calculation (position within current sentence) - gutter feature

### Future Engine API Extension
When implementing gutter (Phase 2), may need to add:
```rust
ReadingState::get_sentence_progress() -> (current_in_sentence: usize, sentence_length: usize)
```
This can be added later without breaking navigation features.

---

## PRD Alignment

✅ **PRD Section 3.3: Sentence-Aware Navigation**
- ✅ Navigation jumps (j/k) always land at sentence beginnings
- ✅ Terminal punctuation: ., ?, ! indicate sentence ends
- ✅ Newlines indicate sentence boundaries
- ✅ Forward (j): Find next sentence start after current position
- ✅ Backward (k): Find nearest sentence start before current position

⏸️ **Explicitly Deferred (v1.1):**
- Abbreviations: Dr., Mr., Mrs., Ms., St., Jr., e.g., i.e., vs., etc.
- Decimal numbers: 3.14, 2.5, 1,000

---

## Design Decisions

### Why is_sentence_start: bool field?
**Chosen over alternatives:**
- ❌ Enum markers (Token::SentenceStart) - Too disruptive, requires updating all match arms
- ❌ Stateless calculation - Pushes complexity to every consumer
- ❌ sentence_id field - Overkill for MVP, requires post-processing

**Reasons for bool field:**
- Follows spaCy NLP pattern (industry standard)
- Minimal disruption to existing code
- Opt-in: consumers can ignore new field
- Scalable: future metadata (is_paragraph_start) easy to add
- Simple implementation: tokenizer needs 1-token lookbehind

### Why pure detection function?
**Benefits:**
- Easy to test in isolation
- No side effects or mutation
- Reusable across contexts
- Clear contract with well-defined inputs/outputs

### Why defer abbreviations/decimals?
**MVP Principle:**
- Basic rule delivers 90% value with 20% effort
- Complex edge cases (Dr., 3.14) require testing with real text samples
- Better to ship functional core, then iterate based on user feedback

---

## Risks & Mitigations

### Risk 1: Capital letter detection may be imperfect
**Mitigation:**
- MVP uses simple A-Z check
- Test with mixed-case text
- Document limitation in PRD

### Risk 2: Performance O(n) scanning for navigation
**Mitigation:**
- For MVP, linear scan is acceptable (typical texts < 100k tokens)
- Future optimization: Pre-compute sentence start indices array O(1)
- Cache sentence starts if navigation becomes performance bottleneck

### Risk 3: Breaking existing tests
**Mitigation:**
- Run full test suite after each bead
- All 49 timing tests must pass
- Add regression tests before modifying tokenize_text()

---

## Success Criteria

✅ All 4 beads (hmc, ui3, 58j, f1q) completed  
✅ All unit tests pass (`cargo test`)  
✅ All existing timing tests pass (no regression)  
✅ Code follows project conventions  
✅ PRD Section 3.3 requirements met  
✅ Design documented and committed  
✅ Ready for UI wiring (future bead)

---

**Total Estimated Effort:** 6-9 hours for MVP core engine
