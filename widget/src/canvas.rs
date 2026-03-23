//! Canvases can be leveraged to draw interactive 2D graphics.
//!
//! # Example: Drawing a Simple Circle
//! ```no_run
//! # mod iced { pub mod widget { pub use iced_widget::*; } pub use iced_widget::Renderer; pub use iced_widget::core::*; }
//! # pub type State = ();
//! # pub type Element<'a, Message> = iced_widget::core::Element<'a, Message, iced_widget::Theme, iced_widget::Renderer>;
//! #
//! use iced::mouse;
//! use iced::widget::canvas;
//! use iced::{Color, Rectangle, Renderer, Theme};
//!
//! // First, we define the data we need for drawing
//! #[derive(Debug)]
//! struct Circle {
//!     radius: f32,
//! }
//!
//! // Then, we implement the `Program` trait
//! impl<Message> canvas::Program<Message> for Circle {
//!     // No internal state
//!     type State = ();
//!
//!     fn draw(
//!         &self,
//!         _state: &(),
//!         renderer: &Renderer,
//!         _theme: &Theme,
//!         bounds: Rectangle,
//!         _cursor: mouse::Cursor
//!     ) -> Vec<canvas::Geometry> {
//!         // We prepare a new `Frame`
//!         let mut frame = canvas::Frame::new(renderer, bounds.size());
//!
//!         // We create a `Path` representing a simple circle
//!         let circle = canvas::Path::circle(frame.center(), self.radius);
//!
//!         // And fill it with some color
//!         frame.fill(&circle, Color::BLACK);
//!
//!         // Then, we produce the geometry
//!         vec![frame.into_geometry()]
//!     }
//! }
//!
//! // Finally, we simply use our `Circle` to create the `Canvas`!
//! fn view<'a, Message: 'a>(_state: &'a State) -> Element<'a, Message> {
//!     canvas(Circle { radius: 50.0 }).into()
//! }
//! ```
mod program;

pub use program::Program;

pub use crate::Action;
pub use crate::core::event::Event;
pub use crate::graphics::cache::Group;
pub use crate::graphics::geometry::{
    Fill, Gradient, Image, LineCap, LineDash, LineJoin, Path, Stroke, Style, Text, fill, gradient,
    path, stroke,
};

use crate::core::event;
use crate::core::layout::{self, Layout};
use crate::core::mouse;
use crate::core::renderer;
use crate::core::widget;
use crate::core::widget::operation::accessible::{Accessible, Role};
use crate::core::widget::tree::{self, Tree};
use crate::core::window;
use crate::core::{Element, Length, Rectangle, Shell, Size, Vector, Widget};
use crate::graphics::geometry;

use std::marker::PhantomData;

/// A simple cache that stores generated [`Geometry`] to avoid recomputation.
///
/// A [`Cache`] will not redraw its geometry unless the dimensions of its layer
/// change or it is explicitly cleared.
pub type Cache<Renderer = crate::Renderer> = geometry::Cache<Renderer>;

/// The geometry supported by a renderer.
pub type Geometry<Renderer = crate::Renderer> = <Renderer as geometry::Renderer>::Geometry;

/// The frame supported by a renderer.
pub type Frame<Renderer = crate::Renderer> = geometry::Frame<Renderer>;

