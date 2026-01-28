# Epic 3: TUI Visual Fixes & Polish

## Overview

Fixes critical TUI rendering bugs and improves visual polish for better reading experience.

## Issues to Address

1. **OVP Anchor Moving Around** (High Priority - Bug)
   - Anchor character moves horizontally instead of staying fixed at visual center
   - Fix: Change `Alignment::Center` to `Alignment::Left` in `src/ui/render.rs`

2. **Context Words Not Dimmed Enough** (High Priority - Bug)
   - Words before/after current word are too visible, causing distraction
   - Fix: Darken dimmed color from RGB(100, 110, 150) to RGB(50, 60, 90) for ~20% opacity

3. **Progress Bar Visibility** (Medium Priority - Polish)
   - Progress bar isn't clearly visible at 1px height under current word
   - Fix: Increase to 1px line with proper contrast ratio

## Detailed Design

### Bead 3-1: Fix OVP Anchor Alignment

**Root Cause:**
```rust
// src/ui/render.rs:41
Paragraph::new(Line::from(spans))
    .alignment(Alignment::Center)  // Conflicts with padding calculation
    .style(Style::default().bg(colors::background()))
```

Padding calculation already positions word correctly for anchor:
```rust
let left_padding = (word_len / 2).saturating_sub(anchor_pos);
for _ in 0..left_padding {
    spans.push(Span::styled(" ", Style::default().fg(colors::text())));
}
```

**Fix:** Change to `Alignment::Left`
```rust
Paragraph::new(Line::from(spans))
    .alignment(Alignment::Left)  // Let padding control position
    .style(Style::default().bg(colors::background()))
```

**Acceptance Criteria:**
- Anchor character stays at fixed horizontal position for all word lengths
- Test with: "I", "hello", "extraordinary", "Antidisestablishmentarianism"

### Bead 3-2: Increase Context Word Dimming

**Current State:**
```rust
// src/ui/theme.rs:20
dimmed: Color::Rgb(100, 110, 150)  // Too bright at ~40% opacity
```

**Visual Problem:** Context words (3 before/after) are too visible:
- Main text: RGB(169, 177, 214) - bright
- Current dimmed: RGB(100, 110, 150) - still visible at ~40%

**Fix:**
```rust
dimmed: Color::Rgb(50, 60, 90)  // ~20% opacity, barely visible
```

**Acceptance Criteria:**
- Context words barely visible when scanning current word
- Still readable when user focuses on them
- Maintains WCAG AA contrast for accessibility

### Bead 3-3: Improve Progress Bar Contrast

**Current State:**
```rust
// src/ui/render.rs:55
let filled_color = colors::text();      // A9B1D6
let empty_color = colors::dimmed();     // 646E96 (too similar)
```

**Visual Problem:** Filled and empty portions lack clear distinction

**Fix:**
```rust
let filled_color = colors::text();      // A9B1D6
let empty_color = Color::Rgb(40, 45, 65); // Much darker for contrast
```

**Acceptance Criteria:**
- Progress bar clearly visible under current word
- Filled portion in text color, empty portion significantly darker
- 1px height maintained per PRD specification

## Implementation Order

1. Fix OVP anchor (1 line change)
2. Adjust dimmed color (1 line change)
3. Test visual improvements together

## Testing Checklist

- [ ] Anchor stays fixed at consistent screen position across all word lengths
- [ ] Context words barely visible (20% opacity effect)
- [ ] Progress bar clearly visible with proper contrast
- [ ] All 119 existing unit tests pass
- [ ] No performance regression (rendering still at 60fps)

## Dependencies

None - uses existing ratatui/crossterm infrastructure

## Risks

**Low Risk** - All changes are simple color/alignment adjustments

## Backwards Compatibility

**No Breaking Changes** - Internal rendering adjustments only

## PRD Alignment

- **PRD 3.1 (OVP Anchoring)**: ✅ Fixed alignment issue
- **PRD 4.1 (Midnight Theme)**: ✅ Improved contrast ratios
- **PRD 4.2 (Gutter)**: Progress indicator enhancement (partial)

---

## Summary

Epic 3 delivers immediate visual improvements with minimal implementation complexity:

1. **OVP Anchor Stability** - Anchor stays fixed for consistent reading focus
2. **Context Word Dimming** - Reduces distraction while maintaining readability
3. **Progress Bar Clarity** - Better visual feedback on reading progress

All fixes are straightforward, well-tested, and deliver immediate user value.