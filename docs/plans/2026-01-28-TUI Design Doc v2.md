# Design Doc: "Speedy" RSVP TUI

**Version:** 2.0

**Core Stack:** Rust, Ratatui, Kitty Graphics Protocol, Imageproc, ab_glyph.

**Status:** Revised after multi-model consensus review incorporating critical architectural improvements

---

## 1. Executive Summary

**Speedy** is a high-performance RSVP (Rapid Serial Visual Presentation) speed reader. It solves the "distraction problem" of traditional terminals by using a **Dual-Engine Architecture with Pluggable Backends**: a standard character-grid TUI for commands and a high-resolution pixel-canvas for reading. This allows for sub-pixel text positioning (the Anchor) and variable opacity context (Ghost Words).

**Key Changes from v1.0:**
- Added pluggable graphics backend architecture (Kitty Protocol initially, extensible to Sixel/iTerm2)
- Added mandatory capability detection with TUI fallback mode
- Added Word-Level LRU caching system for 1000+ WPM performance
- Added Viewport Overlay pattern to separate Ratatui and graphics rendering
- Performance optimization strategies explicitly documented

---

## 2. Architecture Overview

### 2.1 Dual-Engine Design

**Separation of Concerns:**
- **Command Layer (Ratatui):** Handles user input, REPL mode, progress bars, and standard TUI widgets
- **Rendering Layer (Graphics Engine):** Handles pixel-perfect RSVP display with sub-pixel OVP anchoring

**Viewport Overlay Pattern:**
1. Ratatui renders layout (borders, command area, stats) and reserves empty placeholder for reading zone
2. Graphics engine calculates pixel coordinates from Ratatui's placeholder area
3. Graphics engine writes directly to terminal using graphics protocol escape sequences (bypassing Ratatui buffer)
4. This avoids forcing pixel rendering into Ratatui's cell-grid workflow

### 2.2 Pluggable Graphics Backend

```rust
// src/engine/mod.rs
pub trait RsvpRenderer {
    // Core rendering interface
    fn render_word(
        &mut self,
        current: &str,
        prev: Option<&str>,
        next: Option<&str>,
        area: Rect
    ) -> Result<()>;

    fn clear(&mut self) -> Result<()>;
    fn supports_subpixel_ovp(&self) -> bool;
}

// Concrete implementations
pub struct KittyGraphicsRenderer { /* Direct Kitty Protocol */ }
pub struct CellRenderer { /* Pure TUI fallback */ }

// Future implementations
// pub struct SixelRenderer { /* Sixel Protocol */ }
// pub struct ITerm2Renderer { /* iTerm2 Graphics */ }
```

**Rationale:**
- Avoids technical debt of hardcoding Kitty-only paths
- Enables clean migration to other protocols without refactoring core logic
- Allows TUI fallback for unsupported terminals

---

## 3. The Speedy Theme System

To maintain a seamless, "homogeneous" feel between terminal cells and image canvas, a unified color palette is utilized.

| Key | Hex | Purpose | UX Rationale |
| --- | --- | --- | --- |
| **`bg`** | Refer to PRD or update| Base Canvas | Deep slate reduces eye strain for high-speed reading. |
| **`fg`** | Refer to PRD or update | Primary Focus | Off-white provides high contrast for active word. |
| **`accent`** | Refer to PRD or update | ORP Anchor | Vibrant pink/red catches eye's pivot point. |
| **`ghost`** | Refer to PRD or update| Context | 15% opacity prevents "void disorientation." |
| **`surface`** | Refer to PRD or update | Command Deck | Slightly lighter than `bg` to provide subtle depth. |

---

## 4. Visual Layout & Geometry

### 4.1 Viewport Partitioning

* **Reader Zone (Top 85%):**
    * A single RgbaImage canvas (via graphics backend)
    * Vertical Anchor: The reading line is centered at 42% of Reader Zone height
    * Internal Padding: 10% horizontal padding inside the canvas to buffer word cluster
    * Reserved by Ratatui layout, rendered by graphics engine

