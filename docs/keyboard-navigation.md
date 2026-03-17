# Keyboard Navigation

How keyboard navigation works in iced, the design decisions behind
it, and the algorithms that drive scroll behavior. For contributors
and maintainers who need to understand or extend the system.

---

## Principles

Keyboard navigation is built on one guiding principle: **accessible
by default**. Application developers should not need to think about
keyboard users. The framework handles Tab navigation, focus indication,
scroll-into-view, and keyboard scrolling automatically. Widgets
participate by implementing the Focusable trait. The event capture
mechanism provides a natural opt-out -- widgets that consume a key
event prevent the framework from handling it.

No opt-in is required. No subscriptions. No configuration. All
built-in interactive widgets (Button, Checkbox, Radio, Toggler,
Slider, VerticalSlider, TextInput, TextEditor) already implement
Focusable. An iced app built with these widgets is keyboard-navigable
out of the box.

---

## Focus navigation

### Tab and Shift+Tab

The framework intercepts Tab and Shift+Tab key events at the event
loop level. When a Tab event is not captured by any widget, the
framework runs the focus_next operation (or focus_previous for
Shift+Tab), which walks the widget tree and moves focus to the next
(or previous) focusable widget. The event is suppressed from reaching
application subscriptions, preventing double-handling.

Widgets that need Tab for their own purposes (such as a text editor
using Tab for indentation) capture the event in their update handler.
The framework sees the captured status and leaves the event alone.

### Focus order

Focus order follows the depth-first traversal order of the widget
tree, which corresponds to the visual layout order (top-to-bottom,
left-to-right for standard layouts). No explicit focus order
management is needed -- the tree structure determines it.

### Initial state and focus cycling

When an application starts, no widget has focus. The first Tab press
focuses the first focusable widget.

When focus is on the last focusable widget, Tab clears focus (nothing
is focused). The next Tab press focuses the first widget again. This
one-step gap between the last and first widget may be intentional --
the unfocused state allows the user to interact with the window
itself without a widget capturing keystrokes.

### Disabled widgets

Widgets that are disabled (e.g., a button with no on_press handler)
do not register with the focus system. Their operate() method skips
the focusable call entirely, making them invisible to Tab navigation.
If a widget becomes disabled while it has focus, it immediately
unfocuses itself.

### Click-to-focus

Clicking a widget with the mouse sets focus on it, consistent with
how text inputs behave. Tab navigation after a click continues from
the clicked widget's position in the focus order. Clicking outside
a focused widget clears its focus.

### Escape to unfocus

Pressing Escape while a widget is focused unfocuses it. The widget
captures the Escape event, so parent containers (such as modals)
don't see it. This enables a layered dismissal pattern:

1. First Escape: unfocuses the focused widget (captured at the leaf)
2. Second Escape: reaches the modal container, which dismisses it
3. Tab: focuses the first widget in the remaining UI

This also restores keyboard scrolling. A focused slider captures
arrow keys and Page Up/Down for value changes. After Escape unfocuses
the slider, those keys are uncaptured and the framework's scroll
handler takes over, letting the user scroll past the slider to read
content below it.

### Ctrl+Tab

Ctrl+Tab always moves focus to the next widget, regardless of what
the currently focused widget does with Tab. Ctrl+Shift+Tab moves
focus backward. This is the emergency exit from any focus trap --
if a widget captures Tab (for indentation, for example), Ctrl+Tab
still navigates.

Ctrl+Tab runs before any widget-level event processing and ignores
capture status. It matches the convention used by GTK and Qt.

---

## Visual feedback

### Focus-visible pattern

Focus indication follows the CSS `:focus-visible` model. The focus
border is only shown when focus was gained via keyboard (Tab
navigation), not via mouse click. This prevents focus rings from
appearing whenever a user clicks a button.

Each widget tracks two internal flags: whether it has focus (for Tab
navigation positioning) and whether focus should be visible (for
rendering). The Focusable trait's focus() method -- called exclusively
by Tab navigation operations -- sets both. Click handlers set only
the focus flag, leaving the visible flag off.

