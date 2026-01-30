# Epic 2: Image-Based Word Rendering (Pixel-Perfect RSVP)

**Status:** ✅ VALIDATED (Consensus → Challenge → Synthesis Complete)
**Created:** 2026-01-29
**Version:** 2.0 (Refined from 6 tasks to 5 based on critical feedback)

---

## 1. Overview

This epic transitions from text-based rendering (Epic 1's CellRenderer) to pixel-perfect image rendering using the Kitty Graphics Protocol. The focus is on proving basic image-based rendering works reliably before adding complex optimizations.

### 1.1 Why Simplify from 6 to 5 Tasks?

**Original 6-task plan included:**
- Task 3: LRU Cache (1000 entries)
- Task 4: CPU Compositing (single buffer)

**Critical feedback from validation:**
- Cache is **premature** - we don't know if rasterization is the bottleneck
- Compositing is **unnecessary** for single-word rendering (delete + redraw works)
- Must prove "does it work on Konsole" **early**, not at the end

**Principle:** Small, focused scope enables easier debugging and pivoting.

### 1.2 Scope Boundaries

**Epic 1 (Complete):**
- ✅ Text-based CellRenderer with OVP anchoring
- ✅ Capability detection (Konsole/Kitty vs fallback)
- ✅ Font loading (JetBrains Mono via include_bytes!)
- ✅ TUI foundation with command deck

**Epic 2 (This Epic):**
- 🎯 Image-based KittyGraphicsRenderer
- 🎯 Current word only (no ghost words)
- 🎯 Basic rendering pipeline (rasterize → transmit → display)
- 🎯 Word-to-word transitions

**Epic 3 (Next):**
- Ghost words (15% opacity context)
- Progress indicators (macro-gutter, micro-bar)
- Word-Level LRU Cache (if performance measurements show need)
- SIGWINCH resize handling

---

## 2. Background & Context

### 2.1 Current State

The KittyGraphicsRenderer exists as a stub in `src/rendering/kitty.rs` with:
- ✅ RsvpRenderer trait implementation
- ✅ `calculate_start_x()` for sub-pixel OVP
- ✅ `delete_all_graphics()` cleanup method
- ✅ Base64 encoding utilities
- ❌ **Missing:** Actual rasterization with ab_glyph → imageproc
- ❌ **Missing:** Integration with TuiManager for live display

### 2.2 Key Technical Decisions

1. **Reuse Epic 1 Infrastructure:**
   - Use existing font loading (JetBrains Mono already embedded)
   - Use existing capability detection (GraphicsCapability::Kitty)
   - Use existing Viewport for coordinate calculations

2. **Single Image ID Strategy:**
   - Reuse same image ID (e.g., ID=1) for all words
   - Overwrite placement rather than delete + create new
   - Eliminates flicker from image ID changes

3. **Simple Delete + Draw:**
   - For single-word: delete previous placement, draw new word
   - Full compositing (RGBA buffer blending) deferred to Epic 3 with ghost words

---

## 3. Tasks

### Task 1: Konsole Capability Validation
**Priority:** P0 - Blocks all rendering  
**Estimated Effort:** 1 day

**Description:**
Verify Kitty Graphics Protocol works in the target terminal before building infrastructure. This is an early validation/pivot point.

**Technical Details:**
- Send a simple test image (e.g., 100x50 red rectangle) via KGP
- Verify it displays at expected position
- Test image deletion
- Document any Konsole-specific quirks

**Acceptance Criteria:**
- [ ] Test image displays correctly in Konsole 24.12+
- [ ] Image appears at specified pixel coordinates
- [ ] Image deletion works (terminal returns to previous state)
- [ ] Document any protocol quirks or version requirements

**Dependencies:** None

---

### Task 2: ab_glyph Word Rasterization
**Priority:** P0 - Core rendering  
**Estimated Effort:** 2-3 days

**Description:**
Implement text-to-image conversion using ab_glyph and imageproc. Calculate exact pixel positions for sub-pixel OVP anchoring.

**Technical Details:**
1. **Load Font:** Use Epic 1's embedded JetBrains Mono via ab_glyph
2. **Calculate OVP:** Use `calculate_start_x()` logic with pixel metrics
   - W_prefix = pixel width of glyphs before anchor
   - W_anchor_center = half of anchor glyph width
   - StartX = (canvas_width/2) - (W_prefix + W_anchor_center)
3. **Font Size Calculation:**
   - Query terminal cell dimensions (from Viewport)
   - Calculate font size to render at ~5 lines height
   - Formula: font_size_px = cell_height_px × 5 × scale_factor
   - Typical: 24-32px for standard terminals
4. **Vertical Centering:**
   - Calculate canvas height (reading zone height in pixels)
   - Calculate text bounding box height
   - Vertical offset: (canvas_height - text_height) / 2
   - Draw text at vertical center
5. **Rasterize:** Use imageproc to draw text onto RgbaImage
6. **Return:** RGBA buffer + anchor offset

**Visual Requirements:**
- Word must be **vertically centered** in the reading zone (middle of available height)
- Font size should render at approximately **5 lines height** for visibility
- Canvas should have adequate padding around the word

**Acceptance Criteria:**
- [ ] "hello" rasterizes to RgbaImage buffer
- [ ] Anchor position calculated correctly (e.g., anchor at 'e' in "hello")
- [ ] Word appears vertically centered in reading area
- [ ] Font renders at ~5 lines height (legible size)
- [ ] Buffer dimensions match text size + padding
- [ ] Unit tests for rasterization (mock or snapshot tests)

**Dependencies:** Task 1 (verify KGP works)

---

### Task 3: Kitty Protocol Image Display
**Priority:** P0 - Core transmission  
**Estimated Effort:** 2-3 days

**Description:**
Transmit rasterized words to terminal and display at calculated positions. Handle image ID lifecycle.

**Technical Details:**
1. **Encode:** Convert RgbaImage to base64
2. **Transmit:** Send APC sequence with:
   - Image ID (reuse ID=1 for all words to avoid flicker)
   - Base64-encoded RGBA data
   - Placement coordinates (U,V for absolute positioning)
3. **Position:** Use calculated pixel coordinates from Task 2
   - Horizontal: Anchor position (sub-pixel OVP)
   - Vertical: Center of reading zone (from viewport calculations)
4. **Flush:** Ensure stdout flush for immediate display

**Visual Requirements:**
- Word displays in **center** of reading zone (both horizontally and vertically)
- Anchor letter at horizontal center
- Word vertically centered in available space

**Acceptance Criteria:**
- [ ] Rasterized word displays in Konsole
- [ ] Word appears **horizontally centered** (anchor at screen center)
- [ ] Word appears **vertically centered** (middle of reading zone)
- [ ] Font size renders at ~5 lines height
- [ ] Reusing same image ID works (overwrite previous)
- [ ] No visible flicker during display

**Dependencies:** Task 2 (rasterization working)

---

### Task 4: Word-to-Word Transition
**Priority:** P0 - Reading flow  
**Estimated Effort:** 1-2 days

**Description:**
Handle smooth transitions from one word to the next. Implement delete previous + draw new pattern.

**Technical Details:**
1. **Delete Previous:** Send KGP delete command for previous placement
2. **Draw New:** Rasterize and transmit new word (reusing same image ID)
3. **Timing:** Hook into TuiManager's event loop
4. **WPM Support:** Test at various speeds (100-1000 WPM)

**Acceptance Criteria:**
- [ ] Smooth transition from word to word
- [ ] No ghost images (previous words fully deleted)
- [ ] Works at 300 WPM without flicker
- [ ] Works at 600 WPM (stress test)

**Dependencies:** Task 3 (single word display working)

---

### Task 5: Human Testing & Performance Baseline
**Priority:** P0 - Validation  
**Estimated Effort:** 1-2 days

**Description:**
Validate on actual Konsole and establish performance baseline. Decide if optimizations (cache) needed for Epic 3.

**Test Matrix:**
| Test | Description | Pass Criteria |
|------|-------------|---------------|
| Visual Rendering | Display various word lengths (1-20 chars) | All render correctly, OVP accurate |
| Flicker Test | Read at 300 WPM for 60 seconds | No visible flickering |
| Speed Test | Read at 600 WPM | Smooth transitions, no dropped frames |
| Stress Test | Read at 1000 WPM | Terminal remains responsive |
| Fallback Test | Run in non-Kitty terminal | Graceful fallback to CellRenderer |

**Metrics to Collect:**
- Average time per frame (rasterization + transmission)
- Memory usage during reading
- Terminal responsiveness at various WPM

**Decision Point:**
Based on measurements, decide:
- If rasterization > 3ms consistently → Add LRU cache in Epic 3
- If transmission > 5ms consistently → Optimize APC encoding in Epic 3
- If both < 5ms total → Skip cache, focus on features

**Acceptance Criteria:**
- [ ] All test cases pass
- [ ] Performance baseline documented
- [ ] Decision made on Epic 3 scope
- [ ] No regressions in CellRenderer fallback

**Dependencies:** Task 4 (transitions working)

---

## 4. Dependencies

### Existing from Epic 1
- ✅ Capability detection (`GraphicsCapability` enum)
- ✅ Font loading (`get_font()` with JetBrains Mono)
- ✅ Viewport coordinate calculations (`Viewport::convert_rect_to_pixels()`)
- ✅ TuiManager event loop and frame rendering
- ✅ RsvpRenderer trait and RendererError

### New for Epic 2
- ab_glyph for glyph rasterization
- imageproc for RGBA buffer manipulation
- (Already in Cargo.toml from Epic 1)

---

## 5. Success Criteria

### Must Have
- [ ] Words render as pixel-perfect images (not text characters)
- [ ] Sub-pixel OVP anchoring works (anchor letter stays fixed)
- [ ] No flickering between word transitions
- [ ] Works on Konsole 24.12+ with Kitty Graphics Protocol
- [ ] Graceful fallback to CellRenderer on non-Kitty terminals
- [ ] Performance baseline established

### Should Have
- [ ] Stable at 300 WPM (comfortable reading speed)
- [ ] Stable at 600 WPM (fast reading)
- [ ] CellRenderer fallback shows warning message

### Nice to Have
- [ ] Stable at 1000 WPM (without cache - validates baseline)
- [ ] Zero visual artifacts during word transitions

---

## 6. Risks & Mitigations

| Risk | Severity | Mitigation |
|------|----------|------------|
| Konsole KGP version incompatibility | High | Task 1 validates early; fallback to CellRenderer always works |
| Flicker from image ID changes | Medium | Reuse same image ID (overwrite placement) |
| ab_glyph rasterization performance | Medium | Measure in Task 5; add cache in Epic 3 if needed |
| APC transmission overhead | Medium | Optimize encoding in Epic 3 if bottleneck confirmed |
| Font metrics accuracy | Low | Use same JetBrains Mono from Epic 1 |

---

## 7. Next Epic Direction (Epic 3)

Based on Task 5 performance measurements:

**If cache needed (rasterization > 3ms):**
1. Word-Level LRU Cache (1000 entries)
2. Ghost words with compositing
3. Progress indicators
4. Resize handling

**If no cache needed (rasterization < 3ms):**
1. Ghost words (simple compositing)
2. Progress indicators
3. Resize handling
4. Audio metronome

---

## 8. Validation Summary

**Consensus Building (3 models):**
- google/gemini-2.5-pro: For (8/10 confidence)
- anthropic/claude-opus-4.5: Neutral (8/10 confidence)
- openai/gpt-5.2: Against → Supportive after refinements (8/10 confidence)

**Challenge Results:**
- Original 6 tasks over-engineered
- Cache and compositing premature for single-word
- Must validate Konsole works early

**Synthesis:**
- Reduced to 5 focused tasks
- Removed premature optimizations
- Added early validation (Task 1)
- Defer cache to Epic 3 based on measurements

---

*Document created following AGENTS.md workflow: Consensus → Challenge → Synthesis → Plan Documentation*
