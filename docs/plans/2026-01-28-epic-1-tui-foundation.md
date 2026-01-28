# Epic 1: TUI Foundation & Basic Rendering

**Status:** Ready for Implementation  
**Created:** 2026-01-28  
**Version:** 1.0 (Validated via Consensus → Challenge → Synthesis)

---

## 1. Overview

This is the **first epic** following the major architecture pivot from REPL-based to pure TUI dual-engine architecture. It establishes the foundational infrastructure for pixel-perfect RSVP reading using the Kitty Graphics Protocol with a pure TUI fallback.

### 1.1 Scope
- **6 Technical Tasks** (max per AGENTS.md guidelines)
- **2 Human Testing Beads** (terminal compatibility, visual verification)
- **Timeline:** 1-2 weeks
- **Dependencies:** ab_glyph, imageproc, base64

### 1.2 Why This Epic First?
Without the foundation established here, subsequent epics (caching, ghosting, progress bars) have nothing to build upon. This epic delivers the minimal viable dual-engine system that runs everywhere.

---

## 2. Background & Context

### 2.1 Architecture Pivot
- **From:** REPL loop (`speedy> @file.pdf`) → Launch TUI → Return to REPL
- **To:** TUI-only startup with integrated command deck at bottom

### 2.2 Dual-Engine Design
- **Command Layer (Ratatui):** Input handling, layout, progress bars, standard TUI widgets
- **Rendering Layer (Graphics Engine):** Pixel-perfect RSVP with sub-pixel OVP anchoring
- **Viewport Overlay Pattern:** Ratatui reserves placeholder, graphics engine writes pixels directly

### 2.3 Key Technical Decisions (From Design Doc v2)
1. **Pluggable Backend:** `RsvpRenderer` trait from day 1 (prevents technical debt)
2. **Capability Detection:** Auto-detect Kitty Protocol, fallback to CellRenderer
3. **Bundled Font:** JetBrains Mono via `include_bytes!` (consistent metrics for OVP)
4. **Direct Protocol:** Implement Kitty Graphics Protocol directly (not via ratatui-image)

---

## 3. Tasks

### Task 1: Capability Detection System
**Priority:** P0 - Blocks all rendering  
**Estimated Effort:** 2-3 days

**Description:**
Implement terminal graphics capability detection that works reliably across terminal emulators with graceful fallback.

**Technical Details:**
- Query terminal via `$TERM` environment variable (unreliable but fast)
- Send CSI DA1/DA2 queries with timeout handling (reliable but slow)
- Implement `GraphicsCapability` enum with None (TUI fallback) and Kitty (full pixel canvas) variants
- Add CLI flags for manual override: `--force-kitty` and `--force-tui`
- Display warning in fallback mode

**Acceptance Criteria:**
- [ ] Detection works on Konsole, Kitty, Alacritty, GNOME Terminal
- [ ] Timeout fallback triggers within 500ms for non-responsive terminals
- [ ] Manual override flags work correctly
- [ ] Warning message displays in TUI fallback mode
- [ ] Unit tests for detection logic

**Dependencies:** None

---

### Task 2: Pluggable RsvpRenderer Trait
**Priority:** P0 - Core architecture  
**Estimated Effort:** 1-2 days

**Description:**
Define the core renderer trait that abstracts both TUI and graphics backends, enabling future protocol support (Sixel, iTerm2).

**Technical Details:**
- Define `RsvpRenderer` trait in `src/engine/renderer.rs` with methods: initialize, render_word, clear, supports_subpixel_ovp, cleanup
- Add lifecycle methods (initialize, cleanup) now to avoid breaking changes later
- Design for future backend implementations
- Document trait evolution plan for Sixel/iTerm2

**Acceptance Criteria:**
- [ ] Trait compiles with all methods
- [ ] Documentation explains each method's purpose
- [ ] Example implementations compile (can be stubs)
- [ ] Unit tests for trait bounds and object safety

**Dependencies:** None

---

