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
variant. The default themes use a 2px border in the primary strong
color. Slider focus indication uses the handle border rather than the
overall widget border. Custom themes can provide any visual treatment
by implementing the Focused match arm in their style functions.

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

