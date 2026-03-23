use crate::Action;
use crate::canvas::mouse;
use crate::canvas::{Event, Geometry};
use crate::core::Rectangle;
use crate::core::widget;
use crate::graphics::geometry;

/// The state and logic of a [`Canvas`].
///
/// A [`Program`] can mutate internal state and produce messages for an
/// application.
///
/// [`Canvas`]: crate::Canvas
pub trait Program<Message, Theme = crate::Theme, Renderer = crate::Renderer>
where
    Renderer: geometry::Renderer,
{
    /// The internal state mutated by the [`Program`].
    type State: Default + 'static;

    /// Updates the [`State`](Self::State) of the [`Program`].
    ///
    /// When a [`Program`] is used in a [`Canvas`], the runtime will call this
    /// method for each [`Event`].
    ///
    /// This method can optionally return an [`Action`] to either notify an
    /// application of any meaningful interactions, capture the event, or
    /// request a redraw.
    ///
    /// By default, this method does and returns nothing.
    ///
    /// [`Canvas`]: crate::Canvas
    fn update(
        &self,
        _state: &mut Self::State,
        _event: &Event,
        _bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Option<Action<Message>> {
        None
    }

    /// Draws the state of the [`Program`], producing a bunch of [`Geometry`].
    ///
    /// [`Geometry`] can be easily generated with a [`Frame`] or stored in a
    /// [`Cache`].
    ///
    /// [`Geometry`]: crate::canvas::Geometry
    /// [`Frame`]: crate::canvas::Frame
    /// [`Cache`]: crate::canvas::Cache
    fn draw(
        &self,
        state: &Self::State,
        renderer: &Renderer,
        theme: &Theme,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Vec<Geometry<Renderer>>;

    /// Returns the current mouse interaction of the [`Program`].
    ///
    /// The interaction returned will be in effect even if the cursor position
    /// is out of bounds of the program's [`Canvas`].
    ///
    /// [`Canvas`]: crate::Canvas
    fn mouse_interaction(
        &self,
        _state: &Self::State,
        _bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> mouse::Interaction {
        mouse::Interaction::default()
    }

    /// Whether the canvas should participate in iced's focus system.
    ///
    /// When true, the canvas becomes a Tab stop and keyboard events
    /// are delivered to [`update`](Self::update). The canvas widget
    /// manages focus state internally.
    ///
    /// By default, returns false (canvas is not focusable).
    fn is_focusable(&self, _state: &Self::State) -> bool {
        false
    }

    /// Called when the canvas gains iced-level focus (via Tab or click).
    ///
    /// Returns actions to emit (typically messages for the host application).
    /// The default implementation does nothing.
    fn on_focus_gained(&self, _state: &mut Self::State) -> Vec<Action<Message>> {
        vec![]
    }

    /// Called when the canvas loses iced-level focus.
    ///
    /// Returns actions to emit (typically messages for the host application).
    /// The default implementation does nothing.
    fn on_focus_lost(&self, _state: &mut Self::State) -> Vec<Action<Message>> {
        vec![]
    }

    /// Return the widget ID of the currently active child element for
    /// accessibility. Called during `operate()` to set `active_descendant`
    /// on the canvas's accessible node. Screen readers use this to
    /// announce which child has focus in a composite widget.
    ///
    /// By default, returns `None` (no active descendant).
    fn active_descendant_id(&self, _state: &Self::State) -> Option<widget::Id> {
        None
    }

    /// Emit accessible child nodes within the canvas.
    ///
    /// Called by the canvas widget's `operate()` method inside a
    /// `traverse()` block. The Program calls `operation.accessible()`
    /// directly with full [`Accessible`] structs -- no intermediate
    /// type or capability gap.
    ///
    /// For nested groups, recurse: emit the group's accessible node,
    /// then call `operation.traverse()` for the group's children.
    ///
    /// By default, does nothing (no accessible children).
    ///
    /// [`Accessible`]: crate::core::widget::operation::accessible::Accessible
    fn operate_accessible(
        &self,
        _state: &Self::State,
        _canvas_bounds: Rectangle,
        _operation: &mut dyn widget::Operation,
    ) {
    }
}

impl<Message, Theme, Renderer, T> Program<Message, Theme, Renderer> for &T
where
    Renderer: geometry::Renderer,
    T: Program<Message, Theme, Renderer>,
{
    type State = T::State;

    fn update(
        &self,
        state: &mut Self::State,
        event: &Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<Action<Message>> {
        T::update(self, state, event, bounds, cursor)
    }

    fn draw(
        &self,
        state: &Self::State,
        renderer: &Renderer,
        theme: &Theme,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Vec<Geometry<Renderer>> {
        T::draw(self, state, renderer, theme, bounds, cursor)
    }

    fn mouse_interaction(
        &self,
        state: &Self::State,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> mouse::Interaction {
        T::mouse_interaction(self, state, bounds, cursor)
    }

    fn is_focusable(&self, state: &Self::State) -> bool {
        T::is_focusable(self, state)
    }

    fn on_focus_gained(&self, state: &mut Self::State) -> Vec<Action<Message>> {
        T::on_focus_gained(self, state)
    }

    fn on_focus_lost(&self, state: &mut Self::State) -> Vec<Action<Message>> {
        T::on_focus_lost(self, state)
    }

    fn active_descendant_id(&self, state: &Self::State) -> Option<widget::Id> {
        T::active_descendant_id(self, state)
    }

    fn operate_accessible(
        &self,
        state: &Self::State,
        canvas_bounds: Rectangle,
        operation: &mut dyn widget::Operation,
    ) {
        T::operate_accessible(self, state, canvas_bounds, operation);
    }
}