### Task 3: Bundled Font Loading
**Priority:** P0 - Required for rendering  
**Estimated Effort:** 1-2 days

**Description:**
Embed JetBrains Mono font into the binary for consistent metrics and pixel-perfect OVP anchoring across all systems.

**Technical Details:**
- Download JetBrains Mono Regular OTF (~300KB)
- Embed via `include_bytes!` macro
- Load into `lazy_static` `FontRef` on startup
- Add configuration option for font_path override
- Include Apache 2.0 license in licenses directory
- Update README with font attribution

**Acceptance Criteria:**
- [ ] Font loads successfully from binary
- [ ] Font metrics available for OVP calculation
- [ ] Config override works when font_path specified
- [ ] License file included in repo
- [ ] Unit tests for font loading (mock font for tests)

**Dependencies:** ab_glyph crate

---

### Task 4: CellRenderer Implementation
**Priority:** P0 - Fallback requirement  
**Estimated Effort:** 2-3 days

**Description:**
Implement the TUI fallback renderer using pure Ratatui widgets for terminals without graphics protocol support.

**Technical Details:**
- Implement `RsvpRenderer` for `CellRenderer` struct
- Use Ratatui `Paragraph` or `Text` widgets
- OVP anchoring snaps to nearest character cell
- Display single word centered in reading zone
- No ghost words or opacity effects (Epic 2)
- Basic styling from theme system

**Acceptance Criteria:**
- [ ] Implements all `RsvpRenderer` methods
- [ ] Words display centered in reading zone
- [ ] OVP snaps to nearest cell (not sub-pixel)
- [ ] Works on any terminal (GNOME Terminal, xterm, etc.)
- [ ] Unit tests for cell-based anchoring

**Dependencies:** Task 2 (trait), Task 3 (font)

---

### Task 5: Viewport Overlay Pattern with Cell Querying
**Priority:** P0 - Critical for pixel placement  
**Estimated Effort:** 2-3 days

**Description:**
Implement the viewport overlay pattern that coordinates Ratatui layout with direct terminal graphics, including cell dimension querying for pixel-accurate coordinates.

**Technical Details:**
1. **Ratatui Layout:**
   - Reserve placeholder block in layout for reading zone
   - Standard Ratatui widgets for borders, command deck
   
2. **Cell Dimension Querying (NEW - from consensus):**
   - Send CSI 14t (text area in pixels)
   - Send CSI 18t (cell count)
   - Calculate cell dimensions
   - Store in Viewport struct

3. **Coordinate Extraction:**
   - Convert Ratatui Rect (cells) to pixel coordinates
   - Graphics engine writes directly to terminal using escape sequences
   - Bypass Ratatui buffer for graphics area only

**Acceptance Criteria:**
- [ ] CSI 14t/18t queries return valid dimensions
- [ ] Cell-to-pixel conversion is accurate
- [ ] Graphics render in correct screen location
- [ ] Placeholder area properly reserved in Ratatui layout
- [ ] Unit tests for coordinate conversion

**Dependencies:** Task 1 (capability detection)

---

### Task 6: Basic Kitty Graphics Protocol
**Priority:** P0 - Core graphics rendering  
**Estimated Effort:** 3-4 days

**Description:**
Implement direct Kitty Graphics Protocol support for pixel-perfect word rendering without ghosting or opacity effects.

**Technical Details:**
- Implement `RsvpRenderer` for `KittyGraphicsRenderer` struct
- Direct escape sequence implementation (no ratatui-image crate)
- Use ab_glyph for text rasterization
- Use imageproc for RGBA buffer creation
- Base64 encode for protocol transmission
- **Cleanup on exit (NEW - from consensus):** Delete all graphics
- Render current word only (no ghost words yet)

**Acceptance Criteria:**
- [ ] Implements all `RsvpRenderer` methods
- [ ] Words render as graphics in Kitty/Konsole
- [ ] Sub-pixel OVP positioning works
- [ ] Cleanup on exit removes all graphics
- [ ] No compilation warnings
- [ ] Unit tests for protocol encoding