This is a per-widget rendering concern, not a trait-level abstraction.
The Focusable trait's interface is unchanged. No external code queries
focus visibility -- only the widget's own status determination uses it.
Text inputs are not affected; they always show their cursor regardless
of how focus was gained.

### Focus styling

Each widget's theme Catalog provides styling for the Focused status
variant. The default themes use a consistent focus color derived from
the palette:

- **`focus_color(accent, page_bg)`** picks a single base color that
  contrasts with the page background. All widgets share this base.
- **`focus_border_color(widget_bg, accent, page_bg)`** uses the base
  when it contrasts with the widget. When they blend (e.g. an
  accent-colored button), the color is deviated (lightened/darkened
  in oklch) to create a visible border in the same hue family.
- **`focus_shadow(accent, page_bg)`** builds a prominent glow ring
  (0.85 opacity, 10px blur) for compact widgets like slider handles,
  radio buttons, checkboxes, and togglers.
- **`focus_shadow_subtle(accent, page_bg)`** is a less prominent
  variant (0.75 opacity, 6px blur) for large widgets like buttons,
  text inputs, and pick lists.

The shadow extends beyond the widget bounds and provides visibility
even when the border blends with a same-colored widget. Custom themes
can provide any visual treatment by implementing the Focused match
arm in their style functions.

---

## Widget keyboard interaction

All interactive widgets respond to keyboard input when focused:

| Widget | Keys | Behavior |
|--------|------|----------|
| Button | Space, Enter | Activates the button |
| Checkbox | Space | Toggles checked state |
| Radio | Space | Selects the option |
| Toggler | Space | Toggles on/off |
| Slider | Arrow Up/Right | Increment by step |
| Slider | Arrow Down/Left | Decrement by step |
| Slider | Page Up | Increment by 10x step |
| Slider | Page Down | Decrement by 10x step |
| Slider | Home | Set to minimum |
| Slider | End | Set to maximum |
| Vertical Slider | Same as Slider | Same behavior |
| PickList (closed) | Space, Enter, Arrow Down/Up | Opens the dropdown |
| PickList (open) | Arrow Down/Up | Navigate options (wraps) |
| PickList (open) | Space, Enter | Select hovered option and close |
| PickList (open) | Escape | Close dropdown, keep focus |
| PickList (open) | Tab | Close dropdown, move focus |
| ComboBox (open) | Arrow Down/Up | Navigate filtered options |
| ComboBox (open) | Enter | Select highlighted option, dismiss menu |
| ComboBox (open) | Tab | Autocomplete: select highlighted option, dismiss menu, keep focus |
| ComboBox (open) | Escape | Close dropdown, keep focus |
| ComboBox (dismissed) | Tab | Move focus to next widget |
| ComboBox (dismissed) | Enter | No-op (passes through) |
| All focusable | Escape | Unfocuses the widget |
| All focusable | Ctrl+Tab | Move to next widget (unconditional) |
| All focusable | Ctrl+Shift+Tab | Move to previous widget (unconditional) |

Keyboard handlers capture their events to prevent them from
propagating to the scroll system. A focused slider's Arrow Down
changes the slider value; it does not scroll the page. Escape
unfocuses the widget and captures the event, preventing parent
containers from seeing it until the second press.

---

## Event flow

When a key is pressed, processing happens in two phases:

**Phase 1: Widget update** -- the framework calls
`user_interface.update()`, which delivers the event to every widget
in the tree. Each widget's update() method decides whether to handle
and capture the event. Processing is depth-first: container widgets
call their children's update() first, then handle the event
themselves. This means:

- A focused slider captures Arrow Down (its keyboard handler fires).
- A focused button does not capture Arrow Down (no handler for it).
- The scrollable widget's cursor-over keyboard handler runs after its
  children. If a child captured the event, the scrollable skips it.
  If no child captured and the cursor is over the scrollable's bounds,
  it handles scroll keys and captures the event.

After update() returns, each event has a status: Captured or Ignored.

**Phase 2: Framework post-processing** -- the event loop checks
each event's status:

- **Ctrl+Tab/Ctrl+Shift+Tab**: always run focus_next or
  focus_previous, regardless of capture status. This is the
  unconditional escape from any focus trap.