/// A widget capable of drawing 2D graphics.
///
/// # Example: Drawing a Simple Circle
/// ```no_run
/// # mod iced { pub mod widget { pub use iced_widget::*; } pub use iced_widget::Renderer; pub use iced_widget::core::*; }
/// # pub type State = ();
/// # pub type Element<'a, Message> = iced_widget::core::Element<'a, Message, iced_widget::Theme, iced_widget::Renderer>;
/// #
/// use iced::mouse;
/// use iced::widget::canvas;
/// use iced::{Color, Rectangle, Renderer, Theme};
///
/// // First, we define the data we need for drawing
/// #[derive(Debug)]
/// struct Circle {
///     radius: f32,
/// }
///
/// // Then, we implement the `Program` trait
/// impl<Message> canvas::Program<Message> for Circle {
///     // No internal state
///     type State = ();
///
///     fn draw(
///         &self,
///         _state: &(),
///         renderer: &Renderer,
///         _theme: &Theme,
///         bounds: Rectangle,
///         _cursor: mouse::Cursor
///     ) -> Vec<canvas::Geometry> {
///         // We prepare a new `Frame`
///         let mut frame = canvas::Frame::new(renderer, bounds.size());
///
///         // We create a `Path` representing a simple circle
///         let circle = canvas::Path::circle(frame.center(), self.radius);
///
///         // And fill it with some color
///         frame.fill(&circle, Color::BLACK);
///
///         // Then, we produce the geometry
///         vec![frame.into_geometry()]
///     }
/// }
///
/// // Finally, we simply use our `Circle` to create the `Canvas`!
/// fn view<'a, Message: 'a>(_state: &'a State) -> Element<'a, Message> {
///     canvas(Circle { radius: 50.0 }).into()
/// }
/// ```
#[derive(Debug)]
pub struct Canvas<P, Message, Theme = crate::Theme, Renderer = crate::Renderer>
where
    Renderer: geometry::Renderer,
    P: Program<Message, Theme, Renderer>,
{
    width: Length,
    height: Length,
    program: P,
    id: Option<widget::Id>,
    alt: Option<String>,
    description: Option<String>,
    role: Option<Role>,
    active_descendant: Option<widget::Id>,
    message_: PhantomData<Message>,
    theme_: PhantomData<Theme>,
    renderer_: PhantomData<Renderer>,
}

impl<P, Message, Theme, Renderer> Canvas<P, Message, Theme, Renderer>
where
    P: Program<Message, Theme, Renderer>,
    Renderer: geometry::Renderer,
{
    const DEFAULT_SIZE: f32 = 100.0;

    /// Creates a new [`Canvas`].
    pub fn new(program: P) -> Self {
        Canvas {
            width: Length::Fixed(Self::DEFAULT_SIZE),
            height: Length::Fixed(Self::DEFAULT_SIZE),
            program,
            id: None,
            alt: None,
            description: None,
            role: None,
            active_descendant: None,
            message_: PhantomData,
            theme_: PhantomData,
            renderer_: PhantomData,
        }
    }

    /// Sets the width of the [`Canvas`].
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// Sets the height of the [`Canvas`].
    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = height.into();
        self
    }

    /// Sets the alt text of the [`Canvas`].
    ///
    /// This is the accessible name announced by screen readers.
    pub fn alt(mut self, text: impl Into<String>) -> Self {
        self.alt = Some(text.into());
        self
    }

    /// Sets an extended description of the [`Canvas`].
    ///
    /// This supplements the alt text with additional context for
    /// assistive technology.
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Sets the widget ID for focus targeting and accessibility.
    ///
    /// When set, `Command.focus(id)` can target this canvas specifically.
    /// Without an ID, the canvas can still be focused via Tab navigation
    /// but cannot be targeted programmatically.
    pub fn id(mut self, id: widget::Id) -> Self {
        self.id = Some(id);
        self
    }

    /// Sets the accessible role of the canvas.
    ///
    /// Defaults to [`Role::Image`] for static canvases. Interactive canvases
    /// with focusable elements should use [`Role::Group`], [`Role::Toolbar`],
    /// [`Role::RadioGroup`], or another appropriate composite widget role.
    pub fn role(mut self, role: Role) -> Self {
        self.role = Some(role);
        self
    }

    /// Sets the active descendant for accessibility.
    ///
    /// Points to the widget ID of the currently focused child element
    /// within the canvas. Screen readers use this to announce which
    /// element has focus in a composite widget.
    pub fn active_descendant(mut self, id: widget::Id) -> Self {
        self.active_descendant = Some(id);
        self
    }
}