* **Command Section (Bottom 15%):**
    * Fixed height (~5 lines)
    * Separated by a 1-line transparent gutter from Reader
    * Uses a 4px vertical accent bar on the far left
    * Standard Ratatui widgets

* **Both Reader Zone and Command Section have 1 line space margin around them**

### 4.2 The Pixel-Perfect "Anchor" System

The "Anchor" letter (Optimal Recognition Point) remains mathematically stationary to eliminate eye jitter.

* **Logical ORP:** Fixed at X=CanvasWidth/2 (sub-pixel capable)

* **Anchor Logic:**
    * W_prefix = Pixel width of glyphs before anchor index
    * W_anchor_center = Half of pixel width of anchor glyph
    * StartX = (CanvasWidth/2) - (W_prefix + W_anchor_center)

* **Three-Container Model:**
    * **Ghost Left:** Previous word, right-aligned, 15% opacity
    * **Focus Center:** Current word, Anchor centered on ORP, 100% opacity
    * **Ghost Right:** Next word, left-aligned, 15% opacity

### 4.3 Capability Detection & Fallback

**Startup Flow:**
1. Detect terminal graphics capability via `$TERM` env variable and/or CSI queries
2. If Kitty Graphics Protocol supported → Initialize `KittyGraphicsRenderer`
3. If not supported → Initialize `CellRenderer` (pure TUI fallback)
4. Display warning message in fallback mode: "Advanced typography disabled - terminal lacks graphics protocol"

**Fallback Mode Limitations:**
- OVP snaps to nearest character cell
- Ghost words use standard dim attribute (not true opacity)
- Progress bars use standard characters
- Still fully functional as RSVP reader

---

## 5. Progress & Spatial Awareness

### 5.1 The Macro-Gutter (Document Depth)

* **Visual:** 4px vertical bar on extreme right of Reader Zone
* **Fill Logic:** Top-to-Bottom. Represents `Current Word Index / Total Words - 1` (our index starts at 0)
* **UX Detail:** Dims during Reading Mode; brightens during Pause Mode

### 5.2 The Micro-Bar (Sentence Context)

* **Visual:** 2px high horizontal bar, 10px below center word, length from 25% to 75% of center container
* **Fill Logic:** Left-to-Right. Represents progress through current sentence
* **Style:** Completed = `Theme.fg`, Unread = `Theme.ghost`

---

## 6. Performance Optimization Strategy

### 6.1 Word-Level LRU Cache (CRITICAL for 1000+ WPM)

**Problem:** At 1000 WPM (~17 words/sec), re-rasterizing every word every 60ms causes CPU spikes

**Solution:** Cache pre-rendered word buffers keyed by `(word_string, font_size, highlight_color)`

```rust
// src/engine/cache.rs
use lru::LruCache;

struct CachedWord {
    buffer: RgbaImage,    // Pre-rendered pixel data
    ovp_offset: i32,         // Pre-calculated anchor position
}

pub struct WordCache {
    cache: LruCache<String, CachedWord>,  // 1000 entries minimum
    font_size: u32,
}

impl WordCache {
    pub fn get_or_render(&mut self, word: &str) -> &CachedWord {
        let key = format!("{}|{}", word, self.font_size);
        if !self.cache.contains(&key) {
            let rendered = self.render_word_full(word);
            self.cache.put(key, rendered);
        }
        self.cache.get(&key).unwrap()
    }

    fn render_word_full(&self, word: &str) -> CachedWord {
        // Rasterize with ab_glyph → imageproc → RGBA buffer
        // Calculate OVP anchor position
        // Return cached entry
    }
}
```

**Performance Impact:**
- Cache hit: O(1) lookup (~microseconds)
- Cache miss: O(n) rasterization (~1-5ms)
- At 1000 WPM with typical English text (~20% repeated words): ~70% cache hit rate

### 6.2 CPU Compositing (CRITICAL to Avoid Flickering)