**Dependencies:** Task 2 (trait), Task 3 (font), Task 5 (viewport)

---

## 4. Human Testing Beads

### Bead 7: Terminal Compatibility Testing
**Type:** Manual QA  
**Estimated Time:** 2-3 hours

**Test Matrix:**
| Terminal | Expected Behavior | Pass Criteria |
|----------|------------------|---------------|
| Konsole 24.12+ | Full graphics mode | Kitty protocol works, OVP accurate |
| Kitty 0.35+ | Full graphics mode | Kitty protocol works, OVP accurate |
| Alacritty 0.15+ | TUI fallback | Graceful fallback, warning displayed |
| GNOME Terminal 47+ | TUI fallback | Graceful fallback, warning displayed |
| xterm | TUI fallback | Graceful fallback, warning displayed |

**Test Cases:**
1. Start app on each terminal
2. Verify mode detection is correct
3. Verify warning displays in fallback mode
4. Verify `--force-kitty` and `--force-tui` flags work
5. Document any terminals that crash or hang

**Deliverable:** `docs/testing/terminal-compatibility.md` with results

---

### Bead 8: Visual Rendering Verification
**Type:** Manual QA  
**Estimated Time:** 2-3 hours

**Visual Checks:**
1. **OVP Anchoring Accuracy:**
   - Display words of varying lengths (1 char to 20+ chars)
   - Verify anchor letter stays in same screen position

2. **Font Clarity:**
   - Verify JetBrains Mono renders crisply
   - Check at different terminal sizes (80x24, 120x40, full screen)

3. **No Flickering:**
   - Read at 300 WPM for 60 seconds
   - Observe for any flickering between word transitions

4. **Resize Behavior:**
   - Start reading, then resize terminal
   - Verify: pause on resize, resume after, no corruption

**Deliverable:** `docs/testing/visual-rendering-report.md` with screenshots

---

## 5. Dependencies

### 5.1 New Cargo.toml Dependencies
```toml
[dependencies]
# Graphics rendering pipeline
ab_glyph = "0.2.32"       # Font rendering and metrics
imageproc = "0.25"        # Image manipulation  
lru = "0.12"              # LRU cache (for Epic 2)
base64 = "0.22"           # Kitty protocol encoding
lazy_static = "1.5"       # Font singleton
```

### 5.2 System Dependencies
- JetBrains Mono font file (download during development)
- Terminal with Kitty Graphics Protocol support for testing

---

## 6. Success Criteria

### 6.1 Must Have (MVP)
- [ ] App starts on ANY terminal (graceful fallback)
- [ ] Kitty Protocol works on supported terminals (Konsole, Kitty)
- [ ] Basic word display functional in both modes
- [ ] OVP anchoring accurate (sub-pixel in graphics mode, cell in TUI)
- [ ] No compilation errors or warnings
- [ ] All existing tests pass
- [ ] Graphics cleanup on exit (no lingering images)

### 6.2 Should Have
- [ ] Manual override flags work (`--force-kitty`, `--force-tui`)
- [ ] Warning message informative in fallback mode
- [ ] Font metrics accurate for OVP calculation
- [ ] Terminal resize handled gracefully

### 6.3 Nice to Have
- [ ] Performance profiling baseline (for Epic 2 comparison)
- [ ] Multiple font sizes supported

---

## 7. Validation Results

### 7.1 Consensus Summary
- **3 models consulted:** For (Gemini 3-pro), Neutral (Claude Opus 4.5), Against (GPT-5.1-codex)
- **Confidence:** 7-8/10 across all models
- **Key Agreements:**
  - 6-task scope is appropriate
  - LRU cache correctly deferred to Epic 2
  - Ghost words belong in Epic 2
  - Trait design sufficient for future backends
- **Key Concerns Addressed:**
  - Added cell dimension querying (CSI 14t/18t)
  - Added user override flags
  - Added graphics cleanup on exit
  - Added capability detection timeout handling