- **Tab/Shift+Tab** (uncaptured): run focus_next or focus_previous,
  then scroll-into-view. Suppress the event from subscriptions.
- **Scroll keys** (uncaptured, Page Up/Down, arrows, Home/End): run
  scroll_focused_ancestor to scroll the focused widget's nearest
  scrollable. Suppress the event from subscriptions.

If no framework handler matched, the event is broadcast to
application subscriptions as usual.

This two-phase design ensures widgets always get first priority.
The scrollable's cursor-over handler is part of the widget update
phase, not a separate layer. The framework only handles what no
widget claimed.

---

## Keyboard scrolling

### Widget-level handler (cursor-over)

The scrollable widget handles keyboard scroll events when the mouse
cursor is over its bounds. This is the same gate used for mouse wheel
scrolling and provides precise control for mouse+keyboard users.

Supported keys: Page Up/Down, Arrow Up/Down/Left/Right, Home, End.

**Shift swaps the scroll axis** from vertical to horizontal, matching
how Shift+wheel works. Shift+Page Down scrolls right by viewport
width. Shift+Arrow Down scrolls right by line. Shift+Home scrolls
to the horizontal start, Shift+End to the horizontal end. This
applies to both the cursor-over handler and the framework handler.

Home and End are direction-aware: for vertical scrollables, they
scroll to the top and bottom. For horizontal-only scrollables, they
scroll to the left and right edges.

### Framework handler (focus-based)

When a scroll key is not captured by any widget or the cursor-over
handler, the framework scrolls the focused widget's nearest scrollable
ancestor. This enables scrolling for keyboard-only users whose cursor
may not be over a scrollable.

**Scroll bubbling**: the framework tries the innermost scrollable
ancestor first. If it is at its scroll limit for the requested
direction, it bubbles outward to the next ancestor. This continues
until a scrollable can scroll or there are no more ancestors.

For example: a focused button inside a nested scrollable. Page Down
scrolls the inner scrollable. When the inner reaches its bottom, the
next Page Down scrolls the outer scrollable. This matches how web
browsers handle nested scroll regions.

**Tree-wide fallback**: when the focused widget has no scrollable
ancestors (or all are at their limits), the framework searches the
entire widget tree for any scrollable that can scroll in the requested
direction. This handles the common case of a toolbar button above a
scrollable content area.

**Directional search order**: the fallback searches forward in DFS
order for downward/forward actions (Page Down, Arrow Down, End) and
in reverse for upward/backward actions (Page Up, Arrow Up, Home).
This makes sibling scrollable behavior symmetric: scrolling down goes
through them top-to-bottom, scrolling back up goes bottom-to-top.

**Scroll limit detection**: a scrollable is considered "at its limit"
when its current scroll position is within 0.5 pixels of the maximum
(or minimum) offset. This small epsilon prevents floating-point
rounding from causing false positives.

---

## Scroll-into-view

After Tab navigation moves focus to a new widget, the framework
scrolls any ancestor scrollable to bring the focused widget into
the visible area. This runs automatically as a follow-up operation
after focus_next or focus_previous -- no application code is needed.

### Coordinate space

The focused widget's bounds come from `layout.bounds()` in the
scrollable's `operate()` method. These are **content-space**
positions -- the scrollable origin is included but scroll translation
is not (translation is only applied during draw). The scroll offset
calculation subtracts the scrollable origin to get the widget's
position within the content, then compares against the current
scroll range `[translation, translation + visible_size]`.

This means both forward and backward scrolling work correctly: a
widget above the current viewport has a content-space position less
than the current scroll offset, triggering a leading-edge scroll.

### Scrollbar compensation and margin

When content overflows on one axis, a scrollbar appears on the
**other** axis and reduces the visible viewport. The scroll
calculation subtracts an estimated scrollbar thickness (12px) from
the cross-axis dimension. A 12px scroll margin is added around the
target widget so it isn't flush against the viewport edge or hidden
behind a scrollbar.

### Cascade algorithm

For nested scrollables, scroll-into-view uses a cascade: the
innermost scrollable adjusts to show the focused widget within its
viewport. The next scrollable out adjusts to show the inner
scrollable's bounds within its viewport. This continues outward.

