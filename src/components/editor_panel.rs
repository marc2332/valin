use crate::components::EditorTab;
use crate::components::Logo;
use crate::state::EditorView;
use crate::state::TabId;
use crate::state::TabProps;
use crate::state::{AppState, Channel, Panel};
use freya::helpers::from_fn;
use freya::prelude::*;
use freya::radio::use_radio;

#[derive(Clone, PartialEq)]
pub struct EditorPanel {
    pub panel_index: usize,
}

impl Component for EditorPanel {
    fn render_key(&self) -> DiffKey {
        DiffKey::from(&self.panel_index)
    }
    fn render(&self) -> impl IntoElement {
        let panel_index = self.panel_index;
        let mut radio_app_state = use_radio::<AppState, Channel>(Channel::AllTabs);

        let app_state = radio_app_state.read();
        let panels_len = app_state.panels.len();
        let is_last_panel = panels_len - 1 == panel_index;
        let is_focused = app_state.focused_panel == panel_index;
        let panel = &app_state.panels[panel_index];
        let active_tab = panel.active_tab;

        let show_close_panel = panels_len > 1;
        let extra_container_width = if is_last_panel { 0.0 } else { 1.0 };

        let close_panel = move |e: Event<PressEventData>| {
            e.stop_propagation();
            e.prevent_default();
            radio_app_state
                .write_channel(Channel::Global)
                .close_panel(panel_index);
        };

        let split_panel = move |_| {
            let mut app_state = radio_app_state.write_channel(Channel::Global);
            app_state.push_panel(Panel::default());
            app_state.focus_next_panel();
        };

        let on_presspanel = move |_| {
            let is_panel_focused = radio_app_state.read().focused_panel == panel_index;
            let is_panels_view_focused = radio_app_state.read().focused_view == EditorView::Panels;

            if !is_panel_focused {
                radio_app_state
                    .write_channel(Channel::AllTabs)
                    .focused_panel = panel_index;
            }

            if !is_panels_view_focused {
                radio_app_state
                    .write_channel(Channel::Global)
                    .focus_view(EditorView::Panels);
            }
        };

        rect().horizontal().expanded().child(
            rect()
                .width(Size::func(move |ctx| {
                    Some(ctx.parent - extra_container_width)
                }))
                .height(Size::fill())
                .overflow(Overflow::Clip)
                .child(
                    rect()
                        .horizontal()
                        .height(Size::px(32.0))
                        .width(Size::fill())
                        .cross_align(Alignment::Center)
                        .content(Content::Flex)
                        .child(
                            ScrollView::new()
                                .direction(Direction::Horizontal)
                                .width(Size::flex(1.))
                                .show_scrollbar(false)
                                .children(panel.tabs.iter().map(|tab_id| {
                                    let is_selected = active_tab == Some(*tab_id);
                                    PanelTab {
                                        panel_index,
                                        tab_id: *tab_id,
                                        is_selected,
                                    }
                                    .into()
                                })),
                        )
                        .child(
                            rect()
                                .horizontal()
                                .cross_align(Alignment::Center)
                                .main_align(Alignment::End)
                                .height(Size::fill())
                                .spacing(4.0)
                                .padding(4.0)
                                .maybe_child(if show_close_panel {
                                    Some(
                                        Button::new()
                                            .flat()
                                            .height(Size::fill())
                                            .padding((0., 8.))
                                            .on_press(close_panel)
                                            .child(
                                                svg(freya::icons::lucide::x())
                                                    .width(Size::px(16.0))
                                                    .height(Size::px(16.0))
                                                    .color((200, 200, 200)),
                                            ),
                                    )
                                } else {
                                    None
                                })
                                .child(
                                    Button::new()
                                        .flat()
                                        .height(Size::fill())
                                        .padding((0., 8.))
                                        .on_press(split_panel)
                                        .child(
                                            svg(freya::icons::lucide::columns_2())
                                                .width(Size::px(20.0))
                                                .height(Size::px(20.0))
                                                .color((200, 200, 200)),
                                        ),
                                ),
                        ),
                )
                .child(
                    rect()
                        .expanded()
                        .on_mouse_down(on_presspanel)
                        .background((8, 8, 12))
                        .child(if let Some(tab_id) = active_tab {
                            let active_tab = app_state.tab(&tab_id);
                            let render = active_tab.render();
                            from_fn(tab_id, TabProps { tab_id }, render)
                        } else {
                            rect()
                                .expanded()
                                .center()
                                .child(Logo {
                                    enabled: is_focused,
                                    width: 200.,
                                    height: 200.,
                                })
                                .into()
                        }),
                ),
        )
    }
}

#[derive(Clone, PartialEq)]
pub struct PanelTab {
    pub panel_index: usize,
    pub tab_id: TabId,
    pub is_selected: bool,
}

impl Component for PanelTab {
    fn render(&self) -> impl IntoElement {
        let panel_index = self.panel_index;
        let tab_id = self.tab_id;
        let is_selected = self.is_selected;

        let mut radio_app_state = use_radio::<AppState, Channel>(Channel::follow_tab(tab_id));
        let app_state = radio_app_state.read();
        let tab = app_state.tab(&tab_id);
        let tab_data = tab.get_data();

        let on_press = move |_| {
            let mut app_state = radio_app_state.write_channel(Channel::Global);
            app_state.focused_panel = panel_index;
            app_state.panels[panel_index].active_tab = Some(tab_id);
        };

        let on_pressaction = move |_| {
            if tab_data.edited {
                // Save logic here if needed
            } else {
                radio_app_state
                    .write_channel(Channel::Global)
                    .close_tab(tab_id);
            }
        };

        EditorTab {
            on_press: on_press.into(),
            on_click_indicator: on_pressaction.into(),
            value: tab_data.title,
            is_edited: tab_data.edited,
            is_selected,
            icon: tab_data.icon,
        }
    }
}
