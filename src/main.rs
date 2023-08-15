mod commander;
mod controlled_virtual_scroll_view;
mod file_explorer;
mod icons;
mod lsp;
mod manager;
mod parser;
mod sidebar;
mod sidepanel;
mod tab;
mod tabs;
mod text_area;
mod use_debouncer;
mod use_editable;
mod use_metrics;
mod utils;

use std::collections::HashMap;

use commander::*;
use file_explorer::*;
use freya::prelude::{keyboard::Code, *};
use futures::StreamExt;
use icons::*;
use manager::*;
use sidebar::*;
use sidepanel::*;
use tab::*;
use tabs::code_editor::*;
use tabs::config::*;
use text_area::*;
use utils::*;

fn main() {
    launch_cfg(
        app,
        LaunchConfig::<()>::builder()
            .with_width(900.0)
            .with_height(600.0)
            .with_title("Editor")
            .build(),
    );
}

fn app(cx: Scope) -> Element {
    use_init_focus(cx);
    render!(
        ThemeProvider { theme: DARK_THEME, Body {} }
    )
}

#[allow(non_snake_case)]
fn Body(cx: Scope) -> Element {
    let lsp_messages = use_state::<HashMap<String, String>>(cx, HashMap::default);
    let lsp_status_coroutine = use_coroutine(cx, |mut rx: UnboundedReceiver<(String, String)>| {
        to_owned![lsp_messages];
        async move {
            while let Some((name, val)) = rx.next().await {
                lsp_messages.with_mut(|msgs| {
                    msgs.insert(name, val);
                })
            }
        }
    });
    let manager = use_init_manager(cx, lsp_status_coroutine);
    let show_commander = use_state(cx, || false);

    // Commands
    let commands = cx.use_hook(|| {
        vec![Command::new("fs".to_string(), {
            to_owned![manager];
            Box::new(move |size: &str| {
                if let Ok(size) = size.parse::<f32>() {
                    manager.global_write().set_fontsize(size);
                }
            })
        })]
    });

    let onsubmit = {
        to_owned![manager];
        move |_| {
            let mut manager = manager.global_write();
            manager.set_focused(true);
            show_commander.set(false);
        }
    };

    let onkeydown = {
        to_owned![manager];
        move |e: KeyboardEvent| {
            let mut manager = manager.global_write();
            if e.code == Code::Escape {
                if *show_commander.current() {
                    manager.set_focused(true);
                    show_commander.set(false);
                } else {
                    manager.set_focused(false);
                    show_commander.set(true);
                }
            }
        }
    };

    let onglobalclick = |_| {
        show_commander.set(false);
    };

    let panels_len = manager.current().panels().len();
    let panes_width = 100.0 / panels_len as f32;

    let cursor = {
        let manager = manager.current();
        let panel = manager.panel(manager.focused_panel);
        if let Some(active_tab) = panel.active_tab() {
            panel
                .tab(active_tab)
                .as_text_editor()
                .map(|editor| editor.cursor())
        } else {
            None
        }
    };

    render!(
        rect {
            color: "white",
            background: "rgb(20, 20, 20)",
            width: "100%",
            height: "100%",
            onkeydown: onkeydown,
            onglobalclick: onglobalclick,
            rect {
                height: "calc(100% - 32)",
                direction: "horizontal",
                Sidebar {}
                Divider {}
                Sidepanel {
                    FileExplorer {}
                }
                Divider {}
                rect {
                    direction: "vertical",
                    width: "calc(100% - 334)",
                    height: "100%",
                    if *show_commander.current(){
                        rsx!(
                            Commander {
                                onsubmit: onsubmit,
                                commands: commands
                            }
                        )
                    }
                    rect {
                        height: "100%",
                        width: "100%",
                        direction: "horizontal",
                        manager.current().panels().iter().enumerate().map(|(panel_index, panel)| {
                            let is_focused = manager.current().focused_panel() == panel_index;
                            let active_tab_index = panel.active_tab();
                            let close_panel = {
                                to_owned![manager];
                                move |_: MouseEvent| {
                                    manager.global_write().close_panel(panel_index);
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
                                    manager.global_write().set_focused_panel(panel_index);
                                }
                            };

                            let show_close_panel = panels_len > 1;
                            let tabsbar_scroll_offset = if show_close_panel {
                                115
                            } else {
                                55
                            };

                            rsx!(
                                rect {
                                    direction: "vertical",
                                    height: "100%",
                                    width: "{panes_width}%",
                                    overflow: "clip",
                                    rect {
                                        direction: "horizontal",
                                        height: "50",
                                        width: "100%",
                                        padding: "2.5",
                                        ScrollView {
                                            direction: "horizontal",
                                            width: "calc(100% - {tabsbar_scroll_offset})",
                                            panel.tabs().iter().enumerate().map(|(i, tab)| {
                                                let is_selected = active_tab_index == Some(i);
                                                let (tab_id, tab_title) = tab.get_data();

                                                let onclick = {
                                                    to_owned![manager];
                                                    move |_| {
                                                        let mut manager = manager.global_write();
                                                        manager.set_focused_panel(panel_index);
                                                        manager.panel_mut(panel_index).set_active_tab(i);
                                                    }
                                                };

                                                let onclickclose = {
                                                    to_owned![manager];
                                                    move |_| {
                                                        manager.global_write().close_editor(panel_index, i);
                                                    }
                                                };

                                                rsx!(
                                                    Tab {
                                                        key: "{tab_id}",
                                                        onclick: onclick,
                                                        onclickclose: onclickclose,
                                                        value: "{tab_title}",
                                                        is_selected: is_selected
                                                    }
                                                )
                                            })
                                        }
                                        Button {
                                            onclick: split_panel,
                                            label {
                                                "Split"
                                            }
                                        }
                                        if show_close_panel {
                                            rsx!(
                                                Button {
                                                    onclick: close_panel,
                                                    label {
                                                        "Close"
                                                    }
                                                }
                                            )
                                        }
                                    }
                                    rect {
                                        height: "calc(100%-50)",
                                        width: "100%",
                                        onclick: onclickpanel,
                                        if let Some(active_tab_index) = active_tab_index {
                                            let active_tab = panel.tab(active_tab_index);
                                            let (tab_id, _) = active_tab.get_data();
                                            match active_tab {
                                                PanelTab::TextEditor(editor) => {
                                                    rsx!(
                                                        CodeEditorTab {
                                                            key: "{tab_id}",
                                                            panel_index: panel_index,
                                                            editor: active_tab_index,
                                                            language_id: editor.language_id,
                                                            root_path: editor.root_path.clone()
                                                        }
                                                    )
                                                }
                                                PanelTab::Config => {
                                                    rsx!(
                                                        ConfigTab {
                                                            key: "{tab_id}",
                                                        }
                                                    )
                                                }
                                            }
                                        } else {
                                            rsx!(
                                                rect {
                                                    display: "center",
                                                    width: "100%",
                                                    height: "100%",
                                                    direction: "both",
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
                                Divider {

                                }
                            )
                        })
                    }
                }
            }
            VerticalDivider {}
            rect {
                width: "100%",
                height: "30",
                background: "rgb(20, 20, 20)",
                direction: "horizontal",
                padding: "5",
                color: "rgb(200, 200, 200)",
                if let Some(cursor) = cursor {
                    rsx!(
                        label {
                            "Ln {cursor.row() + 1}, Col {cursor.col() + 1}"
                        }
                    )
                }
                for (name, msg) in lsp_messages.get() {
                    rsx!(
                        label {
                            "  {name} {msg}"
                        }
                    )
                }
            }
        }
    )
}
