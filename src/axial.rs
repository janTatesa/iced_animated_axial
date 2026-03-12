use std::{
    array, mem,
    time::{Duration, Instant},
};

use iced_core::{
    Alignment, Animation, Element, Event, Layout, Length, Padding, Rectangle, Shell, Size, Vector,
    Widget,
    animation::{Easing, Interpolable},
    layout::{self, flex::Axis},
    mouse, overlay,
    renderer::Style,
    widget::{
        Operation, Tree,
        tree::{self, Tag},
    },
    window,
};

pub struct AnimatedAxial<
    'a,
    const N: usize,
    Message: Clone,
    Theme = iced_core::Theme,
    Renderer = iced_renderer::Renderer,
> {
    pub(crate) children: [Element<'a, Message, Theme, Renderer>; N],
    pub(crate) spacing: f32,
    pub(crate) padding: Padding,
    pub(crate) width: Length,
    pub(crate) height: Length,
    pub(crate) max_length: Option<f32>,
    pub(crate) align: Alignment,
    pub(crate) clip: bool,
    pub(crate) milliseconds_per_pixel: f32,
    pub(crate) easing: Easing,
    pub(crate) on_animation_start: Option<Message>,
    pub(crate) on_animation_end: Option<Message>,
    pub(crate) axis: Axis,
}

struct State<const N: usize> {
    layout_state: LayoutState<N>,
    was_transitioning_previous_update: bool,
    update_instant: Instant,
}

#[derive(Default, Clone, Debug)]
enum LayoutState<const N: usize> {
    #[default]
    Unspecified,
    Transition {
        animation: Animation<f32>,
        last_returned: AxialLayout<N>,
        start: AxialLayout<N>,
        target: AxialLayout<N>,
    },
    Reached(AxialLayout<N>),
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub(crate) struct AxialLayout<const N: usize>([f32; N]);

impl<const N: usize> Interpolable for AxialLayout<N> {
    fn interpolated(&self, other: Self, ratio: f32) -> Self {
        Self(array::from_fn(|i| {
            self.0[i].interpolated(other.0[i], ratio)
        }))
    }
}

#[allow(private_bounds)]
impl<'a, const N: usize, Message: Clone, Theme, Renderer: iced_core::Renderer>
    AnimatedAxial<'a, N, Message, Theme, Renderer>
{
    fn transition_duration(&mut self, start: AxialLayout<N>, target: AxialLayout<N>) -> Duration {
        let pixels = start
            .0
            .into_iter()
            .enumerate()
            .map(|(i, size)| (size - target.0[i]).abs())
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(0.0);
        Duration::from_secs_f32(self.milliseconds_per_pixel * pixels / 1000.0)
    }
}

impl<const N: usize, Message: Clone, Theme, Renderer> Widget<Message, Theme, Renderer>
    for AnimatedAxial<'_, N, Message, Theme, Renderer>
where
    Renderer: iced_core::Renderer,
{
    fn children(&self) -> Vec<Tree> {
        self.children.iter().map(Tree::new).collect()
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(&self.children);
    }

    fn size(&self) -> Size<Length> {
        Size {
            width: self.width,
            height: self.height,
        }
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let state: &mut State<N> = tree.state.downcast_mut();

        let limits = self
            .max_length
            .map_or(*limits, |max_length| match self.axis {
                Axis::Horizontal => limits.max_height(max_length),
                Axis::Vertical => limits.max_width(max_length),
            });

        let orig_layout = layout::flex::resolve(
            match self.axis {
                Axis::Horizontal => Axis::Horizontal,
                Axis::Vertical => Axis::Vertical,
            },
            renderer,
            &limits,
            self.width,
            self.height,
            self.padding,
            self.spacing,
            self.align,
            &mut self.children,
            &mut tree.children,
        );

        let orig_layout_axial = AxialLayout(array::from_fn(|i| match self.axis {
            Axis::Horizontal => orig_layout.children()[i].size().width,
            Axis::Vertical => orig_layout.children()[i].size().height,
        }));
        let instant = state.update_instant;
        state.layout_state = match mem::take(&mut state.layout_state) {
            LayoutState::Transition {
                target,
                last_returned,
                ..
            }
            | LayoutState::Reached(last_returned @ target)
                if target != orig_layout_axial =>
            {
                LayoutState::Transition {
                    animation: Animation::new(0.0)
                        .go(1.0, instant)
                        .easing(self.easing)
                        .duration(self.transition_duration(last_returned, orig_layout_axial)),
                    last_returned,
                    start: last_returned,
                    target: orig_layout_axial,
                }
            }
            LayoutState::Transition {
                animation, target, ..
            } if !animation.is_animating(instant) => LayoutState::Reached(target),
            LayoutState::Transition {
                animation,
                start,
                target,
                last_returned,
            } => {
                let animation = animation
                    .easing(self.easing)
                    .duration(self.transition_duration(start, target));
                LayoutState::Transition {
                    animation,
                    last_returned,
                    start,
                    target,
                }
            }
            LayoutState::Reached(axial_layout) => LayoutState::Reached(axial_layout),
            LayoutState::Unspecified => LayoutState::Reached(orig_layout_axial),
        };

        if let LayoutState::Transition {
            animation,
            start,
            target,
            last_returned,
        } = &mut state.layout_state
        {
            let new_layout =
                animation.interpolate_with(|ratio| start.interpolated(*target, ratio), instant);
            *last_returned = new_layout;

            let mut children: Vec<_> = self
                .children
                .iter_mut()
                .zip(new_layout.0)
                .map(|(child, len)| {
                    Element::new(DummyWidget {
                        inner: child.as_widget_mut(),
                        len,
                        axis: &self.axis,
                    })
                })
                .collect();

            layout::flex::resolve(
                match self.axis {
                    Axis::Horizontal => Axis::Horizontal,
                    Axis::Vertical => Axis::Vertical,
                },
                renderer,
                &limits,
                self.width,
                self.height,
                self.padding,
                self.spacing,
                self.align,
                &mut children,
                &mut tree.children,
            )
        } else {
            orig_layout
        }
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn Operation,
    ) {
        operation.container(None, layout.bounds());
        operation.traverse(&mut |operation| {
            self.children
                .iter_mut()
                .zip(&mut tree.children)
                .zip(layout.children())
                .for_each(|((child, state), layout)| {
                    child
                        .as_widget_mut()
                        .operate(state, layout, renderer, operation);
                });
        });
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
        let state: &mut State<N> = tree.state.downcast_mut();
        let is_transitioning = matches!(state.layout_state, LayoutState::Transition { .. });
        if let Some(msg) = match (is_transitioning, state.was_transitioning_previous_update) {
            (true, false) => self.on_animation_start.clone(),
            (false, true) => self.on_animation_end.clone(),
            _ => None,
        } {
            shell.publish(msg);
        }

        state.was_transitioning_previous_update = is_transitioning;
        if let Event::Window(window::Event::RedrawRequested(instant)) = event {
            state.update_instant = *instant;
        }

        for ((child, tree), layout) in self
            .children
            .iter_mut()
            .zip(&mut tree.children)
            .zip(layout.children())
        {
            child
                .as_widget_mut()
                .update(tree, event, layout, cursor, renderer, shell, viewport);
        }
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.children
            .iter()
            .zip(&tree.children)
            .zip(layout.children())
            .map(|((child, tree), layout)| {
                child
                    .as_widget()
                    .mouse_interaction(tree, layout, cursor, viewport, renderer)
            })
            .max()
            .unwrap_or_default()
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        if let Some(clipped_viewport) = layout.bounds().intersection(viewport) {
            let viewport = if self.clip {
                &clipped_viewport
            } else {
                viewport
            };

            for ((child, tree), layout) in self
                .children
                .iter()
                .zip(&tree.children)
                .zip(layout.children())
                .filter(|(_, layout)| layout.bounds().intersects(viewport))
            {
                child
                    .as_widget()
                    .draw(tree, renderer, theme, style, layout, cursor, viewport);
            }
        }
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        overlay::from_children(
            &mut self.children,
            tree,
            layout,
            renderer,
            viewport,
            translation,
        )
    }

    fn tag(&self) -> Tag {
        Tag::of::<State<N>>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::<N> {
            layout_state: LayoutState::Unspecified,
            was_transitioning_previous_update: false,
            update_instant: Instant::now(),
        })
    }
}

impl<'a, const N: usize, Message: Clone + 'a, Theme: Clone + 'a, Renderer: iced_core::Renderer + 'a>
    From<AnimatedAxial<'a, N, Message, Theme, Renderer>> for Element<'a, Message, Theme, Renderer>
{
    fn from(value: AnimatedAxial<'a, N, Message, Theme, Renderer>) -> Self {
        Element::new(value)
    }
}

struct DummyWidget<'a, Message, Theme, Renderer> {
    inner: &'a mut dyn Widget<Message, Theme, Renderer>,
    len: f32,
    axis: &'a Axis,
}

impl<'a, Message, Theme, Renderer: iced_core::Renderer> Widget<Message, Theme, Renderer>
    for DummyWidget<'a, Message, Theme, Renderer>
{
    fn size(&self) -> Size<Length> {
        match self.axis {
            Axis::Horizontal => Size {
                width: Length::Fixed(self.len),
                height: self.inner.size().height,
            },
            Axis::Vertical => Size {
                height: Length::Fixed(self.len),
                width: self.inner.size().width,
            },
        }
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        self.inner.layout(
            tree,
            renderer,
            &match self.axis {
                Axis::Horizontal => limits.max_width(self.len).min_width(self.len),
                Axis::Vertical => limits.max_height(self.len).max_height(self.len),
            },
        )
    }

    fn draw(
        &self,
        _tree: &Tree,
        _renderer: &mut Renderer,
        _theme: &Theme,
        _style: &iced_core::renderer::Style,
        _layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _viewport: &Rectangle,
    ) {
        unimplemented!()
    }
}
