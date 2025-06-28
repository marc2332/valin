use super::icons::*;
use super::tab::*;
use crate::state::EditorView;
use crate::state::TabId;
use crate::state::{AppState, Channel, Panel};
use crate::utils::*;
use dioxus_radio::prelude::use_radio;
use freya::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct EditorPanelProps {
    panel_index: usize,
}

#[allow(non_snake_case)]
pub fn EditorPanel(EditorPanelProps { panel_index }: EditorPanelProps) -> Element {
    let mut radio_app_state = use_radio::<AppState, Channel>(Channel::AllTabs);

    let app_state = radio_app_state.read();
    let panels_len = app_state.panels().len();
    let is_last_panel = app_state.panels().len() - 1 == panel_index;
    let is_focused = app_state.focused_panel() == panel_index;
    let panel = app_state.panel(panel_index);
    let active_tab = panel.active_tab();

    let close_panel = move |_| {
        radio_app_state
            .write_channel(Channel::Global)
            .close_panel(panel_index);
    };

    let split_panel = move |_| {
        let mut app_state = radio_app_state.write_channel(Channel::Global);
        app_state.push_panel(Panel::new());
        app_state.focus_next_panel();
    };

    let onclickpanel = move |_| {
        let is_panel_focused = radio_app_state.read().focused_panel() == panel_index;
        let is_panels_view_focused = radio_app_state.read().focused_view() == EditorView::Panels;

        if !is_panel_focused {
            radio_app_state
                .write_channel(Channel::AllTabs)
                .focus_panel(panel_index);
        }

        if !is_panels_view_focused {
            radio_app_state
                .write_channel(Channel::Global)
                .focus_view(EditorView::Panels);
        }
    };

    let show_close_panel = panels_len > 1;
    let tabsbar_tools_width = if show_close_panel { 115 } else { 60 };
    let extra_container_width = if is_last_panel { 0 } else { 1 };

    rsx!(
        rect {
            direction: "horizontal",
            height: "100%",
            width: "fill",
            rect {
                width: "calc(100% - {extra_container_width})",
                height: "100%",
                overflow: "clip",
                rect {
                    direction: "horizontal",
                    height: "34",
                    width: "100%",
                    cross_align: "center",
                    ScrollView {
                        direction: "horizontal",
                        width: "calc(100% - {tabsbar_tools_width})",
                        show_scrollbar: false,
                        {panel.tabs.iter().map(|tab_id| {
                            let is_selected = active_tab == Some(*tab_id);
                            rsx!(
                                PanelTab {
                                    panel_index,
                                    tab_id: *tab_id,
                                    is_selected,
                                }
                            )
                        })}
                    }
                    rect {
                        width: "{tabsbar_tools_width}",
                        direction: "horizontal",
                        cross_align: "center",
                        main_align: "end",
                        height: "100%",
                        spacing: "4",
                        padding: "4",
                        if show_close_panel {
                            Button {
                                theme: theme_with!(ButtonTheme {
                                    height: "fill".into(),
                                    padding: "0 8".into(),
                                }),
                                onpress: close_panel,
                                label {
                                    "Close"
                                }
                            }
                        }
                        Button {
                            theme: theme_with!(ButtonTheme {
                                height: "fill".into(),
                                padding: "0 8".into(),
                            }),
                            onpress: split_panel,
                            label {
                                "Split"
                            }
                        }
                    }
                }
                rect {
                    height: "fill",
                    width: "100%",
                    onclick: onclickpanel,
                    if let Some(tab_id) = active_tab {
                        {
                            let active_tab = app_state.tab(&tab_id);
                            let Render = active_tab.render();
                            rsx!(
                                Render {
                                    key: "{tab_id:?}",
                                    tab_id,
                                }
                            )
                        }
                    } else {
                        rect {
                            main_align: "center",
                            cross_align: "center",
                            width: "100%",
                            height: "100%",
                            background: "rgb(17, 20, 21)",
                            ExpandedIcon {
                                Logo {
                                    enabled: is_focused,
                                    width: "200",
                                    height: "200"
                                }
                            }
                        }
                    }
                }
            }
            if !is_last_panel {
                Divider { }
            }
        }
    )
}

#[derive(Props, Clone, PartialEq)]
pub struct PanelTabProps {
    panel_index: usize,
    tab_id: TabId,
    is_selected: bool,
}

#[allow(non_snake_case)]
fn PanelTab(
    PanelTabProps {
        panel_index,
        tab_id,
        is_selected,
    }: PanelTabProps,
) -> Element {
    let mut radio_app_state = use_radio::<AppState, Channel>(Channel::follow_tab(tab_id));

    let app_state = radio_app_state.read();
    let tab = app_state.tab(&tab_id);
    let tab_data = tab.get_data();

    let onclick = move |_| {
        let mut app_state = radio_app_state.write_channel(Channel::Global);
        app_state.focus_panel(panel_index);
        app_state.panel_mut(panel_index).set_active_tab(tab_id);
    };

    let onclickaction = move |_| {
        if tab_data.edited {
            println!("save...")
        } else {
            radio_app_state
                .write_channel(Channel::Global)
                .close_tab(tab_id);
        }
    };

    rsx!(EditorTab {
        onclick,
        onclickaction,
        value: tab_data.title,
        is_edited: tab_data.edited,
        is_selected
    })
}
