//! Operate on widgets that can be focused.
use crate::widget::Id;
use crate::widget::operation::scrollable::{AbsoluteOffset, RelativeOffset, Scrollable};
use crate::widget::operation::{self, Operation, Outcome};
use crate::{Rectangle, Vector};

/// The internal state of a widget that can be focused.
pub trait Focusable {
    /// Returns whether the widget is focused or not.
    fn is_focused(&self) -> bool;

    /// Focuses the widget.
    fn focus(&mut self);

    /// Unfocuses the widget.
    fn unfocus(&mut self);
}

/// A summary of the focusable widgets present on a widget tree.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Count {
    /// The index of the current focused widget, if any.
    pub focused: Option<usize>,

    /// The total amount of focusable widgets.
    pub total: usize,
}

/// Produces an [`Operation`] that focuses the widget with the given [`Id`].
pub fn focus<T>(target: Id) -> impl Operation<T> {
    struct Focus {
        target: Id,
    }

    impl<T> Operation<T> for Focus {
        fn focusable(&mut self, id: Option<&Id>, _bounds: Rectangle, state: &mut dyn Focusable) {
            match id {
                Some(id) if id == &self.target => {
                    state.focus();
                }
                _ => {
                    state.unfocus();
                }
            }
        }

        fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation<T>)) {
            operate(self);
        }
    }

    Focus { target }
}

/// Produces an [`Operation`] that unfocuses the focused widget.
pub fn unfocus<T>() -> impl Operation<T> {
    struct Unfocus;

    impl<T> Operation<T> for Unfocus {
        fn focusable(&mut self, _id: Option<&Id>, _bounds: Rectangle, state: &mut dyn Focusable) {
            state.unfocus();
        }

        fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation<T>)) {
            operate(self);
        }
    }

    Unfocus
}

/// Produces an [`Operation`] that generates a [`Count`] and chains it with the
/// provided function to build a new [`Operation`].
pub fn count() -> impl Operation<Count> {
    struct CountFocusable {
        count: Count,
    }

    impl Operation<Count> for CountFocusable {
        fn focusable(&mut self, _id: Option<&Id>, _bounds: Rectangle, state: &mut dyn Focusable) {
            if state.is_focused() {
                self.count.focused = Some(self.count.total);
            }

            self.count.total += 1;
        }

        fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation<Count>)) {
            operate(self);
        }

        fn finish(&self) -> Outcome<Count> {
            Outcome::Some(self.count)
        }
    }

    CountFocusable {
        count: Count::default(),
    }
}

/// Produces an [`Operation`] that searches for the current focused widget, and
/// - if found, focuses the previous focusable widget.
/// - if not found, focuses the last focusable widget.
pub fn focus_previous<T>() -> impl Operation<T>
where
    T: Send + 'static,
{
    struct FocusPrevious {
        count: Count,
        current: usize,
    }

    impl<T> Operation<T> for FocusPrevious {
        fn focusable(&mut self, _id: Option<&Id>, _bounds: Rectangle, state: &mut dyn Focusable) {
            if self.count.total == 0 {
                return;
            }

            match self.count.focused {
                None if self.current == self.count.total - 1 => state.focus(),
                Some(0) if self.current == 0 => state.unfocus(),
                Some(0) => {}
                Some(focused) if focused == self.current => state.unfocus(),
                Some(focused) if focused - 1 == self.current => state.focus(),
                _ => {}
            }

            self.current += 1;
        }

        fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation<T>)) {
            operate(self);
        }
    }

    operation::then(count(), |count| FocusPrevious { count, current: 0 })
}

/// Produces an [`Operation`] that searches for the current focused widget, and
/// - if found, focuses the next focusable widget.
/// - if not found, focuses the first focusable widget.
pub fn focus_next<T>() -> impl Operation<T>
where
    T: Send + 'static,
{
    struct FocusNext {
        count: Count,
        current: usize,
    }

    impl<T> Operation<T> for FocusNext {
        fn focusable(&mut self, _id: Option<&Id>, _bounds: Rectangle, state: &mut dyn Focusable) {
            match self.count.focused {
                None if self.current == 0 => state.focus(),
                Some(focused) if focused == self.current => state.unfocus(),
                Some(focused) if focused + 1 == self.current => state.focus(),
                _ => {}
            }

            self.current += 1;
        }

        fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation<T>)) {
            operate(self);
        }
    }

    operation::then(count(), |count| FocusNext { count, current: 0 })
}