**Problem:** Sending multiple layered images (ghost words + current word) causes Z-fighting or flicker depending on terminal composition

**Solution:** Composite all words into single RGBA buffer before transmission

```rust
// src/engine/compositor.rs
pub fn render_frame(
    canvas: &mut RgbaImage,
    current: &str,
    prev: Option<&str>,
    next: Option<&str>,
    font: &FontRef,
    ovp_center: (u32, u32),
) {
    // 1. Clear to background color
    fill_canvas(canvas, Theme.bg);

    // 2. Composite previous word (alpha 0.3, right-aligned)
    if let Some(pw) = prev {
        draw_text_with_alpha(canvas, pw, font, 0.3, ghost_position_left());
    }

    // 3. Compose current word (alpha 1.0, OVP-anchored)
    draw_text_with_alpha(canvas, current, font, 1.0, ovp_center);

    // 4. Compose next word (alpha 0.3, left-aligned)
    if let Some(nw) = next {
        draw_text_with_alpha(canvas, nw, font, 0.3, ghost_position_right());
    }

    // 5. Send single image to terminal
    send_to_graphics_backend(canvas);
}
```

### 6.3 Encoding Optimization

**Problem:** Base64 + PNG/zlib encoding is CPU-intensive for every frame

**Solutions:**
1. **Image-ID Reuse:** Use Kitty Protocol's image storage to re-upload same buffer without re-encoding
2. **Minimal Canvas:** Only render text area (not full terminal dimensions)
3. **Raw RGBA Format:** Prefer raw pixel data over PNG encoding when supported
4. **Format Selection:** Use RGB24 (no alpha channel) when opacity not needed

**Performance Targets (1000+ WPM):**
- Target per-frame budget: <10ms
- Rasterization: <3ms (with cache)
- Encoding: <2ms (with image-ID reuse)
- Transmission: <5ms (depends on terminal throughput)

---

## 7. Interaction Model & Mode States

### 7.1 Mode Transitions

| Mode | Visual State | Trigger | Rendering Backend |
| --- | --- | --- | --- |
| **Command** | Command Bright / Reader Dimmed (30%) | `q` from Reading Mode or App Start | Ratatui (full) |
| **Reading** | Reader Full / Command Dimmed (10%) | `Enter` (with content) | KittyGraphicsRenderer |
| **Paused** | Reader Full / Gutter Highlighted | `Space` during Reading | KittyGraphicsRenderer |
| **Fallback** | Standard TUI (no pixel features) | Unsupported terminal detected | CellRenderer |

### 7.2 Key Bindings

* **Command Mode:**
    * `@filepath`: Load file
    * `@@`: Load from clipboard (`arboard`)
    * `:q`: Quit
    * `:h`: Help

* **Reading/Paused Mode:**
    * `Space`: Toggle Play/Pause
    * `j` / `k`: Forward/Backward by **Full Sentence**
    * `[` / `]`: Adjust WPM (Variable Tick Rate)
    * `q`: Return to Command Mode

---

## 8. Engineering Requirements

### 8.1 Reactive Rasterization

To prevent aliasing during terminal resizing:

1. **Listen:** Detect `SIGWINCH` resize signal
2. **Query:** Call `ioctl(TIOCGWINSZ)` to retrieve exact pixel dimensions of Konsole
3. **Sync:** Re-generate `ImageBuffer` to match pixel-per-cell ratio exactly
4. **Pause:** Pause reading during resize to prevent visual artifacts
5. **Resume:** Resume after resize completes

### 8.2 The Rendering Pipeline (The Tick)

1. **Clear:** Create a transparent buffer with `Theme.bg`
2. **Measure:** Use `ab_glyph` for sub-pixel metrics
3. **Cache Check:** Word cache lookup (skip rasterization on hit)
4. **Composite:** Rasterize Ghost, Anchor Word, and Progress Bars into single buffer
5. **Encode:** Optimize for transmission (image-ID reuse or minimal format)
6. **Display:** Ship to Konsole via Kitty Graphics Protocol