Without the cascade, an outer scrollable would compute its scroll
offset from the focused widget's raw layout position, which includes
the inner scrollable's full content height. This over-scrolls the
outer container. The cascade ensures each level frames its immediate
child correctly.

**Example**: a button at the bottom of an inner scrollable, which is
itself near the bottom of an outer scrollable. Without the cascade,
the outer scrollable would scroll far past the inner scrollable to
chase the button's deep layout position. With the cascade, the inner
scrollable scrolls to show the button, and the outer scrollable
scrolls just enough to show the inner scrollable's viewport area.
Both are properly framed, and the button is visible.

---

## Custom event loops

Applications that use a custom event loop (like the integration
example) bypass the framework's built-in keyboard handling. The
keyboard navigation logic is available as standalone functions in
`runtime::keyboard`:

- `handle_ctrl_tab(event, ui, renderer)` -- unconditional focus nav
- `handle_tab(event, status, ui, renderer)` -- uncaptured Tab nav
- `handle_scroll_keys(event, status, ui, renderer)` -- scroll keys

Call these after `interface.update()` returns, passing each event
and its capture status. Each function returns `true` if it consumed
the event. Backend-specific concerns (requesting redraws, suppressing
events from subscriptions) are handled by the caller.

The integration example demonstrates this pattern.

Additional functions for specific scenarios:
- `handle_tab_within(event, status, ui, renderer, scope)` -- Tab
  cycles only within descendants of the container with the given ID.
  Used for modal dialog focus trapping.
- `handle_ctrl_tab_within(event, ui, renderer, scope)` -- same but
  unconditional.
- `handle_mnemonic(event, status, ui, renderer)` -- matches
  Alt+letter events. Returns `Option<Rectangle>` with the matched
  widget's bounds for the caller to generate a synthetic click.

---

## Modal dialog focus trapping

When a modal dialog is open, Tab should cycle only within the
dialog's content. The `handle_tab_within()` function restricts focus
cycling to descendants of a specific container.

The pattern:
1. Give the modal content container a widget ID
2. When the modal is open, handle Tab events in the app's update
   method using `focus_next_within(modal_id)` or
   `focus_previous_within(modal_id)` as the returned Task
3. Set `hidden: true` on background content's accessible metadata
   so screen readers don't navigate behind the modal
4. Set `modal: true` on the dialog container for AT semantics
5. On open: save the current focus, then focus the first widget in
   the modal
6. On close: restore focus to the saved widget

The modal example demonstrates this pattern.

---

## Alt-key mnemonics

Widgets can declare a keyboard mnemonic via the `mnemonic` field on
`Accessible`. When the user presses Alt plus the mnemonic character,
the framework focuses the widget and generates a synthetic click to
activate it.

Mnemonics work when the Alt+letter event is not captured by any
widget. Text inputs that capture Alt+letter for special character
input prevent the mnemonic from firing.

The mnemonic is mapped to accesskit's `keyboard_shortcut` property
so screen readers can announce the shortcut (e.g., "Save, button,
Alt+S").

Visual underline of the mnemonic character is not yet implemented.
The mnemonic works functionally without the visual indicator.

---

## Making a new widget focusable

To add keyboard support to a new widget:

1. Add focus tracking to the widget's internal state (an is_focused
   flag, plus a focus_visible flag for the focus-visible pattern).

2. Implement the Focusable trait (is_focused, focus, unfocus).

3. In the widget's operate() method, call operation.focusable() when
   the widget is enabled. Skip the call when disabled. Call unfocus()
   on the state when the widget becomes disabled.

4. In the widget's update() method, handle the relevant keyboard
   events when focused (Space for toggle widgets, Space+Enter for
   action widgets, arrows for range widgets). Capture handled events.
   Set focus on mouse click. Clear focus on click outside bounds.

5. Add a Focused variant to the widget's Status enum. Use
   focus_visible (not is_focused) in the status determination. Add
   Focused match arms to all style functions.

6. In the widget's draw() method, the Focused status drives visual
   feedback through the theme's style function.

The button implementation serves as the reference for this pattern.

---

