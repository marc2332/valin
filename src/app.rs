use crate::{
    components::*,
    fs::{FSLocal, FSTransport},
    tabs::welcome::WelcomeTab,
};
use crate::{global_shortcuts::GlobalShortcuts, state::KeyboardShortcuts};
use crate::{hooks::*, settings::watch_settings};
use crate::{keyboard_navigation::use_keyboard_navigation, Args};
use crate::{tabs::editor::EditorTab, utils::*};
use dioxus_radio::prelude::*;
use dioxus_sdk::clipboard::use_clipboard;
use freya::prelude::*;
use std::{rc::Rc, sync::Arc};
use tracing::info;

use crate::state::{EditorSidePanel, EditorView};
use crate::{
    commands::{EditorCommand, FontSizeCommand, SplitCommand},
    state::{AppState, Channel},
};

#[allow(non_snake_case)]
pub fn App() -> Element {
    // Initialize the Language Server Status reporters
    let (lsp_statuses, lsp_sender) = use_lsp_status();

    // Initilize the clipboard context
    let clipboard = use_clipboard();

    // Initialize the State Manager
    use_init_radio_station::<AppState, Channel>(move || {
        let args = consume_context::<Arc<Args>>();
        let default_transport: FSTransport = Arc::new(Box::new(FSLocal));

        let mut app_state = AppState::new(lsp_sender, default_transport, clipboard);

        if args.paths.is_empty() {
            // Default tab
            WelcomeTab::open_with(&mut app_state);
        }

        app_state
    });

    // Subscribe to the State Manager
    let mut radio_app_state = use_radio::<AppState, Channel>(Channel::Global);

    // Load specified files and folders asynchronously
    use_hook(move || {
        let args = consume_context::<Arc<Args>>();
        spawn(async move {
            for path in &args.paths {
                // Files
                if path.is_file() {
                    let root_path = path.parent().unwrap_or(path).to_path_buf();
                    let transport = radio_app_state.read().default_transport.clone();

                    let content = transport.read_to_string(path).await;
                    if let Ok(content) = content {
                        let mut app_state = radio_app_state.write();

                        EditorTab::open_with(&mut app_state, path.clone(), root_path, content);
                    }
                }
                // Folders
                else if path.is_dir() {
                    let mut app_state = radio_app_state.write_channel(Channel::FileExplorer);
                    let folder_path = app_state
                        .default_transport
                        .canonicalize(path)
                        .await
                        .unwrap();

                    let items =
                        read_folder_as_items(&folder_path, &app_state.default_transport).await;
                    if let Ok(items) = items {
                        app_state.open_folder(TreeItem::Folder {
                            path: folder_path,
                            state: FolderState::Opened(items),
                        });
                    }
                }
            }
        });
    });

    use_hook(|| {
        spawn(async move {
            let res = watch_settings(radio_app_state).await;
            if res.is_none() {
                info!("Failed to watch the settings in background.");
            }
            println!("{res:?}");
        })
    });

    // Initialize the Commands
    let commands = use_hook::<Rc<Vec<Box<dyn EditorCommand>>>>(|| {
        Rc::new(vec![
            Box::new(FontSizeCommand(radio_app_state)),
            Box::new(SplitCommand(radio_app_state)),
        ])
    });

    // Initialize the Shorcuts
    let keyboard_shorcuts = use_hook(|| {
        let mut keyboard_shorcuts = KeyboardShortcuts::default();

        GlobalShortcuts::register_handlers(&mut keyboard_shorcuts);
        EditorTab::register_handlers(&mut keyboard_shorcuts);

        Rc::new(keyboard_shorcuts)
    });

    let mut keyboard_navigation = use_keyboard_navigation();

    let onsubmitcommander = move |_| {
        keyboard_navigation.callback(move || {
            let mut app_state = radio_app_state.write_channel(Channel::Global);
            app_state.set_focused_view_to_previous();
        })
    };

    let onkeydown = move |e: KeyboardEvent| {
        keyboard_shorcuts.run(&e.data, radio_app_state);
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
                if let Some(side_panel) = radio_app_state.read().side_panel {
                    Sidepanel {
                        match side_panel {
                            EditorSidePanel::FileExplorer => {
                                rsx!(
                                    FileExplorer {  }
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
