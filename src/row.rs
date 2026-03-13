use iced_core::{
    Alignment, Element, Event, Layout, Length, Padding, Pixels, Rectangle, Shell, Size, Vector,
    Widget,
    animation::Easing,
    layout::{self, flex::Axis},
    mouse, overlay,
    widget::{
        Operation, Tree,
        tree::{self},
    },
};

use crate::axial::AnimatedAxial;

pub struct AnimatedRow<
    'a,
    const N: usize,
    Message: Clone,
    Theme = iced_core::Theme,
    Renderer = iced_renderer::Renderer,
>(AnimatedAxial<'a, N, Message, Theme, Renderer>);

impl<'a, const N: usize, Message: Clone, Theme: Clone, Renderer: iced_core::Renderer>
    AnimatedRow<'a, N, Message, Theme, Renderer>
{
    /// Creates a [`AnimatedRow`]
    ///
    /// If any of the children have a [`Length::Fill`] strategy, you will need to
    /// call [`AnimatedRow::width`] or [`AnimatedRow::height`] accordingly.
    pub fn new(children: [Element<'a, Message, Theme, Renderer>; N]) -> Self {
        Self(AnimatedAxial {
            spacing: 0.0,
            padding: Padding::ZERO,
            width: Length::Shrink,
            height: Length::Shrink,
            max_length: None,
            align: Alignment::Start,
            clip: false,
            children,
            milliseconds_per_pixel: 1.0,
            easing: Easing::Linear,
            on_animation_start: None,
            on_animation_end: None,
            axis: Axis::Horizontal,
        })
    }

    /// Sets the [`Easing`]. Check out [https://easings.net/]
    pub const fn easing(mut self, easing: Easing) -> Self {
        self.0.easing = easing;
        self
    }

    /// Sets the ratio between pixels needing to be changed and the [`Duration`] of the animation
    pub fn milliseconds_per_pixel(mut self, miliseconds: f32) -> Self {
        self.0.milliseconds_per_pixel = miliseconds;
        self
    }

    /// Sets the horizontal spacing _between_ elements.
    ///
    /// Custom margins per element do not exist in iced. You should use this
    /// method instead! While less flexible, it helps you keep spacing between
    /// elements consistent.
    pub fn spacing(mut self, amount: impl Into<Pixels>) -> Self {
        self.0.spacing = amount.into().0;
        self
    }

    /// Sets the [`Padding`] of the [`AnimatedRow`].
    pub fn padding<P: Into<Padding>>(mut self, padding: P) -> Self {
        self.0.padding = padding.into();
        self
    }

    /// Sets the width of the [`AnimatedRow`].
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.0.width = width.into();
        self
    }

    /// Sets the height of the [`AnimatedRow`].
    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.0.height = height.into();
        self
    }

    /// Sets the maximum height of the [`AnimatedRow`].
    pub fn max_height(mut self, max_length: impl Into<Pixels>) -> Self {
        self.0.max_length = Some(max_length.into().0);
        self
    }

    /// Sets the alignment of the contents of the [`AnimatedRow`] .
    pub fn align(mut self, align: impl Into<Alignment>) -> Self {
        self.0.align = align.into();
        self
    }

    /// Sets whether the contents of the [`AnimatedRow`] should be clipped on
    /// overflow.
    pub fn clip(mut self, clip: bool) -> Self {
        self.0.clip = clip;
        self
    }

    pub fn on_animation_start(mut self, msg: Message) -> Self {
        self.0.on_animation_start = Some(msg);
        self
    }

    pub fn on_animation_end(mut self, msg: Message) -> Self {
        self.0.on_animation_end = Some(msg);
        self
    }
}

impl<'a, const N: usize, Message: Clone, Theme: Clone, Renderer: iced_core::Renderer>
    Widget<Message, Theme, Renderer> for AnimatedRow<'a, N, Message, Theme, Renderer>
{
    fn size(&self) -> Size<Length> {
        self.0.size()
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        self.0.layout(tree, renderer, limits)
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &iced_core::renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        self.0
            .draw(tree, renderer, theme, style, layout, cursor, viewport);
    }

    fn size_hint(&self) -> Size<Length> {
        self.0.size_hint()
    }

    fn tag(&self) -> tree::Tag {
        self.0.tag()
    }

    fn state(&self) -> tree::State {
        self.0.state()
    }

    fn children(&self) -> Vec<Tree> {
        self.0.children()
    }

    fn diff(&self, tree: &mut Tree) {
        self.0.diff(tree);
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn Operation,
    ) {
        self.0.operate(tree, layout, renderer, operation);
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
        self.0
            .update(tree, event, layout, cursor, renderer, shell, viewport);
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.0
            .mouse_interaction(tree, layout, cursor, viewport, renderer)
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        self.0
            .overlay(tree, layout, renderer, viewport, translation)
    }
}

impl<'a, const N: usize, Message: Clone + 'a, Theme: Clone + 'a, Renderer: iced_core::Renderer + 'a>
    From<AnimatedRow<'a, N, Message, Theme, Renderer>> for Element<'a, Message, Theme, Renderer>
{
    fn from(value: AnimatedRow<'a, N, Message, Theme, Renderer>) -> Self {
        Element::new(value)
    }
}

#[macro_export]
macro_rules! animated_row {
    ($($x:expr),+ $(,)?) => {
        $crate::AnimatedRow::new([$(Element::from($x)),+])
    };
}
