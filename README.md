# plushie-iced

Vendored fork of [iced](https://github.com/iced-rs/iced) maintained
for the [Plushie UI](https://github.com/plushie-ui) project.
**[Pre-1.0](#status)**

Tracks iced's master branch with additional features. Version numbers
are independent of upstream iced releases.

## What's different from upstream iced

### Accessibility

- Full accessibility tree via [AccessKit](https://accesskit.dev)
  (Linux AT-SPI2, Windows UI Automation, macOS NSAccessibility)
- Accessible properties on all built-in widgets: role, label,
  description, value, live regions, orientation, relationships
  (labelled-by, described-by), and more
- Alt text and `decorative()` on Image and SVG
- Tooltip and table (Row, Cell, ColumnHeader) roles
- RadioGroup widget with proper radio group semantics
  (single Tab stop, arrow key navigation, active descendant tracking)
- Canvas `operate_accessible()`, `is_focusable()`, and
  `active_descendant_id()` for element-level a11y and keyboard focus
- Canvas click-to-focus and focus lifecycle callbacks
  (`on_focus_gained`, `on_focus_lost`)
- Canvas widget ID, configurable role, and active descendant support
- Form validation, modal, busy, hidden, and read-only properties
- Alt-key mnemonics mapped to AccessKit keyboard shortcuts

### Keyboard navigation

- Tab/Shift+Tab focus cycling across all interactive widgets
  (including focusable canvases)
- Ctrl+Tab unconditional focus escape from any widget
- Focus-visible pattern (focus ring on keyboard navigation only,
  not on mouse clicks)
- Keyboard activation: Button (Space/Enter), Checkbox (Space),
  Radio (Space), Toggler (Space), Slider (arrows, Home/End,
  Page Up/Down), PickList (arrows, Enter/Space, Escape), ComboBox
  (arrow navigation with Escape state preservation, Tab autocomplete,
  Enter/Tab select from dropdown)
- Keyboard scrolling: Page Up/Down, arrows, Home/End; Shift swaps
  to horizontal axis
- Scroll-into-view on Tab navigation with nested scrollable cascade
  (properly handles backward scrolling and scrollbar overlap)
- Scroll bubbling through ancestor scrollables with tree-wide
  directional fallback
- PaneGrid keyboard pane switching (F6/Shift+F6)
- Modal dialog focus trapping via scoped focus operations
- Alt-key mnemonic activation with synthetic click injection
- Escape-to-unfocus with layered dismissal (widget first, then
  container)

### Framework APIs

- `runtime::keyboard` module with backend-agnostic keyboard
  handlers for custom event loops
- Scoped focus operations (`focus_next_within`, `focus_previous_within`)
  for modal and dialog patterns
- Mnemonic lookup operation for Alt+letter widget activation
- Accessibility selectors in `iced_test` for headless a11y testing
- `focus_color`, `focus_border_color`, `focus_shadow`,
  `focus_shadow_subtle` palette helpers for consistent focus indicators
- TextEditor undo/redo (Ctrl+Z/Ctrl+Y, Cmd on macOS)
- TextInput/TextEditor `input_purpose` for IME hints
- Markdown `code_theme` for syntax highlighting themes
- `find_focused` runtime operation
- Runtime font loading at any time (not just startup)

### Removed from upstream

- **macOS URL scheme handling** (`event::listen_url`,
  `ReceivedUrl` subscription event). Upstream iced uses a forked
  winit to support macOS deep links / custom URL schemes. This fork
  switches to upstream winit from crates.io, which doesn't include
  that feature. We'd like to support it once upstream winit does.

### Crate naming

All crates are published under a `plushie-iced-` prefix. Cargo's
`package` aliasing means Rust source code still uses `use iced::*`
-- the renaming is entirely in Cargo.toml.

## Status

Pre-1.0. The accessibility and keyboard navigation layers are
functional but the API is still evolving -- especially the canvas
`Program` trait and accessible operation interfaces.

- Pin to an exact version and read the
  [CHANGELOG](CHANGELOG.md) when upgrading.
- The canvas accessibility API (`operate_accessible`,
  `is_focusable`) is the least stable part of the public surface.

## License

MIT -- same as upstream iced.
