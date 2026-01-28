
# Design Doc: "Speedy" RSVP TUI

**Version:** 1.0 

**Core Stack:** Rust, Ratatui, Kitty Graphics Protocol, Imageproc, ab_glyph.

---

## 1. Executive Summary

**Speedy** is a high-performance RSVP (Rapid Serial Visual Presentation) speed reader. It solves the "distraction problem" of traditional terminals by using a **Dual-Engine Architecture**: a standard character-grid TUI for commands and a high-resolution pixel-canvas for reading. This allows for sub-pixel text positioning (the Anchor) and variable opacity context (Ghost Words).

---

## 2. The Speedy Theme System

To maintain a seamless, "homogeneous" feel between terminal cells and the image canvas, a unified color palette is utilized.

| Key | Hex | Purpose | UX Rationale |
| --- | --- | --- | --- |
| **`bg`** | Refer to PRD or update| Base Canvas | Deep slate reduces eye strain for high-speed reading. |
| **`fg`** | Refer to PRD or update | Primary Focus | Off-white provides high contrast for the active word. |
| **`accent`** | Refer to PRD or update | ORP Anchor | Vibrant pink/red catches the eye’s pivot point. |
| **`ghost`** | Refer to PRD or update| Context | 15% opacity prevents "void disorientation." |
| **`surface`** | Refer to PRD or update | Command Deck | Slightly lighter than `bg` to provide subtle depth. |

---

## 3. Visual Layout & Geometry

### 3.1 Viewport Partitioning

* Reader Zone (Top 85%):

    A single RgbaImage canvas.

    Vertical Anchor: The reading line is centered at 42% of the Reader Zone height.

    Internal Padding: 10% horizontal padding inside the canvas to buffer the word cluster.

* Command Section (Bottom 15%):

    Fixed height (~5 lines).

    Separated by a 1-line transparent gutter from the Reader.

    Uses a 4px vertical accent bar on the far left.

* Both Reader Zone and Command Section have 1 line space margin around them.

### 3.2 The Pixel-Perfect "Anchor" System

The "Anchor" letter (Optimal Recognition Point) remains mathematically stationary to eliminate eye jitter.

* **Logical ORP**: Fixed at X=CanvasWidth/2.

* **Anchor Logic**:

    W_prefix​=Pixel width of glyphs before the anchor index.

    W_anchor_center​=Half the pixel width of the anchor glyph.

    StartX​=(CanvasWidth/2)−(Wprefix​+Wanchor_center​).


* **Three-Container Model:**
* **Ghost Left:** Previous word, right-aligned, 15% opacity.
* **Focus Center:** Current word, Anchor centered on ORP, 100% opacity.
* **Ghost Right:** Next word, left-aligned, 15% opacity.



---

## 4. Progress & Spatial Awareness

### 4.1 The Macro-Gutter (Document Depth)

* **Visual:** 4px vertical bar on the extreme right of the Reader Zone.
* **Fill Logic:** Top-to-Bottom. Represents `Current Word Index / Total Words - 1` (our index starts at 0).
* **UX Detail:** Dims during Reading Mode; brightens during Pause Mode.

### 4.2 The Micro-Bar (Sentence Context)

* **Visual:** 2px high horizontal bar, 10px below the center word, length from 25% to 75% of the center container.
* **Fill Logic:** Left-to-Right. Represents progress through the current sentence.
* **Style:** Completed = `Theme.fg`, Unread = `Theme.ghost`.

---

## 5. Interaction Model & Mode States

### 5.1 Mode Transitions

| Mode | Visual State | Trigger |
| --- | --- | --- |
| **Command** | Command Bright / Reader Dimmed (30%) | `q` from Reading Mode or App Start |
| **Reading** | Reader Full / Command Dimmed (10%) | `Enter` (with content) |
| **Paused** | Reader Full / Gutter Highlighted | `Space` during Reading |

### 5.2 Key Bindings

* **Command Mode:**
* `@filepath`: Load file.
* `@@`: Load from clipboard (`arboard`).
* `:q`: Quit.
* `:h`: Help.


* **Reading/Paused Mode:**
* `Space`: Toggle Play/Pause.
* `j / k`: Forward/Backward by **Full Sentence**.
* `[ / ]`: Adjust WPM (Variable Tick Rate).
* `q`: Return to Command Mode.



---

## 6. Engineering Requirements

### 6.1 Reactive Rasterization

To prevent aliasing during terminal resizing:

1. **Listen:** Detect `SIGWINCH` resize signal.
2. **Query:** Call `ioctl(TIOCGWINSZ)` to retrieve exact pixel dimensions of Konsole.
3. **Sync:** Re-generate the `ImageBuffer` to match the pixel-per-cell ratio exactly.

### 6.2 The Rendering Pipeline (The Tick)

1. **Clear:** Create a transparent buffer with `Theme.bg`.
2. **Measure:** Use `ab_glyph` for sub-pixel metrics.
3. **Draw:** Rasterize Ghost, Anchor Word, and Progress Bars using `imageproc`.
4. **Display:** Ship to Konsole via Kitty Graphics Protocol using `ratatui-image`.

---

## 7. UX Polish (The "Speedy" Feel)

* **Anti-Jitter:** Use anti-aliased font rendering to ensure the anchor letter never "shimmers" between pixel boundaries.
* **Focus Lock:** Hide the terminal cursor (`f.set_cursor(0,0)`) while in Reading Mode to remove visual noise.
* **Content Awareness:** When jumping sentences (`j/k`), the "Ghost Word" briefly displays the last word of the previous sentence to provide a mental bridge.

## 8. Technical Implementation: The "Speedy" Tick (High Level)

    Reactive Rasterization: Query ioctl(TIOCGWINSZ) for exact pixel dimensions on SIGWINCH. The canvas is recreated to match the window scale perfectly.

    The Rendering Pipeline:

        Create a transparent ImageBuffer with Theme.bg.

        Calculate sub-pixel anchor: (Width / 2) - PrefixWidth - (AnchorWidth / 2).

        Draw the Global Gutter and Sentence Bar with imageproc::draw_filled_rect_mut.

        Draw the Word Cluster with ab_glyph.

        Ship the single completed frame to Konsole via Kitty Protocol.

    Performance: Rust's imageproc allows these operations to happen in sub-millisecond time, facilitating WPM speeds of 1000+.