/// Produces an [`Operation`] that searches for the current focused widget
/// and stores its ID. This ignores widgets that do not have an ID.
pub fn find_focused() -> impl Operation<Id> {
    struct FindFocused {
        focused: Option<Id>,
    }

    impl Operation<Id> for FindFocused {
        fn focusable(&mut self, id: Option<&Id>, _bounds: Rectangle, state: &mut dyn Focusable) {
            if state.is_focused() && id.is_some() {
                self.focused = id.cloned();
            }
        }

        fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation<Id>)) {
            operate(self);
        }

        fn finish(&self) -> Outcome<Id> {
            if let Some(id) = &self.focused {
                Outcome::Some(id.clone())
            } else {
                Outcome::None
            }
        }
    }

    FindFocused { focused: None }
}

/// Produces an [`Operation`] that searches for the focusable widget
/// and stores whether it is focused or not. This ignores widgets that
/// do not have an ID.
pub fn is_focused(target: Id) -> impl Operation<bool> {
    struct IsFocused {
        target: Id,
        is_focused: Option<bool>,
    }

    impl Operation<bool> for IsFocused {
        fn focusable(&mut self, id: Option<&Id>, _bounds: Rectangle, state: &mut dyn Focusable) {
            if id.is_some_and(|id| *id == self.target) {
                self.is_focused = Some(state.is_focused());
            }
        }

        fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation<bool>)) {
            if self.is_focused.is_some() {
                return;
            }

            operate(self);
        }

        fn finish(&self) -> Outcome<bool> {
            self.is_focused.map_or(Outcome::None, Outcome::Some)
        }
    }

    IsFocused {
        target,
        is_focused: None,
    }
}

/// Positional information about a scrollable.
#[derive(Debug, Clone, Copy)]
struct ScrollableInfo {
    bounds: Rectangle,
    content_bounds: Rectangle,
    translation: Vector,
}

impl ScrollableInfo {
    /// Whether this scrollable can scroll further in the given direction.
    fn can_scroll(&self, action: ScrollAction) -> bool {
        let max_y = (self.content_bounds.height - self.bounds.height).max(0.0);
        let max_x = (self.content_bounds.width - self.bounds.width).max(0.0);

        match action {
            ScrollAction::PageDown | ScrollAction::LineDown => self.translation.y < max_y - 0.5,
            ScrollAction::PageUp | ScrollAction::LineUp => self.translation.y > 0.5,
            ScrollAction::LineRight => self.translation.x < max_x - 0.5,
            ScrollAction::LineLeft => self.translation.x > 0.5,
            ScrollAction::Home => self.translation.y > 0.5 || self.translation.x > 0.5,
            ScrollAction::End => {
                self.translation.y < max_y - 0.5 || self.translation.x < max_x - 0.5
            }
        }
    }
}

/// A scroll adjustment to apply to a specific scrollable.
#[derive(Debug, Clone, Copy)]
struct ScrollAdjustment {
    scrollable_bounds: Rectangle,
    offset: AbsoluteOffset<Option<f32>>,
}

/// Computes the scroll offset needed to bring `target` into view within
/// a scrollable defined by `sb` (bounds) and `t` (translation).
///
/// Returns `None` if `target` is already fully visible.
fn compute_scroll_to(
    sb: Rectangle,
    t: Vector,
    target: Rectangle,
) -> Option<AbsoluteOffset<Option<f32>>> {
    let rx = target.x - sb.x + t.x;
    let ry = target.y - sb.y + t.y;

    let mut offset_x = None;
    let mut offset_y = None;

    if target.width >= sb.width || rx < t.x {
        offset_x = Some(rx);
    } else if rx + target.width > t.x + sb.width {
        offset_x = Some(rx + target.width - sb.width);
    }

    if target.height >= sb.height || ry < t.y {
        offset_y = Some(ry);
    } else if ry + target.height > t.y + sb.height {
        offset_y = Some(ry + target.height - sb.height);
    }

    if offset_x.is_some() || offset_y.is_some() {
        Some(AbsoluteOffset {
            x: offset_x,
            y: offset_y,
        })
    } else {
        None
    }
}

