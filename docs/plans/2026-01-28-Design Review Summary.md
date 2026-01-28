# Design Review: Dual-Engine Architecture for Speedy

**Date:** 2026-01-28
**Review Method:** Superpowers Brainstorming → Pal Consensus (5 models) → Pal Challenge → Pal ThinkDeep
**Decision:** **PROCEED** with Dual-Engine Architecture (with critical modifications)

---

## Executive Summary

After comprehensive review from 5 AI models with different stances, the Dual-Engine Architecture (Ratatui + Kitty Graphics Protocol pixel canvas) is **viable for MVP** but requires **5 non-negotiable modifications** to address concerns raised during consensus and challenge phases.

**Final Confidence:** High (8/10)
**Key Insight:** This is NOT a universal solution, but a compelling niche tool for Konsole users who value precise typography over portability.

---

## Consensus Results (5 Models)

| Model | Stance | Confidence | Key Position |
|-------|--------|------------|---------------|
| Google Gemini 3-pro | **For** | 9/10 | Killer feature, innovation, performance non-issue |
| Anthropic Claude Opus 4.5 | **For** | 8/10 | Correct architectural pattern, sub-pixel OVP justifies complexity |
| OpenAI GPT-5.1-codex | **Neutral** | 7/10 | Feasible IF tightly scoped to Kitty terminals + capability detection |
| Google Gemini 2.5-pro | **Against** | 9/10 | Over-engineered, marginal benefit, portability violation |
| OpenAI GPT-5.2 | **Neutral** | 7/10 | Feasible IF Konsole-specific, performance concerns require mitigation |

**Votes:** 2 Strong Support | 2 Conditional Support | 1 Strong Opposition

---

## Challenge Findings: What Was Wrong

### Flaw 1: Overstated Consensus on "Killer Feature"
**Original Consensus Summary Claimed:** "Sub-pixel OVP anchoring is a 'killer feature' that justifies complexity"

**Challenge Reality:** Only 2/5 models used this language. The "against" model explicitly called it a "marginal enhancement" and stated that users can achieve "good enough" experience by configuring terminal fonts.

**Impact:** Summary glossed over fundamental disagreement on the core value proposition.

---

### Flaw 2: Performance Claims Lacked Nuance
**Original Consensus Summary Claimed:** "Performance at 1000+ WPM is non-issue"

**Challenge Reality:** TWO models explicitly flagged this as a risk requiring mitigation:
- **Against model:** "60ms for rasterize + transmit will likely introduce latency and high CPU usage"
- **Neutral model (GPT-5.2):** "Encoding cost is biggest cost" and requires specific mitigation strategies

**Impact:** Summary dismissed critical performance concerns that require architectural solutions (caching, encoding optimization).

---

### Flaw 3: "Bundled Font Eliminates Bugs" is Too Strong
**Original Consensus Summary Claimed:** "Bundled font is a brilliant move... eliminates cross-platform bugs"

**Challenge Reality:** Three models flagged bundling concerns:
- **Against model:** Binary bloat (300KB → megabytes for full family)
- **Neutral 1 (GPT-5.1):** License compliance obligations, i18n coverage gaps
- **Neutral 2 (GPT-5.2):** Suggests bundled default + `font_path` override

**Impact:** Summary presented bundling as purely positive when it introduces a new class of problems.

---

### Flaw 4: "Strategically Correct" Ignores "Works Everywhere" Promise
**Original Consensus Summary Claimed:** "Konsole-first is strategically correct"

**Challenge Reality:** The "against" model argues this "dramatically narrows potential user base" and "violates principle of least surprise." Both neutral models strongly emphasize capability detection + fallback.

**Impact:** Summary presented Konsole-only as a strategic win without acknowledging it breaks Speedy's implicit promise of universal terminal compatibility.

---

### Flaw 5: Mischaracterized Neutral Positions
**Original Consensus Summary Grouped:** "2 'for', 2 'neutral'"

**Challenge Reality:** The neutral models are not "neutral on should we proceed?" - they are "proceed WITH conditions and mitigations."

**Impact:** Summary underrepresented the conditional nature of neutral support, making consensus appear stronger than reality.

---

## ThinkDeep Analysis: Expert Recommendations

### Critical Modifications Required (NON-NEGOTIABLE)

#### 1. Capability Detection + TUI Fallback
**Requirement:** Detect Kitty Graphics Protocol support at startup; fall back to pure TUI mode if unsupported.

