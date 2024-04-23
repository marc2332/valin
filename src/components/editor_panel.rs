use super::icons::*;
use super::tab::*;
use crate::state::{AppState, Channel, Panel, PanelTab};
use crate::tabs::config::*;
use crate::tabs::editor::*;
use crate::tabs::welcome::*;
use crate::utils::*;
use dioxus_radio::prelude::use_radio;
use freya::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct EditorPanelProps {
    panel_index: usize,
    #[props(into)]
    width: String,
}

#[allow(non_snake_case)]
pub fn EditorPanel(EditorPanelProps { panel_index, width }: EditorPanelProps) -> Element {
    let mut radio_app_state = use_radio::<AppState, Channel>(Channel::Global);

    let app_state = radio_app_state.read();
    let panels_len = app_state.panels().len();
    let is_last_panel = app_state.panels().len() - 1 == panel_index;
    let is_focused = app_state.focused_panel() == panel_index;
    let panel = app_state.panel(panel_index);
    let active_tab_index = panel.active_tab();

    let close_panel = move |_: Option<MouseEvent>| {
        radio_app_state
            .write_channel(Channel::Global)
            .close_panel(panel_index);
    };

    let split_panel = move |_| {
        let len_panels = radio_app_state.read().panels().len();
        let mut app_state = radio_app_state.write_channel(Channel::Global);
        app_state.push_panel(Panel::new());
        app_state.set_focused_panel(len_panels - 1);
    };

    let onclickpanel = move |_| {
        let is_panel_focused = radio_app_state.read().focused_panel == panel_index;
        if !is_panel_focused {
            radio_app_state
                .write_channel(Channel::Global)
                .set_focused_panel(panel_index);
        }
    };

    let show_close_panel = panels_len > 1;
    let tabsbar_tools_width = if show_close_panel { 115 } else { 60 };
    let extra_container_width = if is_last_panel { 0 } else { 1 };

    rsx!(
        rect {
            direction: "horizontal",
            height: "100%",
            width: "{width}",
            rect {
                width: "calc(100% - {extra_container_width})",
                height: "100%",
                overflow: "clip",
                rect {
                    direction: "horizontal",
                    height: "40",
                    width: "100%",
                    cross_align: "center",
                    ScrollView {
                        direction: "horizontal",
                        theme: theme_with!(ScrollViewTheme {
                            width: format!("calc(100% - {tabsbar_tools_width})").into(),
                        }),
                        show_scrollbar: false,
                        {panel.tabs().iter().enumerate().map(|(editor_index, _)| {
                            let is_selected = active_tab_index == Some(editor_index);
                            rsx!(
                                PanelTab {
                                    panel_index,
                                    editor_index,
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
                        if show_close_panel {
                            Button {
                                theme: theme_with!(ButtonTheme {
                                    height: "100%".into(),
                                    padding: "10 8".into(),
                                }),
                                onclick: close_panel,
                                label {
                                    "Close"
                                }
                            }
                        }
                        Button {
                            theme: theme_with!(ButtonTheme {
                                height: "100%".into(),
                                padding: "10 8".into(),
                            }),
                            onclick: split_panel,
                            label {
                                "Split"
                            }
                        }
                    }
                }
                rect {
                    height: "calc(100% - 40)",
                    width: "100%",
                    onclick: onclickpanel,
                    if let Some(active_tab_index) = active_tab_index {
                        {
                            let active_tab = panel.tab(active_tab_index);
                            let tab_data = active_tab.get_data();
                            match active_tab {
                                PanelTab::TextEditor(editor) => {
                                    rsx!(
                                        EditorTab {
                                            key: "{tab_data.id}",
                                            panel_index,
                                            editor_index: active_tab_index,
                                            editor_type: editor.editor_type.clone()
                                        }
                                    )
                                }
                                PanelTab::Config => {
                                    rsx!(
                                        ConfigTab {
                                            key: "{tab_data.id}",
                                        }
                                    )
                                }
                                PanelTab::Welcome => {
                                    rsx!(
                                        WelcomeTab {
                                            key: "{tab_data.id}",
                                        }
                                    )
                                }
                            }
                        }
                    } else {
                        rect {
                            main_align: "center",
                            cross_align: "center",
                            width: "100%",
                            height: "100%",
                            background: "rgb(20, 20, 20)",
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
    editor_index: usize,
    is_selected: bool,
}

#[allow(non_snake_case)]
fn PanelTab(props: PanelTabProps) -> Element {
    let mut radio_app_state = use_radio::<AppState, Channel>(Channel::Tab {
        panel_index: props.panel_index,
        editor_index: props.editor_index,
    });

    let app_state = radio_app_state.read();
    let tab = app_state.panel(props.panel_index).tab(props.editor_index);
    let tab_data = tab.get_data();
    let is_selected = props.is_selected;

    let onclick = {
        move |_| {
            let mut app_state = radio_app_state.write_channel(Channel::Global);
            app_state.set_focused_panel(props.panel_index);
            app_state
                .panel_mut(props.panel_index)
                .set_active_tab(props.editor_index);
        }
    };

    let onclickaction = move |_| {
        if tab_data.edited {
            println!("save...")
        } else {
            radio_app_state
                .write_channel(Channel::Global)
                .close_tab(props.panel_index, props.editor_index);
        }
    };

    rsx!(Tab {
        key: "{tab_data.id}",
        onclick,
        onclickaction,
        value: "{tab_data.title}",
        is_edited: tab_data.edited,
        is_selected
    })
}