### 7.2 Challenge Results
**8 Assumptions Examined:**

1. **Scope appropriate?** VALIDATED - Not bloated, but added cell dimension query
2. **TUI fallback needed?** VALIDATED - Hard requirement from design doc
3. **Direct Kitty feasible?** VALIDATED - Consensus confirmed, better than ratatui-image
4. **Bundled font right choice?** VALIDATED - Enables OVP, core value prop
5. **Detection reliable?** ADDRESSED - Added timeout + user override
6. **Overlay pattern viable?** VALIDATED - Canonical approach confirmed
7. **Cache needed in Epic 1?** REJECTED - 3-5ms acceptable for testing
8. **Human testing sufficient?** ACCEPTED - Automated tests hard, manual QA required

---

## 8. Epic 2 Direction (Brief)

Following Epic 1 completion, the next epic will focus on:

1. **Word-Level LRU Cache (1000 entries)** - Performance optimization for 1000+ WPM
2. **CPU Compositing** - Single buffer with ghost words (15% opacity)
3. **Progress Indicators** - Macro-gutter (document depth) + micro-bar (sentence progress)
4. **SIGWINCH Resize Handling** - Coordinate recomputation on terminal resize
5. **Ghost Word Rendering** - Three-container model (left ghost, focus, right ghost)
6. **Reading Mode Polish** - Cursor hiding, smooth transitions

---

## 9. Risks & Mitigations

| Risk | Severity | Mitigation |
|------|----------|------------|
| **Konsole implementation variance** | High | Test on actual Konsole build; verify sub-cell placement |
| **Capability detection false negatives** | Medium | User override flags; clear documentation |
| **Cell dimension query unsupported** | Medium | Fallback to terminal size estimation |
| **Font loading failures** | Medium | Config override option; error messages |
| **Graphics cleanup failures** | Low | Test exit paths; add panic cleanup |

---

## 10. Notes

### 10.1 Why Not ratatui-image?
Per Design Doc v2 Section 14:
1. Custom rendering requirements (ghosting + OVP)
2. Need precise escape sequence control
3. Performance optimizations (caching, encoding)
4. Pluggable architecture requirements

### 10.2 Font Licensing
- JetBrains Mono: Apache 2.0 (permissive)
- Must include LICENSE file in binary
- Attribution in README required

### 10.3 Testing Strategy
- **Unit tests:** Each task has specific test requirements
- **Integration tests:** Test full rendering pipeline
- **Human testing:** Terminal compatibility, visual verification
- **No automated terminal tests** - too complex for Epic 1

---

## 11. Approval

**Plan Status:** VALIDATED AND READY

This plan has been validated through:
- Consensus building (3 models, 7-8/10 confidence)
- Challenge of assumptions (8 assumptions examined)
- Synthesis of findings

**Next Step:** Create beads using `bd create` for each task

---

## Appendix A: File Structure Changes

```
src/
├── engine/
│   ├── mod.rs              # Add renderer module
│   ├── renderer.rs         # NEW: RsvpRenderer trait
│   ├── cell_renderer.rs    # NEW: CellRenderer implementation
│   ├── kitty_renderer.rs   # NEW: KittyGraphicsRenderer implementation
│   ├── viewport.rs         # NEW: Viewport coordinate management
│   └── font.rs             # NEW: Bundled font loading
├── ui/
│   └── ...                 # Existing files
├── config.rs               # Add font_path option
└── main.rs                 # Add CLI flags

assets/
└── fonts/
    └── JetBrainsMono-Regular.otf

licenses/
└── jetbrains-mono-LICENSE.txt
```

## Appendix B: CLI Interface

```bash
# Auto-detect (default)
speedy

# Force graphics mode
speedy --force-kitty

# Force TUI mode
speedy --force-tui

# With file
speedy @document.pdf

# From clipboard
speedy @@
```

---

*This document follows the AGENTS.md epic planning workflow: Consensus → Challenge → Synthesis → Plan Documentation*