**Why:** Both "neutral" models and "against" model raised this. Without fallback, users get blank screen on non-Konsole terminals.

**Implementation:**
```rust
pub enum GraphicsCapability {
    None,           // Pure TUI fallback
    Kitty,          // Full pixel canvas
}
```

#### 2. Pluggable Graphics Backend Interface
**Requirement:** Design `RsvpRenderer` trait from day 1, abstracting Cell vs Pixel rendering.

**Why:** Neutral models warned about "hardcoding Kitty-only paths" creating technical debt for future Sixel/iTerm2 support.

**Implementation:**
```rust
pub trait RsvpRenderer {
    fn render_word(&mut self,
        current: &str,
        prev: Option<&str>,
        next: Option<&str>,
        area: Rect) -> Result<()>;
}
```

#### 3. Word-Level LRU Cache (HIGH PRIORITY)
**Requirement:** Cache pre-rendered RGBA buffers for `(word_string, font_size, highlight_color)` keys, NOT individual glyphs.

**Why:** ThinkDeep expert emphasized this. At 1000+ WPM, re-rasterizing every word every 60ms will cause CPU spikes.

**Impact:** ~70% cache hit rate at typical reading speeds with English text.

#### 4. CPU Compositing Before Sending (CRITICAL)
**Requirement:** Composite ghost words + current word into single RGBA buffer before sending to terminal. Do NOT send multiple overlapping images.

**Why:** ThinkDeep expert warned about Z-fighting and flickering. Multiple layered images cause composition issues depending on terminal implementation.

#### 5. Viewport Overlay Pattern (HIGH PRIORITY)
**Requirement:** Ratatui renders layout AND reserves empty placeholder block for RSVP area. Graphics engine writes pixels to that exact coordinate using raw Kitty escape sequences (bypassing Ratatui buffer).

**Why:** ThinkDeep expert's core architectural recommendation. Avoids forcing pixel rendering into Ratatui's cell-grid workflow.

---

## Why Not ratatui-image?

**Question:** Why implement Kitty Protocol directly instead of using `ratatui-image` crate?

**Answer:**

1. **Custom Rendering Requirements:** Speedy needs real-time word rendering with specific compositing (ghosting + OVP anchoring). `ratatui-image` is designed for rendering static image files, not dynamic per-frame text generation.

2. **Escape Sequence Control:** Our design requires precise control over Kitty Protocol's image placement, deletion, and encoding strategies. `ratatui-image` abstraction layer would fight against these needs.

3. **Performance Optimization:** We need to implement Word-Level LRU cache, image-ID reuse, and encoding optimizations directly. These are tightly coupled to the rendering pipeline and don't fit a generic image widget abstraction.

4. **Pluggable Architecture:** By implementing the protocol directly, we can define our own `RsvpRenderer` trait that matches our exact needs. If we later use `ratatui-image`, it would be an optional backend, not the foundation.

**Conclusion:** Direct Kitty Protocol implementation gives us the control and performance characteristics we need. `ratatui-image` (or similar crates) can be added later as **optional backends** for Sixel/iTerm2 when expanding to other terminals.

---

## Acknowledged Trade-offs

| Trade-off | Impact | Mitigation |
|-----------|--------|------------|
| **Not universal initially** | Only full features on Konsole/Kitty terminals | Clear documentation, capability detection + TUI fallback |
| **Binary size** | ~300KB for bundled JetBrains Mono | Consider `font_path` override config option |
| **Font licensing** | JetBrains Mono requires compliance | Include LICENSE file in docs, note in README |
| **i18n limited** | ab_glyph doesn't handle RTL/ligatures well | Accept English-only for MVP, document limitation |
| **Higher complexity** | Two rendering paths, protocol quirks | Pluggable backend interface from day 1 |
| **Performance at 1000+ WPM** | May have issues on slow machines | Aggressive caching, encoding optimization, profiling required |

---

## Performance Requirements

**Target:** 1000+ WPM (~17 words/sec, 60ms per frame budget)