/// Produces an [`Operation`] that scrolls the currently focused widget into
/// view by adjusting any scrollable ancestors whose viewport does not fully
/// contain it.
///
/// Uses a cascade approach for nested scrollables: the innermost scrollable
/// targets the focused widget, each outer scrollable targets the next inner
/// scrollable's bounds. This ensures proper framing at each nesting level.
pub fn scroll_focused_into_view<T>() -> impl Operation<T>
where
    T: Send + 'static,
{
    struct FindFocusedScrollContext {
        pending_scrollable: Option<ScrollableInfo>,
        scrollable_stack: Vec<ScrollableInfo>,
        focused_bounds: Option<Rectangle>,
        focused_ancestors: Vec<ScrollableInfo>,
    }

    impl Operation<Vec<ScrollAdjustment>> for FindFocusedScrollContext {
        fn scrollable(
            &mut self,
            _id: Option<&Id>,
            bounds: Rectangle,
            content_bounds: Rectangle,
            translation: Vector,
            _state: &mut dyn Scrollable,
        ) {
            self.pending_scrollable = Some(ScrollableInfo {
                bounds,
                content_bounds,
                translation,
            });
        }

        fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation<Vec<ScrollAdjustment>>)) {
            if let Some(info) = self.pending_scrollable.take() {
                self.scrollable_stack.push(info);
            }

            let depth = self.scrollable_stack.len();
            operate(self);
            self.scrollable_stack.truncate(depth);
        }

        fn focusable(&mut self, _id: Option<&Id>, bounds: Rectangle, state: &mut dyn Focusable) {
            if state.is_focused() {
                self.focused_bounds = Some(bounds);
                self.focused_ancestors = self.scrollable_stack.clone();
            }
        }

        fn finish(&self) -> Outcome<Vec<ScrollAdjustment>> {
            let Some(focused) = self.focused_bounds else {
                return Outcome::None;
            };

            let mut adjustments = Vec::new();

            // Cascade: innermost scrollable targets the focused widget,
            // each outer scrollable targets the next inner scrollable.
            let mut target_bounds = focused;

            for ancestor in self.focused_ancestors.iter().rev() {
                if let Some(offset) =
                    compute_scroll_to(ancestor.bounds, ancestor.translation, target_bounds)
                {
                    adjustments.push(ScrollAdjustment {
                        scrollable_bounds: ancestor.bounds,
                        offset,
                    });
                }

                // Outer ancestors target this scrollable, not the focused widget
                target_bounds = ancestor.bounds;
            }

            if adjustments.is_empty() {
                Outcome::None
            } else {
                Outcome::Some(adjustments)
            }
        }
    }

    struct ApplyScrollAdjustments {
        adjustments: Vec<ScrollAdjustment>,
        applied: usize,
    }

    impl<T> Operation<T> for ApplyScrollAdjustments {
        fn scrollable(
            &mut self,
            _id: Option<&Id>,
            bounds: Rectangle,
            _content_bounds: Rectangle,
            _translation: Vector,
            state: &mut dyn Scrollable,
        ) {
            if let Some(adj) = self
                .adjustments
                .iter()
                .find(|a| a.scrollable_bounds == bounds)
            {
                state.scroll_to(adj.offset);
                self.applied += 1;
            }
        }

        fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation<T>)) {
            if self.applied < self.adjustments.len() {
                operate(self);
            }
        }
    }

    operation::then(
        FindFocusedScrollContext {
            pending_scrollable: None,
            scrollable_stack: Vec::new(),
            focused_bounds: None,
            focused_ancestors: Vec::new(),
        },
        |adjustments| ApplyScrollAdjustments {
            adjustments,
            applied: 0,
        },
    )
}

/// A keyboard scroll action to apply to a scrollable.
#[derive(Debug, Clone, Copy)]
pub enum ScrollAction {
    /// Scroll up by viewport height.
    PageUp,
    /// Scroll down by viewport height.
    PageDown,
    /// Scroll up by one line.
    LineUp,
    /// Scroll down by one line.
    LineDown,
    /// Scroll left by one line.
    LineLeft,
    /// Scroll right by one line.
    LineRight,
    /// Scroll to the start.
    Home,
    /// Scroll to the end.
    End,
}

/// Result from phase 1 of [`scroll_focused_ancestor`].
struct ScrollTarget {
    scrollable_bounds: Rectangle,
    action: ScrollAction,
}