### 8.3 Font Management

**Bundled Font Strategy:**
- Embed JetBrains Mono OTF via `include_bytes!` macro
- Load once into `lazy_static` `FontRef` on startup
- Single font weight (~300KB) for English text
- Optional `font_path` config override for user preferences
- License compliance: Include JetBrains Mono license in `/licenses/` directory

**Future Enhancements:**
- Font selection UI
- Multiple font weights
- International font bundles for i18n

---

## 9. Dependency Management

```toml
[dependencies]
# Existing
crossterm = "0.29"
ratatui = "0.30"
unicode-segmentation = "1.12"
rustyline = "17.0"
pdf-extract = "0.10.0"
epub = "2.1.5"
arboard = { version = "3.6.1", features = ["wayland-data-control"] }
thiserror = "2.0.18"

# Graphics rendering pipeline
ab_glyph = "0.2.32"           # Font rendering and metrics
imageproc = "0.25"              # Image manipulation
lru = "0.12"                   # LRU cache for words
base64 = "0.22"                 # Kitty protocol encoding

# Future (for multi-protocol support)
# ratatui-image = "0.5.1"       # Protocol abstraction (optional)
```

---

## 10. Implementation Roadmap

### Phase 1: Foundation (1-2 weeks)
- [ ] Capability detection system (Kitty vs TUI fallback)
- [ ] Pluggable `RsvpRenderer` trait definition
- [ ] Bundled font loading with `include_bytes!`
- [ ] Kitty Graphics Protocol implementation (direct escape sequences)
- [ ] Basic word rendering (current only, no ghosting)

### Phase 2: Core Features (2-3 weeks)
- [ ] Word-Level LRU Cache (1000 entries)
- [ ] CPU compositing (single buffer with ghosting)
- [ ] Viewport Overlay pattern (Ratatui + Graphics coordination)
- [ ] Sub-pixel OVP anchoring
- [ ] Progress bars (macro gutter + micro-bar)
- [ ] SIGWINCH resize handling

### Phase 3: Optimization (1-2 weeks)
- [ ] Performance benchmarking at 1000+ WPM
- [ ] Image-ID reuse in Kitty protocol
- [ ] Encoding optimization (minimal canvas, efficient format)
- [ ] Cache hit/miss profiling
- [ ] Memory usage optimization

### Phase 4: Polish (1 week)
- [ ] Anti-aliased font rendering
- [ ] Cursor hiding in Reading Mode
- [ ] Smooth sentence transitions (ghost word bridges)
- [ ] Error handling for unsupported terminals
- [ ] User documentation (terminal requirements, config options)

### Phase 5: Future Expansion (Post-MVP)
- [ ] Sixel protocol backend
- [ ] iTerm2 graphics protocol backend
- [ ] Font selection UI
- [ ] Advanced features (animations, color fonts, ligatures)
- [ ] International font bundles

---

## 11. Known Limitations & Trade-offs

### Accepted Trade-offs

| Trade-off | Impact | Mitigation |
|-----------|--------|------------|
| **Not universal initially** | Only full features on Konsole/Kitty terminals | Clear documentation, capability detection + TUI fallback |
| **Binary size** | ~300KB for bundled JetBrains Mono | Consider `font_path` override config option |
| **Font licensing** | JetBrains Mono requires compliance | Include LICENSE file, note in README |
| **i18n limited** | English-only fonts for MVP | Document limitation, plan for future expansion |
| **Higher complexity** | Two rendering paths, protocol quirks | Pluggable backend interface from day 1 |

### Technical Limitations

- **ab_glyph limitations:** No complex text shaping (ligatures, RTL, many scripts) - English-only for MVP
- **Performance at extreme WPM:** 1000+ WPM requires aggressive caching; may have issues on slow machines
- **Terminal dependencies:** Konsole's Kitty implementation may differ from reference Kitty - requires testing
- **Resize artifacts:** Terminal resize during active reading can cause momentary visual glitches (mitigated by pause-on-resize)

