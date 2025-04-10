use crate::views::commander::commander_ui::Commander;
use crate::views::file_explorer::file_explorer_ui::{
    read_folder_as_items, ExplorerItem, FileExplorer, FolderState,
};
use crate::views::search::search_ui::Search;
use crate::Args;
use crate::{
    components::*,
    fs::{FSLocal, FSTransport},
    state::EditorCommands,
    views::panels::tabs::welcome::WelcomeTab,
};
use crate::{global_defaults::GlobalDefaults, state::KeyboardShortcuts};
use crate::{hooks::*, settings::watch_settings};
use crate::{utils::*, views::panels::tabs::editor::EditorTab};
use dioxus_clipboard::prelude::use_clipboard;
use dioxus_radio::prelude::*;
use freya::prelude::*;
use std::sync::Arc;
use tracing::info;

use crate::state::{AppState, Channel};
use crate::state::{EditorSidePanel, EditorView};

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

                        EditorTab::open_with(
                            radio_app_state,
                            &mut app_state,
                            path.clone(),
                            root_path,
                            content,
                        );
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
                        app_state.file_explorer.open_folder(ExplorerItem::Folder {
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
        })
    });

    // Initialize the Commands
    let mut editor_commands = use_hook(|| Signal::new(EditorCommands::default()));

    // Initialize the Shorcuts
    let mut keyboard_shorcuts = use_hook(|| Signal::new(KeyboardShortcuts::default()));

    // Register Commands and Shortcuts
    #[allow(clippy::explicit_auto_deref)]
    use_hook(|| {
        GlobalDefaults::init(
            &mut *keyboard_shorcuts.write(),
            &mut *editor_commands.write(),
            radio_app_state,
        );
        EditorTab::init(
            &mut *keyboard_shorcuts.write(),
            &mut *editor_commands.write(),
            radio_app_state,
        );
    });

    // Trigger Shortcuts
    let onglobalkeydown = move |e: KeyboardEvent| {
        keyboard_shorcuts
            .write()
            .run(&e.data, &mut editor_commands.write(), radio_app_state);
    };

    let focused_view = radio_app_state.read().focused_view;
    let panels_len = radio_app_state.read().panels().len();
    let panes_width = 100.0 / panels_len as f32;

    rsx!(
        rect {
            font_size: "14",
            color: "white",
            background: "rgb(17, 20, 21)",
            width: "100%",
            height: "100%",
            onglobalkeydown,
            if focused_view == EditorView::Commander {
                Commander {
                    editor_commands
                }
            } else if focused_view == EditorView::Search {
                Search { }
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
                                panel_index,
                                width: "{panes_width}%"
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
