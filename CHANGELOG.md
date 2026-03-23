# Changelog

All notable changes to plushie-iced will be documented in this file.
This changelog tracks changes **specific to this fork**. For upstream
iced changes, see the
[iced changelog](https://github.com/iced-rs/iced/blob/master/CHANGELOG.md).

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).

## [Unreleased]

### Added

- Canvas: click-to-focus -- clicking inside a focusable canvas grants
  iced-level focus (matching text_input behavior)
- Canvas: `on_focus_gained()` / `on_focus_lost()` callbacks on the
  `Program` trait for focus lifecycle notifications
- Canvas: `active_descendant_id()` on `Program` trait -- dynamically
  resolves the focused child element for accessibility
- Canvas: `.id()` builder for widget ID (enables `Command.focus(id)` targeting)
- Canvas: `.role()` builder for configurable accessible role
- Canvas: `.active_descendant()` builder for static active descendant
- `process_actions` helper function for draining `Vec<Action>` through a Shell

### Fixed

- Button: clear focus on external click even when the event was captured
  by a sibling widget. Fixes dual focus indicators and Tab going to the
  wrong widget after clicking a text field.

### Changed

- Canvas `operate()` resolves `active_descendant` dynamically from
  `Program::active_descendant_id()`, falling back to the static field

## [0.7.0] - 2026-03-21

### Added
- Canvas `Program` trait: `is_focusable()` and `operate_accessible()` methods
  for keyboard focus and shape-level accessibility
- `decorative()` builder on `Image` and `Svg` (hides from assistive technology)
- Accessible labels on `Button`, `ProgressBar`, `Slider`, `VerticalSlider`
- `alt`/`description` support on `QRCode`, `Canvas`, and `Shader`
- Tooltip-role accessible node emitted for tooltip text
- `Row`, `Cell`, `ColumnHeader` variants in accessible `Role` enum for table
  semantics
- Slider and VerticalSlider orientation set on accessible nodes

### Fixed
- Canvas keyboard events now gated on focus (mouse events unaffected)
- Focus contrast thresholds raised to meet WCAG AA SC 1.4.11
  (`focus_color` 2.0 -> 3.0, `focus_border_color` 1.5 -> 3.0)
- `named_to_code` returns `Option` instead of panicking on non-arrow keys
- Modifier state reset on window unfocus (prevents stuck keys)
- `InputMethod` state applied from non-redraw event processing

### Changed
- **Breaking:** Canvas `Program` now gates keyboard events on focus -- programs
  that handle keyboard input must return `true` from `is_focusable()` to
  continue receiving keyboard events
- **Breaking:** All crates renamed from `toddy-iced-*` to `plushie-iced-*`

## [0.6.2] - 2026-03-19

### Added
- `Display` impl for `widget::Id`

### Fixed
- `find_focused` now returns `Task<Option<Id>>` instead of `Task<Id>`,
  correctly representing the case where no widget has focus

### Changed
- **Breaking:** All crates renamed from `julep-iced-*` to `plushie-iced-*`
- **Breaking:** `find_focused` return type changed from `Task<Id>` to
  `Task<Option<Id>>`

## [0.6.1] - 2026-03-17

### Added
- Keyboard focus visibility with shadow glow and adaptive border color
  (`focus_color`, `focus_border_color`, `focus_shadow`, `focus_shadow_subtle`
  palette helpers)
- TextEditor undo/redo via Ctrl+Z / Ctrl+Y (Cmd on macOS)
- TextInput/TextEditor `input_purpose` builder for IME hints
- Markdown `code_theme` for syntax highlighting
- Modal focus trapping from accessible modal property
- `find_focused` runtime operation
- Test selector re-exports from `iced_test`

### Fixed
- `scroll_focused_into_view` now scrolls backward (was broken: target
  coordinates incorrectly included scroll translation, preventing
  backward detection). Also accounts for scrollbar dimensions and adds
  scroll margin
- ComboBox keyboard selection: Enter/Tab select the highlighted option
  and display the selected text; Tab autocompletes when the menu is
  open (captured) and moves focus when closed; Enter is ignored when
  the menu is dismissed; cursor moves to end of selected text;
  on_close callback fires in all dismiss paths
- Focus border color consistency: single base color from the palette
  with per-widget deviation (lighten/darken in oklch) when the base
  blends with the widget background, instead of switching to an
  unrelated color
- Focus shadow scaled for widget size: `focus_shadow` (prominent, for
  compact widgets) and `focus_shadow_subtle` (for large widgets)
- Styling example: removed global keyboard subscription that stole
  Space/Arrow events from focused widgets

## [0.6.0] - 2026-03-15

Based on [iced 0.14.0](https://github.com/iced-rs/iced/blob/master/CHANGELOG.md#0140---2025-12-07).

### Added
- Full accessibility tree via AccessKit with platform backends (Linux
  AT-SPI2, Windows UI Automation, macOS NSAccessibility)
- Accessible properties on all built-in widgets: role, label,
  description, value, live regions, orientation, relationships, form
  validation, read-only, busy, hidden, modal, mnemonic, radio group,
  position-in-set, size-of-set, active descendant, and has-popup
- Alt text support for Image and SVG widgets
- `announce()` API for live region announcements
- Assistive technology action handling
- ComboBox and PickList popup options exposed in the accessibility tree
- Accessibility selectors in `iced_test` for headless a11y testing
- Keyboard focus support for Button, Checkbox, Radio, Toggler, Slider,
  VerticalSlider, and PickList
- Framework-level Tab/Shift+Tab focus cycling with wrapping at
  boundaries
- Ctrl+Tab as unconditional focus escape from any widget
- Focus-visible pattern (focus ring on keyboard navigation only)
- Keyboard activation with pressed state for all focusable widgets
- Keyboard scrolling (Page Up/Down, arrows, Home/End) with Shift for
  horizontal axis
- Scroll-into-view on Tab navigation with nested scrollable cascade
- Scroll bubbling through ancestor scrollables
- Escape-to-unfocus with layered dismissal
- PaneGrid keyboard pane switching (F6/Shift+F6)
- RadioGroup widget with roving focus
- Modal dialog focus trapping via scoped focus operations
- Alt-key mnemonic activation with synthetic click injection
- Tooltip display on keyboard focus
- `runtime::keyboard` module with backend-agnostic handlers for custom
  event loops

### Fixed
- ComboBox Escape and Tab keyboard behavior (Escape now closes dropdown
  without removing focus; Tab passes through for framework focus
  navigation instead of cycling dropdown options)
- Focus wrapping gap at Tab order boundaries
- Keyboard scroll line height using renderer text size instead of
  hardcoded values

### Changed
- All crates renamed to `plushie-iced-*` for crates.io publishing
- Switched from iced-rs winit fork to upstream winit

### Removed
- macOS URL scheme handling (`event::listen_url`, `ReceivedUrl`). We
  use upstream winit which does not include this feature.