**Must Implement:**
- [ ] Word-Level LRU Cache (1000 entries minimum)
- [ ] Image-ID reuse in Kitty protocol (don't re-upload same buffer)
- [ ] Minimal canvas size (only text area, not full terminal)
- [ ] Raw RGBA format preferred over PNG encoding
- [ ] Pre-rasterize "Next 100 words" in background thread
- [ ] Benchmark frame latency on actual Konsole build
- [ ] Profile CPU usage during sustained 1000 WPM reading

**Performance Targets:**
- Per-frame budget: <10ms total
- Rasterization (cache hit): <0.5ms
- Rasterization (cache miss): <3ms
- Encoding + transmission: <7ms

---

## Implementation Roadmap

### Phase 1: Foundation (1-2 weeks)
- [ ] Capability detection system (Kitty vs TUI fallback)
- [ ] Pluggable `RsvpRenderer` trait definition
- [ ] Bundled font loading with `include_bytes!`
- [ ] Kitty Graphics Protocol implementation (direct escape sequences)
- [ ] Basic word rendering (current only, no ghosting)

### Phase 2: Core Features (2-3 weeks)
- [ ] Word-Level LRU Cache
- [ ] CPU compositing (single buffer)
- [ ] Viewport Overlay pattern (Ratatui + Graphics coordination)
- [ ] Sub-pixel OVP anchoring
- [ ] Ghost words with variable opacity

### Phase 3: Progress Indicators (1 week)
- [ ] Macro-gutter (document depth)
- [ ] Micro-bar (sentence progress)
- [ ] Progress bars in both rendering modes

### Phase 4: Optimization (1-2 weeks)
- [ ] Performance benchmarking at 1000+ WPM
- [ ] Encoding optimization (image-ID reuse, minimal canvas)
- [ ] SIGWINCH resize handling
- [ ] Memory profiling and optimization

### Phase 5: Polish (1 week)
- [ ] Anti-aliased font rendering
- [ ] Cursor hiding in Reading Mode
- [ ] Smooth sentence transitions (ghost word bridges)
- [ ] Error handling for unsupported terminals
- [ ] User documentation (terminal requirements, config options)

### Phase 6: Future Expansion (Post-MVP)
- [ ] Sixel protocol backend
- [ ] iTerm2 graphics protocol backend
- [ ] Font selection UI
- [ ] Advanced features (animations, color fonts, ligatures)
- [ ] International font bundles

---

## Risk Assessment

| Risk | Severity | Probability | Mitigation |
|------|----------|------------|------------|
| **Konsole Kitty implementation variance** | High | Medium | Test on actual Konsole build; verify sub-cell placement support |
| **Performance at 1000+ WPM** | High | Medium | Implement LRU cache Phase 1; benchmark early |
| **SIGWINCH race conditions** | Medium | High | Pause reading on resize; flush graphics buffer before resume |
| **Memory leaks in caching** | Medium | Low | Use LRU with fixed size; profile memory usage |
| **Font licensing violations** | Low | Low | Use permissive JetBrains Mono (Apache 2.0); include license in binary |

---

## Final Recommendation

**PROCEED** with Dual-Engine Architecture provided you:

1. ✅ Accept Konsole-only initial scope with documented terminal requirement
2. ✅ Commit to all 5 critical modifications above (especially #1, #2, #3, and #4)
3. ✅ Treat this as a compelling niche tool for Konsole users, not universal solution
4. ✅ Document trade-offs clearly in README
5. ✅ Benchmark performance before claiming 1000+ WPM support
6. ✅ Implement Word-Level LRU cache in Phase 1 (not Phase 2)
7. ✅ Design pluggable `RsvpRenderer` trait from day 1

**If you cannot accept these constraints**, the "against" model's alternative applies: either embrace pure TUI limitations OR pivot to true GUI (egui/iced). The middle-ground (Dual-Engine) requires accepting both complexity AND portability trade-off.

---

## Design Documents

- **Original Proposal:** `docs/plans/2026-01-28-TUI Design Doc Revised.md`
- **Revised v2.0:** `docs/plans/2026-01-28-TUI Design Doc v2.md` (incorporates all consensus/challenge/thinkdeep findings)
- **This Review:** `docs/plans/2026-01-28-Design Review Summary.md`

---

## Process Notes

This review followed the **Brainstorming → Consensus → Challenge → ThinkDeep** workflow from Superpowers, leveraging multiple AI perspectives with different stances (for, against, neutral) to stress-test assumptions and identify blind spots.

**Total Time:** Comprehensive 5-model consensus + critical challenge + expert deep analysis
**Outcome:** Confirmed architectural viability with actionable implementation path addressing all major concerns.
