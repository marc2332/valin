use crate::hooks::*;
use crate::utils::*;
use crate::{
    components::*,
    fs::{FSLocal, FSTransport},
};
use crate::{
    constants::{BASE_FONT_SIZE, MAX_FONT_SIZE},
    keyboard_navigation::use_keyboard_navigation,
    Args,
};
use dioxus_radio::prelude::*;
use dioxus_sdk::clipboard::use_clipboard;
use freya::prelude::keyboard::{Key, Modifiers};
use freya::prelude::*;
use std::{rc::Rc, sync::Arc};
use tokio::{fs::OpenOptions, io::AsyncWriteExt};

use crate::state::{AppStateUtils, EditorSidePanel, EditorView, PanelTab};
use crate::{
    commands::{EditorCommand, FontSizeCommand, SplitCommand},
    state::{AppState, Channel},
};

#[allow(non_snake_case)]
pub fn App() -> Element {
    // Initialize the Language Server Status reporters
    let (lsp_statuses, lsp_sender) = use_lsp_status();

    // Initialize the default FS Transport
    let default_transport = use_hook::<FSTransport>(|| Arc::new(Box::new(FSLocal)));

    // Initilize the clipboard context
    let clipboard = use_clipboard();

    // Initialize the State Manager
    use_init_radio_station::<AppState, Channel>(move || {
        let args = consume_context::<Arc<Args>>();

        let mut state = AppState::new(lsp_sender);

        if args.paths.is_empty() {
            // Default tab
            state.push_tab(PanelTab::Welcome, 0, true);
        }

        state
    });

    // Subscribe to the State Manager
    let mut radio_app_state = use_radio::<AppState, Channel>(Channel::Global);

    // Load specified files and folders asynchronously
    use_hook({
        to_owned![default_transport];
        move || {
            let args = consume_context::<Arc<Args>>();
            spawn(async move {
                for path in &args.paths {
                    // Files
                    if path.is_file() {
                        let root_path = path.parent().unwrap_or(path).to_path_buf();
                        let content = default_transport.read_to_string(path).await;
                        if let Ok(content) = content {
                            let mut app_state =
                                radio_app_state.write_channel(Channel::FileExplorer);
                            let font_size = app_state.font_size();
                            let font_collection = app_state.font_collection.clone();
                            app_state.open_file(
                                path.clone(),
                                root_path,
                                clipboard,
                                content,
                                default_transport.clone(),
                                font_size,
                                &font_collection,
                            );
                        }
                    }
                    // Folders
                    else if path.is_dir() {
                        let folder_path = default_transport.canonicalize(path).await.unwrap();

                        let items = read_folder_as_items(&folder_path, &default_transport).await;
                        if let Ok(items) = items {
                            let mut app_state =
                                radio_app_state.write_channel(Channel::FileExplorer);
                            app_state.open_folder(TreeItem::Folder {
                                path: folder_path,
                                state: FolderState::Opened(items),
                            });
                        }
                    }
                }
            });
        }
    });

    // Initialize the Commands
    let commands = use_hook::<Rc<Vec<Box<dyn EditorCommand>>>>(|| {
        Rc::new(vec![
            Box::new(FontSizeCommand(radio_app_state)),
            Box::new(SplitCommand(radio_app_state)),
        ])
    });

    let mut keyboard_navigation = use_keyboard_navigation();

    let onsubmitcommander = move |_| {
        keyboard_navigation.callback(move || {
            let mut app_state = radio_app_state.write_channel(Channel::Global);
            app_state.set_focused_view_to_previous();
        })
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

                        if let Some((Some(file_path), rope, transport)) = editor_data {
                            spawn(async move {
                                let mut writer = transport
                                    .open(&file_path, OpenOptions::default().write(true))
                                    .await
                                    .unwrap();
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

    let focused_view = radio_app_state.read().focused_view;
    let panels_len = radio_app_state.read().panels().len();
    let panes_width = 100.0 / panels_len as f32;

    rsx!(
        rect {
            font_size: "14",
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
                height: "calc(100% - 35)",
                direction: "horizontal",
                EditorSidebar {}
                Divider {}
                if let Some(side_panel) = radio_app_state.read().side_panel {
                    Sidepanel {
                        match side_panel {
                            EditorSidePanel::FileExplorer => {
                                rsx!(
                                    FileExplorer {
                                        transport: default_transport
                                    }
                                )
                            }
                        }
                    }
                    Divider {}
                }
                rect {
                    width: "fill",
                    height: "fill",
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
            VerticalDivider {}
            StatusBar {
                lsp_statuses,
                focused_view
            }
        }
    )
}