/// Canvas-level widget state wrapping the Program's state with focus
/// tracking and redraw suppression. The Program receives `&mut S` and
/// never sees this wrapper.
#[derive(Debug)]
struct CanvasWidgetState<S: Default + 'static> {
    program: S,
    is_focused: bool,
    /// Last mouse interaction reported by the Program. Used to detect
    /// changes and request redraws. Lives here (not on the Canvas widget
    /// struct) because the widget is rebuilt every frame by view().
    last_mouse_interaction: Option<mouse::Interaction>,
    /// Set by operate() when iced's focus system changes focus state
    /// (e.g. Tab navigation). The next update() call fires the
    /// on_focus_gained or on_focus_lost callback.
    /// `Some(true)` = gained, `Some(false)` = lost.
    pending_focus_notification: Option<bool>,
}

impl<S: Default + 'static> Default for CanvasWidgetState<S> {
    fn default() -> Self {
        Self {
            program: S::default(),
            is_focused: false,
            last_mouse_interaction: None,
            pending_focus_notification: None,
        }
    }
}

impl<S: Default + 'static> widget::operation::focusable::Focusable for CanvasWidgetState<S> {
    fn is_focused(&self) -> bool {
        self.is_focused
    }

    fn focus(&mut self) {
        self.is_focused = true;
    }

    fn unfocus(&mut self) {
        self.is_focused = false;
    }
}

/// Process a list of [`Action`]s from a Program callback, publishing
/// messages and handling redraw requests and event capture.
fn process_actions<Message>(
    actions: Vec<crate::Action<Message>>,
    shell: &mut Shell<'_, Message>,
) {
    for action in actions {
        let (message, redraw_request, event_status) = action.into_inner();
        shell.request_redraw_at(redraw_request);
        if let Some(message) = message {
            shell.publish(message);
        }
        if event_status == event::Status::Captured {
            shell.capture_event();
        }
    }
}

