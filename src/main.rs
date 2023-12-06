mod commands;
mod components;
mod history;
mod hooks;
mod lsp;
mod parser;
mod tabs;
mod utils;

use components::*;
use freya::prelude::keyboard::{Key, Modifiers};
use freya::prelude::*;
use futures::StreamExt;
use hooks::*;
use std::collections::HashMap;
use utils::*;

use crate::commands::{EditorCommand, FontSizeCommand, SplitCommand};

static BASE_FONT_SIZE: f32 = 5.0;
static MAX_FONT_SIZE: f32 = 150.0;

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
    let focused_view = manager.current().focused_view.clone();

    // Commands
    let commands = cx.use_hook::<Vec<Box<dyn EditorCommand>>>(|| {
        vec![
            Box::new(FontSizeCommand(manager.clone())),
            Box::new(SplitCommand(manager.clone())),
        ]
    });

    let onsubmitcommander = {
        to_owned![manager];
        move |_| {
            let mut manager = manager.global_write();
            manager.set_focused_view_to_previous();
        }
    };

    let onkeydown =
        {
            to_owned![manager];
            move |e: KeyboardEvent| match &e.key {
                Key::Escape => {
                    let mut manager = manager.global_write();
                    if manager.focused_view == EditorView::Commander {
                        manager.set_focused_view_to_previous();
                    } else {
                        manager.set_focused_view(EditorView::Commander);
                    }
                }
                Key::Character(ch) if e.modifiers.contains(Modifiers::ALT) => {
                    let mut manager = manager.global_write();
                    let font_size = manager.font_size;
                    match ch.as_str() {
                        "+" => manager
                            .set_fontsize((font_size + 4.0).clamp(BASE_FONT_SIZE, MAX_FONT_SIZE)),
                        "-" => manager
                            .set_fontsize((font_size - 4.0).clamp(BASE_FONT_SIZE, MAX_FONT_SIZE)),
                        "e" => {
                            if *manager.focused_view() == EditorView::FilesExplorer {
                                manager.set_focused_view(EditorView::CodeEditor)
                            } else {
                                manager.set_focused_view(EditorView::FilesExplorer)
                            }
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        };

    let onglobalmousedown = |_| {
        if *manager.current().focused_view() == EditorView::Commander {
            let mut manager = manager.global_write();
            manager.set_focused_view_to_previous();
        }
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
            onglobalmousedown: onglobalmousedown,
            if focused_view == EditorView::Commander {
                rsx!(
                    Commander {
                        onsubmit: onsubmitcommander,
                        commands: commands
                    }
                )
            }
            rect {
                height: "calc(100% - 25)",
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
                    rect {
                        height: "100%",
                        width: "100%",
                        direction: "horizontal",
                        manager.current().panels().iter().enumerate().map(|(panel_index, _)| {
                            rsx!(
                                EditorPanel {
                                    key: "{panel_index}",
                                    panel_index: panel_index,
                                    width: "{panes_width}%"
                                }
                            )
                        })
                    }
                }
            }
            VerticalDivider {}
            StatusBar {
                cursor: cursor.clone(),
                lsp_messages: lsp_messages.clone(),
                focused_view: focused_view
            }
        }
    )
}
