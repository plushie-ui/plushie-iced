# Changelog

All notable changes to julep-iced will be documented in this file.
This changelog tracks changes **specific to this fork**. For upstream
iced changes, see the
[iced changelog](https://github.com/iced-rs/iced/blob/master/CHANGELOG.md).

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).

## [Unreleased]

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
- All crates renamed to `julep-iced-*` for crates.io publishing
- Switched from iced-rs winit fork to upstream winit

### Removed
- macOS URL scheme handling (`event::listen_url`, `ReceivedUrl`). We
  use upstream winit which does not include this feature.
