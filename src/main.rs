mod commander;
mod controlled_virtual_scroll_view;
mod file_explorer;
mod lsp;
mod manager;
mod parser;
mod sidebar;
mod sidepanel;
mod tab;
mod tabs;
mod text_area;
mod use_editable;
mod use_metrics;
mod utils;

use std::collections::HashMap;

use commander::*;
use dioxus_std::utils::channel::use_channel;
use dioxus_std::utils::channel::use_listen_channel;
use file_explorer::*;
use freya::prelude::{keyboard::Code, *};
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
    let lsp_status = use_channel::<(String, String)>(cx, 5);
    let editor_manager = use_state::<EditorManager>(cx, || EditorManager::new(lsp_status.clone()));
    let show_commander = use_state(cx, || false);
    let commands = cx.use_hook(|| {
        vec![Command::new("fs".to_string(), {
            to_owned![editor_manager];
            Box::new(move |size: &str| {
                if let Ok(size) = size.parse::<f32>() {
                    editor_manager.with_mut(|editor_manager| {
                        editor_manager.set_fontsize(size);
                    })
                }
            })
        })]
    });
    let lsp_messages = use_state::<HashMap<String, String>>(cx, HashMap::default);

    use_listen_channel(cx, &lsp_status, {
        to_owned![lsp_messages];
        move |message| {
            to_owned![lsp_messages];
            async move {
                match message {
                    Ok((name, val)) => lsp_messages.with_mut(|msgs| {
                        msgs.insert(name, val);
                    }),
                    Err(err) => {
                        println!("{err:?}")
                    }
                }
            }
        }
    });

    let split_panel = move |_| {
        to_owned![editor_manager];
        editor_manager.with_mut(|editor_manager| {
            editor_manager.push_panel(Panel::new());
            editor_manager.set_focused_panel(editor_manager.panels().len() - 1);
        });
    };

    let onsubmit = {
        move |_| {
            editor_manager.with_mut(|editor_manager| {
                editor_manager.set_focused(true);
                show_commander.set(false);
            });
        }
    };

    let onkeydown = move |e: KeyboardEvent| {
        editor_manager.with_mut(|editor_manager| {
            if e.code == Code::Escape {
                if *show_commander.current() {
                    editor_manager.set_focused(true);
                    show_commander.set(false);
                } else {
                    editor_manager.set_focused(false);
                    show_commander.set(true);
                }
            }
        })
    };

    let onglobalclick = |_| {
        show_commander.set(false);
    };

    let panels_len = editor_manager.get().panels().len();
    let panes_width = 100.0 / panels_len as f32;
    let panel = editor_manager.panel(editor_manager.focused_panel);
    let cursor = if let Some(active_tab) = panel.active_tab() {
        panel
            .tab(active_tab)
            .as_text_editor()
            .map(|editor| editor.cursor())
    } else {
        None
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
                Sidebar { editor_manager: editor_manager.clone() }
                Divider {}
                Sidepanel {
                    FileExplorer { editor_manager: editor_manager.clone() }
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
                        editor_manager.get().panels().iter().enumerate().map(move |(panel_index, panel)| {
                            let is_focused = editor_manager.get().focused_panel() == panel_index;
                            let active_tab_index = panel.active_tab();
                            let panel_background = if is_focused {
                                "rgb(247, 127, 0)"
                            } else {
                                "transparent"
                            };

                            let close_panel = move |_: MouseEvent| {
                                editor_manager.with_mut(|editor_manager| {
                                    editor_manager.close_panel(panel_index);
                                });
                            };

                            rsx!(
                                rect {
                                    direction: "vertical",
                                    height: "100%",
                                    width: "{panes_width}%",
                                    rect {
                                        direction: "horizontal",
                                        height: "50",
                                        width: "100%",
                                        padding: "2.5",
                                        ScrollView {
                                            direction: "horizontal",
                                            width: "calc(100% - 55)",
                                            editor_manager.get().panel(panel_index).tabs().iter().enumerate().map(|(i, tab)| {
                                                let is_selected = active_tab_index == Some(i);
                                                let (tab_id, tab_title) = tab.get_data();
                                                rsx!(
                                                    Tab {
                                                        key: "{tab_id}",
                                                        onclick: move |_| {
                                                            editor_manager.with_mut(|editor_manager| {
                                                                editor_manager.set_focused_panel(panel_index);
                                                                editor_manager.panel_mut(panel_index).set_active_tab(i);
                                                            });
                                                        },
                                                        onclickclose: move |_| {
                                                            editor_manager.with_mut(|editor_manager| {
                                                                editor_manager.close_editor(panel_index, i);
                                                            });
                                                        },
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
                                    }
                                    rect {
                                        height: "calc(100%-50)",
                                        width: "100%",
                                        background: "{panel_background}",
                                        padding: "1.5",
                                        onclick: move |_| {
                                            editor_manager.with_mut(|editor_manager| {
                                                editor_manager.set_focused_panel(panel_index);
                                            });
                                        },
                                        if let Some(active_tab_index) = active_tab_index {
                                            let active_tab = panel.tab(active_tab_index);
                                            let (tab_id, _) = active_tab.get_data();
                                            match active_tab {
                                                PanelTab::TextEditor(editor) => {
                                                    rsx!(
                                                        CodeEditorTab {
                                                            key: "{tab_id}-{active_tab_index}",
                                                            manager: editor_manager.clone(),
                                                            panel_index: panel_index,
                                                            editor: active_tab_index,
                                                            language_id: editor.language_id.clone(),
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
                                                    if panels_len > 1 {
                                                        rsx!(
                                                            Button {
                                                                onclick: close_panel,
                                                                label {
                                                                    "Close panel"
                                                                }
                                                            }
                                                        )
                                                    } else {
                                                        rsx!(
                                                            label {
                                                                "Coding is fun"
                                                            }
                                                        )
                                                    }
                                                }
                                            )
                                        }
                                    }
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
