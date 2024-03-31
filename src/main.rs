mod commands;
mod components;
mod editor_manager;
mod history;
mod hooks;
mod lsp;
mod parser;
mod tabs;
mod utils;

use components::*;
use dioxus_radio::prelude::*;
use freya::prelude::keyboard::{Key, Modifiers};
use freya::prelude::*;
use futures::StreamExt;
use hooks::*;
use std::{collections::HashMap, rc::Rc};
use utils::*;

use crate::editor_manager::EditorView;
use crate::{
    commands::{EditorCommand, FontSizeCommand, SplitCommand},
    editor_manager::{EditorManager, SubscriptionModel},
};

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

fn app() -> Element {
    rsx!(
        ThemeProvider { theme: DARK_THEME, Body {} }
    )
}

#[allow(non_snake_case)]
fn Body() -> Element {
    let mut lsp_messages = use_signal::<HashMap<String, String>>(HashMap::default);
    let lsp_status_coroutine = use_coroutine(
        move |mut rx: UnboundedReceiver<(String, String)>| async move {
            while let Some((name, val)) = rx.next().await {
                lsp_messages.with_mut(|msgs| {
                    msgs.insert(name, val);
                })
            }
        },
    );

    use_init_radio_station::<EditorManager, SubscriptionModel>(|| {
        EditorManager::new(lsp_status_coroutine)
    });
    let mut radio = use_radio::<EditorManager, SubscriptionModel>(SubscriptionModel::All);

    let focused_view = radio.read().focused_view;

    // Commands
    let commands = use_hook::<Rc<Vec<Box<dyn EditorCommand>>>>(|| {
        Rc::new(vec![
            Box::new(FontSizeCommand(radio)),
            Box::new(SplitCommand(radio)),
        ])
    });

    let onsubmitcommander = move |_| {
        let mut manager = radio.write_channel(SubscriptionModel::All);
        manager.set_focused_view_to_previous();
    };

    let onkeydown = move |e: KeyboardEvent| match &e.key {
        Key::Escape => {
            let mut manager = radio.write_channel(SubscriptionModel::All);
            if manager.focused_view == EditorView::Commander {
                manager.set_focused_view_to_previous();
            } else {
                manager.set_focused_view(EditorView::Commander);
            }
        }
        Key::Character(ch) if e.modifiers.contains(Modifiers::ALT) => {
            let mut manager = radio.write_channel(SubscriptionModel::All);
            let font_size = manager.font_size;
            match ch.as_str() {
                "+" => manager.set_fontsize((font_size + 4.0).clamp(BASE_FONT_SIZE, MAX_FONT_SIZE)),
                "-" => manager.set_fontsize((font_size - 4.0).clamp(BASE_FONT_SIZE, MAX_FONT_SIZE)),
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
    };

    let onglobalmousedown = move |_| {
        if *radio.read().focused_view() == EditorView::Commander {
            let mut manager = radio.write_channel(SubscriptionModel::All);
            manager.set_focused_view_to_previous();
        }
    };

    let panels_len = radio.read().panels().len();
    let panes_width = 100.0 / panels_len as f32;

    let cursor = {
        let manager = radio.read();
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

    rsx!(
        rect {
            color: "white",
            background: "rgb(20, 20, 20)",
            width: "100%",
            height: "100%",
            onkeydown: onkeydown,
            onglobalmousedown: onglobalmousedown,
            if focused_view == EditorView::Commander {
                Commander {
                    onsubmit: onsubmitcommander,
                    commands: commands
                }
            }
            rect {
                height: "calc(100% - 25)",
                direction: "horizontal",
                EditorSidebar {}
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
                        {radio.read().panels().iter().enumerate().map(|(panel_index, _)| {
                            rsx!(
                                EditorPanel {
                                    key: "{panel_index}",
                                    panel_index: panel_index,
                                    width: format!("{panes_width}%")
                                }
                            )
                        })}
                    }
                }
            }
            VerticalDivider {}
            StatusBar {
                cursor,
                lsp_messages,
                focused_view
            }
        }
    )
}
