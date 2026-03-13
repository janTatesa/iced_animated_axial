use std::array;

use iced::{
    Alignment, Element, Length, Subscription, Task, Theme,
    widget::{
        self, button, column, container,
        slider::{self},
        space,
    },
    window,
};
use iced_animated_axial::AnimatedRow;

fn main() {
    iced::application(
        || App {
            sliders: [5, 120, 120],
            layout: [5, 120, 120],
            animating: false,
        },
        App::update,
        App::view,
    )
    .subscription(|app| app.subscription())
    .run()
    .unwrap()
}

struct App {
    sliders: [u8; 3],
    layout: [u8; 3],
    animating: bool,
}

impl App {
    fn update(&mut self, msg: Message) -> Task<Message> {
        match msg {
            Message::ChangeSlider(slider, val) => {
                let is_smaller = self.sliders[slider] > val;
                let to_distribute = if is_smaller {
                    self.sliders[slider] - val
                } else {
                    val - self.sliders[slider]
                };
                self.sliders[slider] = val;

                let distribute_slider1 = to_distribute / 2 + to_distribute % 2;

                let distribute_slider2 = to_distribute / 2;
                let [slider1, slider2] = self
                    .sliders
                    .get_disjoint_mut([(slider + 1) % 3, (slider + 2) % 3])
                    .unwrap();
                (*slider1, *slider2) = if is_smaller {
                    (
                        slider1.saturating_add(distribute_slider1)
                            + distribute_slider2.saturating_sub(u8::MAX - *slider2),
                        slider2.saturating_add(distribute_slider2)
                            + distribute_slider1.saturating_sub(u8::MAX - *slider1),
                    )
                } else {
                    (
                        slider1
                            .saturating_sub(distribute_slider1)
                            .saturating_sub(distribute_slider2.saturating_sub(*slider2)),
                        slider2
                            .saturating_sub(distribute_slider2)
                            .saturating_sub(distribute_slider1.saturating_sub(*slider1)),
                    )
                }
            }
            Message::SubmitLayout => self.layout = self.sliders,
            Message::Animate => {}
            Message::StartAnimatingAxial => self.animating = true,
            Message::EndAnimatingAxial => self.animating = false,
        }

        Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        let children: [_; 3] = array::from_fn(|i| {
            let layout = self.layout[i] as u16;
            container(space().width(Length::Fill).height(10))
                .style([container::primary, container::danger, container::success][i])
                .width(if layout == 0 {
                    0.into()
                } else {
                    Length::FillPortion(layout)
                })
                .into()
        });
        let row = AnimatedRow::new(children)
            .width(Length::Fill)
            .spacing(5)
            .milliseconds_per_pixel(3.0)
            .on_animation_end(Message::EndAnimatingAxial)
            .on_animation_start(Message::StartAnimatingAxial);
        let content = column![
            "Layout",
            widget::slider(0..=255, self.sliders[0], |val| Message::ChangeSlider(
                0, val
            )),
            widget::slider(0..=255, self.sliders[1], |val| Message::ChangeSlider(
                1, val
            ))
            .style(|theme: &Theme, status| {
                let palette = theme.palette();

                let color = match status {
                    slider::Status::Active => palette.danger.base.color,
                    slider::Status::Hovered => palette.danger.strong.color,
                    slider::Status::Dragged => palette.danger.weak.color,
                };

                let mut style = slider::default(theme, status);
                style.rail.backgrounds.0 = color.into();
                style.handle.background = color.into();
                style
            }),
            widget::slider(0..=255, self.sliders[2], |val| Message::ChangeSlider(
                2, val
            ))
            .style(|theme: &Theme, status| {
                let palette = theme.palette();

                let color = match status {
                    slider::Status::Active => palette.success.base.color,
                    slider::Status::Hovered => palette.success.strong.color,
                    slider::Status::Dragged => palette.success.weak.color,
                };

                let mut style = slider::default(theme, status);
                style.rail.backgrounds.0 = color.into();
                style.handle.background = color.into();
                style
            })
            .width(Length::Fill),
            button("Submit").on_press(Message::SubmitLayout),
            row
        ]
        .align_x(Alignment::Center)
        .max_width(500)
        .spacing(5);
        container(content).center(Length::Fill).into()
    }

    fn subscription(&self) -> Subscription<Message> {
        if self.animating {
            window::frames().map(|_| Message::Animate)
        } else {
            Subscription::none()
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum Message {
    ChangeSlider(usize, u8),
    SubmitLayout,
    Animate,
    StartAnimatingAxial,
    EndAnimatingAxial,
}