impl<P, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for Canvas<P, Message, Theme, Renderer>
where
    Renderer: geometry::Renderer,
    P: Program<Message, Theme, Renderer>,
{
    fn tag(&self) -> tree::Tag {
        struct Tag<T>(T);
        tree::Tag::of::<Tag<CanvasWidgetState<P::State>>>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(CanvasWidgetState::<P::State>::default())
    }

    fn size(&self) -> Size<Length> {
        Size {
            width: self.width,
            height: self.height,
        }
    }

    fn layout(
        &mut self,
        _tree: &mut Tree,
        _renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        layout::atomic(limits, self.width, self.height)
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();

        let widget_state = tree.state.downcast_mut::<CanvasWidgetState<P::State>>();
        let is_redraw_request =
            matches!(event, Event::Window(window::Event::RedrawRequested(_now)),);

        // Drain any pending focus notification queued by operate()
        // (e.g. from Tab navigation).
        if let Some(gained) = widget_state.pending_focus_notification.take() {
            let actions = if gained {
                self.program.on_focus_gained(&mut widget_state.program)
            } else {
                self.program.on_focus_lost(&mut widget_state.program)
            };
            process_actions(actions, shell);
        }

        // Only forward keyboard and IME events to the Program when the
        // canvas is focused in iced's focus system. Mouse events always
        // pass through regardless of focus state.
        let is_keyboard_like = matches!(event, Event::Keyboard(_) | Event::InputMethod(_));
        if is_keyboard_like && !widget_state.is_focused {
            return;
        }

        let was_focused = widget_state.is_focused;

        // Click-to-focus: when a left click lands inside bounds and
        // the Program is focusable, claim focus directly (matching
        // the pattern used by text_input).
        if matches!(
            event,
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
        ) && cursor.is_over(bounds)
            && self.program.is_focusable(&widget_state.program)
        {
            widget_state.is_focused = true;
        }

        if let Some(action) = self
            .program
            .update(&mut widget_state.program, event, bounds, cursor)
        {
            let (message, redraw_request, event_status) = action.into_inner();

            shell.request_redraw_at(redraw_request);

            if let Some(message) = message {
                shell.publish(message);
            }

            if event_status == event::Status::Captured {
                shell.capture_event();
            }
        }

        // Fire focus lifecycle callbacks on transitions caused by
        // click-to-focus or direct state changes within this update.
        if !was_focused && widget_state.is_focused {
            process_actions(
                self.program.on_focus_gained(&mut widget_state.program),
                shell,
            );
        } else if was_focused && !widget_state.is_focused {
            process_actions(
                self.program.on_focus_lost(&mut widget_state.program),
                shell,
            );
        }

        if shell.redraw_request() != window::RedrawRequest::NextFrame {
            let mouse_interaction =
                self.mouse_interaction(tree, layout, cursor, viewport, renderer);

            let widget_state = tree.state.downcast_mut::<CanvasWidgetState<P::State>>();
            if is_redraw_request {
                widget_state.last_mouse_interaction = Some(mouse_interaction);
            } else if widget_state
                .last_mouse_interaction
                .is_some_and(|last| last != mouse_interaction)
            {
                shell.request_redraw();
            }
        }
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _viewport: &Rectangle,
        _renderer: &Renderer,
    ) -> mouse::Interaction {
        let bounds = layout.bounds();
        let widget_state = tree.state.downcast_ref::<CanvasWidgetState<P::State>>();

        self.program
            .mouse_interaction(&widget_state.program, bounds, cursor)
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();

        if bounds.width < 1.0 || bounds.height < 1.0 {
            return;
        }

        let widget_state = tree.state.downcast_ref::<CanvasWidgetState<P::State>>();

        renderer.with_translation(Vector::new(bounds.x, bounds.y), |renderer| {
            let layers = self
                .program
                .draw(&widget_state.program, renderer, theme, bounds, cursor);

            for layer in layers {
                renderer.draw_geometry(layer);
            }
        });
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        _renderer: &Renderer,
        operation: &mut dyn widget::Operation,
    ) {
        let bounds = layout.bounds();
        let widget_state = tree.state.downcast_mut::<CanvasWidgetState<P::State>>();

        // Canvas-level accessible node. The active_descendant is resolved
        // dynamically from the Program's state (focused element ID) rather
        // than from the static Canvas struct field, so it stays in sync
        // with keyboard navigation.
        let dynamic_active_desc = self
            .program
            .active_descendant_id(&widget_state.program);
        operation.accessible(
            self.id.as_ref(),
            bounds,
            &Accessible {
                role: self.role.unwrap_or(Role::Image),
                label: self.alt.as_deref(),
                description: self.description.as_deref(),
                active_descendant: dynamic_active_desc
                    .as_ref()
                    .or(self.active_descendant.as_ref()),
                ..Accessible::default()
            },
        );

        // Focus integration: register as focusable when the Program
        // has interactive elements that need keyboard access.
        let focusable = self.program.is_focusable(&widget_state.program);
        if focusable {
            let was_focused = widget_state.is_focused;
            operation.focusable(self.id.as_ref(), bounds, widget_state);

            // Detect focus transitions caused by iced's operate pass
            // (e.g. Tab navigation, programmatic focus). Queue a
            // notification for the next update() since we have no
            // Shell here.
            if !was_focused && widget_state.is_focused {
                widget_state.pending_focus_notification = Some(true);
            } else if was_focused && !widget_state.is_focused {
                widget_state.pending_focus_notification = Some(false);
            }
        } else {
            // Program no longer accepts focus (e.g., interactive shapes
            // were removed). Clear the flag so keyboard events stop.
            if widget_state.is_focused {
                widget_state.is_focused = false;
                widget_state.pending_focus_notification = Some(false);
            }
        }

        // Accessible child nodes via the Program.
        operation.traverse(&mut |child_op| {
            self.program
                .operate_accessible(&widget_state.program, bounds, child_op);
        });
    }
}

impl<'a, P, Message, Theme, Renderer> From<Canvas<P, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: 'a,
    Renderer: 'a + geometry::Renderer,
    P: 'a + Program<Message, Theme, Renderer>,
{
    fn from(canvas: Canvas<P, Message, Theme, Renderer>) -> Element<'a, Message, Theme, Renderer> {
        Element::new(canvas)
    }
}
