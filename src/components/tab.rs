use freya::prelude::*;

#[derive(Default, PartialEq)]
pub enum ButtonStatus {
    Hovering,
    #[default]
    Idle,
}

#[derive(Clone, PartialEq)]
pub struct EditorTab {
    pub value: String,
    pub on_press: EventHandler<Event<PressEventData>>,
    pub on_click_indicator: EventHandler<()>,
    pub is_selected: bool,
    pub is_edited: bool,
}

impl Component for EditorTab {
    fn render(&self) -> impl IntoElement {
        let mut status = use_state(ButtonStatus::default);

        use_drop(move || {
            if *status.read() == ButtonStatus::Hovering {
                Cursor::set(CursorIcon::default());
            }
        });

        let on_pointer_enter = move |_| {
            status.set(ButtonStatus::Hovering);
        };

        let on_pointer_leave = move |_| {
            status.set(ButtonStatus::default());
        };

        let background = match *status.read() {
            _ if self.is_selected => (29, 32, 33).into(),
            ButtonStatus::Hovering => (25, 28, 29).into(),
            ButtonStatus::Idle => Color::TRANSPARENT,
        };
        let selected_color = if self.is_selected {
            (60, 60, 60).into()
        } else {
            background
        };
        let is_hovering = *status.read() == ButtonStatus::Hovering;

        let on_pressaction = self.on_click_indicator.clone();

        rect()
            .width(Size::px(140.0))
            .height(Size::fill())
            .child(
                rect()
                    .height(Size::px(2.))
                    .width(Size::fill())
                    .background(selected_color),
            )
            .child(
                rect()
                    .background(background)
                    .on_press(self.on_press.clone())
                    .on_pointer_enter(on_pointer_enter)
                    .on_pointer_leave(on_pointer_leave)
                    .expanded()
                    .cross_align(Alignment::Center)
                    .horizontal()
                    .padding(Gaps::new(0., 0., 0., 10.))
                    .child(
                        label()
                            .width(Size::func(|c| Some(c.available_parent - 28.)))
                            .max_lines(1)
                            .text_overflow(TextOverflow::Ellipsis)
                            .text(self.value.clone()),
                    )
                    .child(
                        rect()
                            .width(Size::px(24.0))
                            .height(Size::fill())
                            .center()
                            .on_press(move |e: Event<PressEventData>| {
                                e.stop_propagation();
                                e.prevent_default();
                                on_pressaction.call(());
                            })
                            .maybe_child(if self.is_edited {
                                Some(
                                    rect()
                                        .center()
                                        .expanded()
                                        .child(
                                            rect()
                                                .background((180, 180, 180))
                                                .width(Size::px(10.0))
                                                .height(Size::px(10.0))
                                                .corner_radius(CornerRadius::new_all(100.0)),
                                        )
                                        .into_element(),
                                )
                            } else if is_hovering || self.is_selected {
                                Some(
                                    Button::new()
                                        .flat()
                                        .padding(4.)
                                        .rounded()
                                        .child(CrossIcon {
                                            fill: (150, 150, 150).into(),
                                        })
                                        .into_element(),
                                )
                            } else {
                                None
                            }),
                    ),
            )
    }
}

#[derive(Clone, PartialEq)]
pub struct CrossIcon {
    pub fill: Color,
}

impl Component for CrossIcon {
    fn render(&self) -> impl IntoElement {
        svg(Bytes::from_static(
            r#"<svg viewBox="0 0 16 16" fill="none" xmlns="http://www.w3.org/2000/svg">
                <path d="M12.8536 2.85355C13.0488 2.65829 13.0488 2.34171 12.8536 2.14645C12.6583 1.95118 12.3417 1.95118 12.1464 2.14645L7.5 6.79289L2.85355 2.14645C2.65829 1.95118 2.34171 1.95118 2.14645 2.14645C1.95118 2.34171 1.95118 2.65829 2.14645 2.85355L6.79289 7.5L2.14645 12.1464C1.95118 12.3417 1.95118 12.6583 2.14645 12.8536C2.34171 13.0488 2.65829 13.0488 2.85355 12.8536L7.5 8.20711L12.1464 12.8536C12.3417 13.0488 12.6583 13.0488 12.8536 12.8536C13.0488 12.6583 13.0488 12.3417 12.8536 12.1464L8.20711 7.5L12.8536 2.85355Z"/>
            </svg>"#
                .as_bytes(),
        ))
        .width(Size::px(15.0))
        .height(Size::px(15.0))
        .fill(self.fill)
    }
}
