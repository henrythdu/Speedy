# Fonts Directory

This directory contains embedded fonts for Speedy.

## JetBrains Mono

- **File:** `JetBrainsMono-Regular.otf`
- **Source:** https://www.jetbrains.com/lp/mono/
- **License:** SIL Open Font License 1.1 (see `licenses/JetBrainsMono-LICENSE.txt`)
- **Purpose:** Primary font for RSVP rendering and OVP calculation
- **Embedded Size:** ~140KB

The font is embedded in the binary using Rust's `include_bytes!` macro.
