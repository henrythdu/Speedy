# Refactoring Plan: Slim Down kitty.rs to Essential Code

**Status:** 2026-01-30
**Goal:** Reduce kitty.rs from 1700+ lines to ~300-400 lines by removing all unused composite/canvas code and dead code.

---

## Current State Analysis

**What's Working:**
- Per-word KGP rendering: `render_word()` → `rasterize_word()` → `transmit_graphics()`
- Clearing: `clear()` method deletes previous KGP image
- Viewport: `Viewport` struct for terminal dimensions
- Font: Font loading via `get_font()`
- Image ID management: `current_image_id` increments per word

**What's NOT Working:**
- ReadingCanvas struct and methods - full-canvas composite approach (lines 32-103)
- render_frame() canvas-based orchestrator (lines 340-418)
- composite_word() canvas-based compositing (lines 205-336)
- create_canvas() method (lines 95-103)
- calculate_start_x_for_canvas() method (lines 335-352)
- Composite rendering tests (~800 lines)

**Root Cause of Problems:**
- Competing rendering approaches: per-word vs canvas-based
- Duplicate positioning logic: calculate_start_x() vs calculate_start_x_for_canvas()
- Excessive abstractions: ReadingCanvas struct unnecessary complexity

---

## Refactoring Strategy

### Phase 1: Remove Dead Code (Lines 32-103 + 95-103 = ~200 lines)

**Action Items:**
1. Delete `ReadingCanvas` struct (lines 32-71)
2. Delete `calculate_reading_line_y()` from ReadingCanvas
3. Delete `calculate_start_x_for_canvas()` (lines 335-352)
4. Delete `composite_word()` method (lines 205-336)
5. Delete `render_frame()` implementation (lines 340-418)
6. Delete `create_canvas()` method (lines 95-103)
7. Delete `calculate_start_x_for_canvas()` helper method if it exists

**Rationale:**
- The composite/canvas approach was abandoned in favor of per-word rendering
- All canvas-related code is now dead weight
- Removing this eliminates an entire code path and its tests

### Phase 2: Clean Up Per-Word Rendering (Lines ~600-700 lines)

**Action Items:**

1. **Remove duplicate `calculate_start_x_for_canvas()` method** (~300 lines)
   - Keep only `calculate_start_x()` which is used by render_word()
   - Remove the canvas-specific variant entirely

2. **Consolidate viewport/font handling**
   - Ensure `set_reading_zone_center()` is only defined once
   - Verify `calculate_font_size_from_cell_height()` is essential

3. **Simplify test suite**
   - Remove all composite/canvas test (~800 lines)
   - Keep only essential per-word tests:
     - Test: `calculate_start_x` positioning
     - Test: `rasterize_word` creates valid image
     - Test: `render_word` clears and transmits
     - Test: `clear` deletes previous image
     - Test: `transmit_graphics` format is correct

4. **Update TuiManager to add `clear()` call**
   - Add: `RsvpRenderer::clear(&mut self.kitty_renderer)` before calling `render_word()`
   - This prevents image stacking
   - Update comments to reflect the change

**Estimated Lines After Phase 2:**
- Remove: ~1100 lines of dead code
- Remove: ~300 lines of duplicate logic
- Remove: ~800 lines of composite tests
- Add: ~10 lines to TuiManager
- **Net reduction: ~2100 lines** (from 1700 to ~600 lines)

### Phase 3: Final Code Quality (Lines ~100-150)

**Action Items:**

1. **Remove excessive comments**
   - Delete comments that explain dead/unused features
   - Keep only functional explanations
   - Target: Remove ~50-100 lines of comments

2. **Improve code organization**
   - Group related methods together
   - Ensure logical ordering: viewport → font → rendering
   - Add section comments where needed

3. **Add minimal documentation**
   - Brief docstrings for key methods
   - Example: "/// Renders word at 42% of reading zone"

**Estimated Lines After Phase 3:**
- Improve documentation: ~50-75 lines
- Improve organization: ~25-50 lines

---

## Success Criteria

**After Refactoring, kitty.rs should:**
1. **~300-400 lines total** (down from 1700)
2. **Only per-word rendering** (no canvas approach)
3. **Clearing works** (no image stacking)
4. **Words positioned correctly** (at 42% of reading zone)
5. **All tests pass** (essential per-word tests only)
6. **No dead code**
7. **Clear documentation**
8. **No duplicate logic**

---

## Implementation Steps

### Step 1: Remove Dead Code
```bash
git add src/rendering/kitty.rs
git commit -m "refactor: remove unused composite/canvas code

Per commit:
- Delete ReadingCanvas struct (was unused)
- Delete render_frame (canvas approach)
- Delete composite_word (canvas approach)
- Delete create_canvas (canvas approach)
- Delete calculate_start_x_for_canvas (duplicate)
```

### Step 2: Simplify Per-Word Rendering
```bash
# Remove duplicate calculate_start_x_for_canvas
# Remove tests for composite rendering
# Keep only essential per-word rendering
```

### Step 3: Add clear() to TuiManager
```bash
# Update TuiManager::render_frame()
# Add clear() call before render_word()
```

### Step 4: Final Cleanup
```bash
# Remove excessive comments
# Add minimal documentation
# Run tests to verify
```

---

## Risks

1. **Breaking existing functionality** - Removing too much code could break working features
2. **Incomplete removal** - Might miss dependencies or use cases
3. **Test failures** - Existing tests might rely on removed code

**Mitigations:**
- Run `cargo test` after each phase
- Test manually with `cargo run -- --force-kitty`
- Keep backup commits until stable

---

**Ready to proceed?**
This plan removes ~2000 lines of code while preserving all working functionality.