/// Produces an [`Operation`] that scrolls a scrollable related to the
/// currently focused widget by the given [`ScrollAction`].
///
/// Uses scroll bubbling: tries the innermost scrollable ancestor first,
/// bubbles to outer ancestors if at the scroll limit, and falls back to
/// any scrollable in the tree if no ancestor can scroll.
pub fn scroll_focused_ancestor<T>(action: ScrollAction) -> impl Operation<T>
where
    T: Send + 'static,
{
    struct FindTarget {
        action: ScrollAction,
        pending_scrollable: Option<ScrollableInfo>,
        scrollable_stack: Vec<ScrollableInfo>,
        focused_ancestors: Option<Vec<ScrollableInfo>>,
        all_scrollables: Vec<ScrollableInfo>,
    }

    impl Operation<ScrollTarget> for FindTarget {
        fn scrollable(
            &mut self,
            _id: Option<&Id>,
            bounds: Rectangle,
            content_bounds: Rectangle,
            translation: Vector,
            _state: &mut dyn Scrollable,
        ) {
            let info = ScrollableInfo {
                bounds,
                content_bounds,
                translation,
            };

            self.pending_scrollable = Some(info);
            self.all_scrollables.push(info);
        }

        fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation<ScrollTarget>)) {
            if let Some(info) = self.pending_scrollable.take() {
                self.scrollable_stack.push(info);
            }

            let depth = self.scrollable_stack.len();
            operate(self);
            self.scrollable_stack.truncate(depth);
        }

        fn focusable(&mut self, _id: Option<&Id>, _bounds: Rectangle, state: &mut dyn Focusable) {
            if state.is_focused() && self.focused_ancestors.is_none() {
                self.focused_ancestors = Some(self.scrollable_stack.clone());
            }
        }

        fn finish(&self) -> Outcome<ScrollTarget> {
            // Try ancestors innermost to outermost (scroll bubbling)
            if let Some(ancestors) = &self.focused_ancestors {
                for ancestor in ancestors.iter().rev() {
                    if ancestor.can_scroll(self.action) {
                        return Outcome::Some(ScrollTarget {
                            scrollable_bounds: ancestor.bounds,
                            action: self.action,
                        });
                    }
                }
            }

            // Fall back to any scrollable in the tree.
            // Reverse search order for upward/backward actions so
            // scrolling back through sibling scrollables is symmetric
            // with scrolling forward.
            let reverse = matches!(
                self.action,
                ScrollAction::PageUp
                    | ScrollAction::LineUp
                    | ScrollAction::LineLeft
                    | ScrollAction::Home
            );

            let find = |scrollables: &[ScrollableInfo]| {
                scrollables.iter().position(|s| s.can_scroll(self.action))
            };

            let index = if reverse {
                // Search from the end: find last scrollable that can scroll
                self.all_scrollables
                    .iter()
                    .rposition(|s| s.can_scroll(self.action))
            } else {
                find(&self.all_scrollables)
            };

            if let Some(i) = index {
                return Outcome::Some(ScrollTarget {
                    scrollable_bounds: self.all_scrollables[i].bounds,
                    action: self.action,
                });
            }

            Outcome::None
        }
    }

    struct ApplyScroll {
        target: ScrollTarget,
    }

    impl<T> Operation<T> for ApplyScroll {
        fn scrollable(
            &mut self,
            _id: Option<&Id>,
            bounds: Rectangle,
            content_bounds: Rectangle,
            _translation: Vector,
            state: &mut dyn Scrollable,
        ) {
            if bounds != self.target.scrollable_bounds {
                return;
            }

            let line_height = 20.0;

            match self.target.action {
                ScrollAction::PageDown => {
                    state.scroll_by(
                        AbsoluteOffset {
                            x: 0.0,
                            y: bounds.height,
                        },
                        bounds,
                        content_bounds,
                    );
                }
                ScrollAction::PageUp => {
                    state.scroll_by(
                        AbsoluteOffset {
                            x: 0.0,
                            y: -bounds.height,
                        },
                        bounds,
                        content_bounds,
                    );
                }
                ScrollAction::LineDown => {
                    state.scroll_by(
                        AbsoluteOffset {
                            x: 0.0,
                            y: line_height,
                        },
                        bounds,
                        content_bounds,
                    );
                }
                ScrollAction::LineUp => {
                    state.scroll_by(
                        AbsoluteOffset {
                            x: 0.0,
                            y: -line_height,
                        },
                        bounds,
                        content_bounds,
                    );
                }
                ScrollAction::LineRight => {
                    state.scroll_by(
                        AbsoluteOffset {
                            x: line_height,
                            y: 0.0,
                        },
                        bounds,
                        content_bounds,
                    );
                }
                ScrollAction::LineLeft => {
                    state.scroll_by(
                        AbsoluteOffset {
                            x: -line_height,
                            y: 0.0,
                        },
                        bounds,
                        content_bounds,
                    );
                }
                ScrollAction::Home => {
                    let overflows_x = content_bounds.width > bounds.width;
                    let overflows_y = content_bounds.height > bounds.height;

                    state.snap_to(RelativeOffset {
                        x: if overflows_x && !overflows_y {
                            Some(0.0)
                        } else {
                            None
                        },
                        y: if overflows_y || !overflows_x {
                            Some(0.0)
                        } else {
                            None
                        },
                    });
                }
                ScrollAction::End => {
                    let overflows_x = content_bounds.width > bounds.width;
                    let overflows_y = content_bounds.height > bounds.height;

                    state.snap_to(RelativeOffset {
                        x: if overflows_x && !overflows_y {
                            Some(1.0)
                        } else {
                            None
                        },
                        y: if overflows_y || !overflows_x {
                            Some(1.0)
                        } else {
                            None
                        },
                    });
                }
            }
        }

        fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation<T>)) {
            operate(self);
        }
    }

    operation::then(
        FindTarget {
            action,
            pending_scrollable: None,
            scrollable_stack: Vec::new(),
            focused_ancestors: None,
            all_scrollables: Vec::new(),
        },
        |target| ApplyScroll { target },
    )
}
