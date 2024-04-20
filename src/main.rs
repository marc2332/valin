#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod commands;
mod components;
mod hooks;
mod lsp;
mod parser;
mod state;
mod tabs;
mod utils;

use components::*;
use dioxus_radio::prelude::*;
use freya::prelude::keyboard::{Key, Modifiers};
use freya::prelude::*;
use futures::StreamExt;
use hooks::*;
use std::{collections::HashMap, rc::Rc};
use tokio::{fs, io::AsyncWriteExt};
use utils::*;

use crate::state::{AppStateUtils, EditorSidePanel, EditorView, PanelTab};
use crate::{
    commands::{EditorCommand, FontSizeCommand, SplitCommand},
    state::{AppState, Channel},
};

static BASE_FONT_SIZE: f32 = 5.0;
static MAX_FONT_SIZE: f32 = 150.0;

const CUSTOM_THEME: Theme = Theme {
    button: ButtonTheme {
        border_fill: Cow::Borrowed("rgb(50, 50, 50)"),
        ..DARK_THEME.button
    },
    ..DARK_THEME
};

fn main() {
    launch_cfg(
        app,
        LaunchConfig::<()>::builder()
            .with_width(900.0)
            .with_height(600.0)
            .with_title("freya-editor")
            .build(),
    );
}

fn app() -> Element {
    rsx!(
        ThemeProvider { theme: CUSTOM_THEME, Body {} }
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

    use_init_radio_station::<AppState, Channel>(|| {
        let mut state = AppState::new(lsp_status_coroutine);
        state.push_tab(PanelTab::Welcome, 0, true);
        state
    });
    let mut radio_app_state = use_radio::<AppState, Channel>(Channel::Global);

    let focused_view = radio_app_state.read().focused_view;

    // Commands
    let commands = use_hook::<Rc<Vec<Box<dyn EditorCommand>>>>(|| {
        Rc::new(vec![
            Box::new(FontSizeCommand(radio_app_state)),
            Box::new(SplitCommand(radio_app_state)),
        ])
    });

    let onsubmitcommander = move |_| {
        let mut app_state = radio_app_state.write_channel(Channel::Global);
        app_state.set_focused_view_to_previous();
    };

    let onkeydown = move |e: KeyboardEvent| match &e.key {
        Key::Escape => {
            let mut app_state = radio_app_state.write_channel(Channel::Global);
            if app_state.focused_view == EditorView::Commander {
                app_state.set_focused_view_to_previous();
            } else {
                app_state.set_focused_view(EditorView::Commander);
            }
        }
        Key::Character(ch) => {
            if e.modifiers.contains(Modifiers::ALT) {
                match ch.as_str() {
                    "+" => {
                        let mut app_state = radio_app_state.write_channel(Channel::AllTabs);
                        let font_size = app_state.font_size;
                        app_state
                            .set_fontsize((font_size + 4.0).clamp(BASE_FONT_SIZE, MAX_FONT_SIZE))
                    }
                    "-" => {
                        let mut app_state = radio_app_state.write_channel(Channel::AllTabs);
                        let font_size = app_state.font_size;
                        app_state
                            .set_fontsize((font_size - 4.0).clamp(BASE_FONT_SIZE, MAX_FONT_SIZE))
                    }
                    "e" => {
                        let mut app_state = radio_app_state.write_channel(Channel::Global);
                        if *app_state.focused_view() == EditorView::FilesExplorer {
                            app_state.set_focused_view(EditorView::CodeEditor)
                        } else {
                            app_state.set_focused_view(EditorView::FilesExplorer)
                        }
                    }
                    _ => {}
                }
            } else if e.modifiers == Modifiers::CONTROL && ch.as_str() == "s" {
                let (focused_view, panel, active_tab) = radio_app_state.get_focused_data();

                if focused_view == EditorView::CodeEditor {
                    if let Some(active_tab) = active_tab {
                        let editor_data = radio_app_state.editor_mut_data(panel, active_tab);

                        if let Some((path, rope)) = editor_data {
                            spawn(async move {
                                let mut writer =
                                    fs::File::options().write(true).open(path).await.unwrap();
                                for chunk in rope.chunks() {
                                    writer.write_all(chunk.as_bytes()).await.unwrap();
                                }
                                writer.flush().await.unwrap();
                                drop(writer);

                                let mut app_state = radio_app_state
                                    .write_channel(Channel::follow_tab(panel, active_tab));
                                let editor = app_state.try_editor_mut(panel, active_tab);
                                if let Some(editor) = editor {
                                    editor.mark_as_saved()
                                }
                            });
                        }
                    }
                }
            }
        }
        _ => {}
    };

    let onglobalmousedown = move |_| {
        if *radio_app_state.read().focused_view() == EditorView::Commander {
            let mut app_state = radio_app_state.write_channel(Channel::Global);
            app_state.set_focused_view_to_previous();
        }
    };

    let panels_len = radio_app_state.read().panels().len();
    let panes_width = 100.0 / panels_len as f32;

    let cursor = {
        let app_state = radio_app_state.read();
        let panel = app_state.panel(app_state.focused_panel);
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
                if let Some(side_panel) = radio_app_state.read().side_panel {
                    Sidepanel {
                        match side_panel {
                            EditorSidePanel::FileExplorer => {
                                rsx!(
                                    FileExplorer {}
                                )
                            }
                        }
                    }
                    Divider {}
                }
                rect {
                    direction: "vertical",
                    width: "fill",
                    height: "100%",
                    rect {
                        height: "100%",
                        width: "100%",
                        direction: "horizontal",
                        {radio_app_state.read().panels().iter().enumerate().map(|(panel_index, _)| {
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