---

## 12. Risk Assessment

### Critical Risks

| Risk | Severity | Probability | Mitigation |
|------|----------|------------|
| **Konsole Kitty implementation variance** | High | Medium | Test on actual Konsole build; verify sub-cell placement support |
| **Performance at 1000+ WPM** | High | Medium | Implement LRU cache Phase 1; benchmark early |
| **SIGWINCH race conditions** | Medium | High | Pause reading on resize; flush graphics buffer before resume |
| **Memory leaks in caching** | Medium | Low | Use LRU with fixed size; profile memory usage |
| **Font licensing violations** | Low | Low | Use permissive JetBrains Mono (Apache 2.0); include license in binary |

---

## 13. Success Criteria

### MVP Definition

**Must-Have:**
- [ ] Sub-pixel OVP anchoring on Konsole with Kitty Protocol
- [ ] Capability detection with TUI fallback (no blank screen failures)
- [ ] Word-Level LRU cache (1000 entries minimum)
- [ ] Single-buffer compositing (no flickering)
- [ ] Stable 1000+ WPM performance (measured on Konsole)
- [ ] SIGWINCH resize handling (no artifacts)
- [ ] All existing features functional in fallback TUI mode

**Stretch Goals:**
- [ ] <5ms per-frame render time
- [ ] 90%+ cache hit rate at typical reading speeds
- [ ] Zero visual artifacts during resize
- [ ] Memory usage <50MB during active reading

---

## 14. Why Not ratatui-image?

**Question:** Why implement Kitty Protocol directly instead of using `ratatui-image` crate?

**Answer:**

1. **Custom Rendering Requirements:** Speedy needs real-time word rendering with specific compositing (ghosting + OVP anchoring). `ratatui-image` is designed for rendering static image files, not dynamic per-frame text generation.

2. **Escape Sequence Control:** Our design requires precise control over Kitty Protocol's image placement, deletion, and encoding strategies. `ratatui-image` abstraction layer would fight against these needs.

3. **Performance Optimization:** We need to implement Word-Level LRU cache, image-ID reuse, and encoding optimizations directly. These are tightly coupled to the rendering pipeline and don't fit a generic image widget abstraction.

4. **Pluggable Architecture:** By implementing the protocol directly, we can define our own `RsvpRenderer` trait that matches our exact needs. If we later use `ratatui-image`, it would be an optional backend, not the foundation.

**Conclusion:** Direct Kitty Protocol implementation gives us the control and performance characteristics we need. `ratatui-image` (or similar crates) can be added later as **optional backends** for Sixel/iTerm2 when expanding to other terminals.

---

## 15. Technical Implementation: The "Speedy" Tick (High Level)

**Reactive Rasterization:**
Query `ioctl(TIOCGWINSZ)` for exact pixel dimensions on SIGWINCH. The canvas is recreated to match window scale perfectly.

**The Rendering Pipeline:**

1. **Viewport Overlay Phase (Ratatui):**
   * Render layout (borders, command area, progress bars)
   * Reserve empty placeholder block for reading zone
   * Extract absolute coordinates of placeholder (pixel or cell)

2. **State Sync:**
   * Update graphics engine with viewport coordinates
   * Check capability detection result

3. **Rendering Phase (Graphics Backend):**
   * Check Word-Level LRU cache for `(current_word, prev_word, next_word)`
   * On cache miss: Rasterize with ab_glyph → imageproc → RGBA buffer
   * On cache hit: Use pre-rendered buffer
   * Compose all words into single buffer with alpha blending
   * Encode optimized format (image-ID reuse or minimal format)
   * Send to terminal via protocol escape sequences

**Performance:**
Rust's imageproc allows these operations to happen in sub-millisecond time when cached. Word-Level LRU cache reduces per-frame rasterization from ~3ms to ~1ms at typical reading speeds, enabling WPM speeds of 1000+ with consistent frame timing.
