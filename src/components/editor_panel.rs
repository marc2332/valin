use super::{CrossIcon, Logo};
use crate::state::{AppState, Channel, EditorView, PanelId, TabId};
use freya::prelude::*;
use freya::radio::use_radio;

/// Content shown in a panel with no tabs.
#[derive(Clone, PartialEq)]
pub struct EmptyPanel {
    pub panel_id: PanelId,
}

impl Component for EmptyPanel {
    fn render(&self) -> impl IntoElement {
        let panel_id = self.panel_id;
        let mut radio = use_radio::<AppState, Channel>(Channel::Global);
        let is_focused = radio.read().focused_panel == Some(panel_id);

        rect()
            .expanded()
            .center()
            .background((8, 8, 12))
            .on_pointer_down(move |_| {
                if radio.read().focused_panel == Some(panel_id) {
                    return;
                }
                let mut state = radio.write_channel(Channel::Global);
                state.focused_panel = Some(panel_id);
                if state.focused_view != EditorView::Panels {
                    state.focused_view = EditorView::Panels;
                }
            })
            .child(Logo {
                enabled: is_focused,
                width: 200.,
                height: 200.,
            })
    }
}

/// Visual tab-header button rendered inside the docking tab bar.
#[derive(Clone, PartialEq)]
pub struct EditorTabButton {
    pub tab_id: TabId,
    pub value: String,
    pub on_close: EventHandler<()>,
    pub is_selected: bool,
    pub icon: Option<Bytes>,
}

impl Component for EditorTabButton {
    fn render(&self) -> impl IntoElement {
        let mut is_hovering = use_state(|| false);

        let radio = use_radio::<AppState, Channel>(Channel::follow_tab(self.tab_id));
        let is_edited = radio.read().tab(&self.tab_id).get_data().edited;

        let background = match (*is_hovering.read(), self.is_selected) {
            (_, true) | (true, _) => (13, 17, 23).into(),
            _ => Color::TRANSPARENT,
        };
        let selected_color: Color = if self.is_selected {
            (247, 129, 102).into()
        } else {
            background
        };

        let on_close = self.on_close.clone();

        rect()
            .width(Size::px(140.0))
            .height(Size::fill())
            .on_pointer_over(move |_| is_hovering.set(true))
            .on_pointer_out(move |_| is_hovering.set(false))
            .child(
                rect()
                    .height(Size::px(2.))
                    .width(Size::fill())
                    .background(selected_color),
            )
            .child(
                rect()
                    .background(background)
                    .expanded()
                    .cross_align(Alignment::Center)
                    .horizontal()
                    .padding((0., 0., 0., 10.))
                    .maybe_child(self.icon.clone().map(|icon_bytes| {
                        svg(icon_bytes)
                            .width(Size::px(14.0))
                            .height(Size::px(14.0))
                            .fill(Color::from_rgb(180, 180, 180))
                            .margin((0., 4., 0., 0.))
                    }))
                    .child(
                        label()
                            .width(Size::func(|c| Some(c.available_parent - 28.)))
                            .max_lines(1)
                            .font_size(13.)
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
                                on_close.call(());
                            })
                            .maybe_child(if is_edited {
                                Some(
                                    rect()
                                        .center()
                                        .expanded()
                                        .child(
                                            rect()
                                                .background((125, 133, 144))
                                                .width(Size::px(10.0))
                                                .height(Size::px(10.0))
                                                .corner_radius(CornerRadius::new_all(100.0)),
                                        )
                                        .into_element(),
                                )
                            } else if *is_hovering.read() || self.is_selected {
                                Some(
                                    Button::new()
                                        .flat()
                                        .padding(4.)
                                        .rounded()
                                        .child(CrossIcon {
                                            fill: (125, 133, 144).into(),
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
