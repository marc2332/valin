use super::icons::*;
use super::tab::*;
use crate::hooks::*;
use crate::tabs::config::*;
use crate::tabs::editor::*;
use crate::utils::*;
use freya::prelude::*;

#[derive(Props, PartialEq)]
pub struct EditorPanelProps {
    panel_index: usize,
    #[props(into)]
    width: String,
}

#[allow(non_snake_case)]
pub fn EditorPanel(cx: Scope<EditorPanelProps>) -> Element {
    let EditorPanelProps { panel_index, width } = cx.props;
    let manager = use_manager(cx, SubscriptionModel::All);

    let panels_len = manager.current().panels().len();
    let is_last_panel = manager.current().panels().len() - 1 == *panel_index;
    let is_focused = manager.current().focused_panel() == *panel_index;
    let current_manager = manager.current();
    let panel = current_manager.panel(*panel_index);
    let active_tab_index = panel.active_tab();

    let close_panel = {
        to_owned![manager];
        move |_: MouseEvent| {
            manager.global_write().close_panel(*panel_index);
        }
    };

    let split_panel = {
        to_owned![manager];
        move |_| {
            let len_panels = manager.current().panels().len();
            let mut manager = manager.global_write();
            manager.push_panel(Panel::new());
            manager.set_focused_panel(len_panels - 1);
        }
    };

    let onclickpanel = {
        to_owned![manager];
        move |_| {
            manager.global_write().set_focused_panel(*panel_index);
        }
    };

    let show_close_panel = panels_len > 1;
    let tabsbar_tools_width = if show_close_panel { 125 } else { 60 };

    render!(
        rect {
            direction: "horizontal",
            height: "100%",
            width: "{width}",
            rect {
                width: "calc(100% - 2)",
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
                            width: "calc(100% - {tabsbar_tools_width})".into(),
                        }),
                        show_scrollbar: false,
                        panel.tabs().iter().enumerate().map(|(editor_index, _)| {
                            let is_selected = active_tab_index == Some(editor_index);
                            rsx!(
                                PanelTab {
                                    panel_index: *panel_index,
                                    editor_index: editor_index,
                                    is_selected: is_selected,
                                }
                            )
                        })
                    }
                    rect {
                        width: "{tabsbar_tools_width}",
                        direction: "horizontal",
                        cross_align: "center",
                        height: "100%",
                        if show_close_panel {
                            rsx!(
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
                            )
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
                        let active_tab = panel.tab(active_tab_index);
                        let tab_data = active_tab.get_data();
                        match active_tab {
                            PanelTab::TextEditor(editor) => {
                                rsx!(
                                    EditorTab {
                                        key: "{tab_data.id}",
                                        panel_index: *panel_index,
                                        editor_index: active_tab_index,
                                        language_id: editor.language_id,
                                        root_path: editor.root_path.clone()
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
                        }
                    } else {
                        rsx!(
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
                        )
                    }
                }
            }
            if !is_last_panel {
                rsx!(
                    Divider {

                    }
                )
            }
        }
    )
}

#[derive(Props, PartialEq)]
pub struct PanelTabProps {
    panel_index: usize,
    editor_index: usize,
    is_selected: bool,
}

#[allow(non_snake_case)]
fn PanelTab(cx: Scope<PanelTabProps>) -> Element {
    let manager = use_manager(
        cx,
        SubscriptionModel::Tab {
            panel_index: cx.props.panel_index,
            editor_index: cx.props.editor_index,
        },
    );

    let manager_ref = manager.current();
    let tab = manager_ref
        .panel(cx.props.panel_index)
        .tab(cx.props.editor_index);
    let tab_data = tab.get_data();
    let is_selected = cx.props.is_selected;

    let onclick = {
        to_owned![manager];
        move |_| {
            let mut manager = manager.global_write();
            manager.set_focused_panel(cx.props.panel_index);
            manager
                .panel_mut(cx.props.panel_index)
                .set_active_tab(cx.props.editor_index);
        }
    };

    let onclickaction = {
        to_owned![manager];
        move |_| {
            if tab_data.edited {
                println!("save...")
            } else {
                manager
                    .global_write()
                    .close_editor(cx.props.panel_index, cx.props.editor_index);
            }
        }
    };

    render!(Tab {
        key: "{tab_data.id}",
        onclick: onclick,
        onclickaction: onclickaction,
        value: "{tab_data.title}",
        is_edited: tab_data.edited,
        is_selected: is_selected
    })
}
