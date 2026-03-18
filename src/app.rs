use crate::Args;
use crate::components::{EditorPanel, StatusBar};
use crate::settings::watch_settings;
use crate::state::{EditorSidePanel, EditorView};
use crate::views::commander::commander_ui::Commander;
use crate::views::file_explorer::FileExplorer;
use crate::views::file_explorer::file_explorer_ui::{
    ExplorerItem, FolderState, read_folder_as_items,
};
use crate::views::panels::tabs::editor::EditorTab;
use crate::views::panels::tabs::welcome::WelcomeTab;
use crate::{
    fs::{FSLocal, FSTransport},
    state::EditorCommands,
};
use crate::{global_defaults::GlobalDefaults, state::KeyboardShortcuts};
use freya::prelude::*;
use freya::radio::*;
use std::rc::Rc;
use std::sync::Arc;
use tracing::info;

use crate::state::{AppState, Channel};

#[derive(PartialEq)]
pub struct AppView(pub Args);
impl App for AppView {
    fn render(&self) -> impl IntoElement {
        use_init_theme(|| DARK_THEME);
        use_provide_context(|| Rc::new(self.0.clone()));

        // Initialize the State Manager
        use_init_radio_station::<AppState, Channel>(move || {
            let default_transport: FSTransport = Arc::new(Box::new(FSLocal));

            let mut app_state = AppState::new(default_transport);

            if self.0.paths.is_empty() {
                // Default tab
                WelcomeTab::open_with(&mut app_state);
            }

            app_state
        });

        let mut radio_app_state = use_radio::<AppState, Channel>(Channel::Global);

        // Load specified files and folders asynchronously
        use_hook(move || {
            let args = self.0.clone();
            spawn(async move {
                for path in args.paths {
                    // Files
                    if path.is_file() {
                        let transport = radio_app_state.read().default_transport.clone();

                        let mut app_state = radio_app_state.write();
                        EditorTab::open_with(
                            radio_app_state,
                            &mut app_state,
                            path.clone(),
                            transport.as_read(),
                        )
                    }
                    // Folders
                    else if path.is_dir() {
                        let mut app_state = radio_app_state.write_channel(Channel::FileExplorer);
                        let folder_path = app_state
                            .default_transport
                            .canonicalize(&path)
                            .await
                            .unwrap();

                        let items =
                            read_folder_as_items(&folder_path, &app_state.default_transport).await;
                        if let Ok(items) = items {
                            app_state.file_explorer.open_folder(ExplorerItem::Folder {
                                path: folder_path.to_path_buf(),
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
        let mut editor_commands = use_hook(|| State::create(EditorCommands::default()));

        // Initialize the Shorcuts
        let mut keyboard_shorcuts = use_hook(|| State::create(KeyboardShortcuts::default()));

        // Register Commands and Shortcuts
        use_hook(|| {
            GlobalDefaults::init(
                &mut keyboard_shorcuts.write(),
                &mut editor_commands.write(),
                radio_app_state,
            );
            EditorTab::init(
                &mut keyboard_shorcuts.write(),
                &mut editor_commands.write(),
                radio_app_state,
            );
        });

        // Trigger Shortcuts
        let on_global_key_down = move |e: Event<KeyboardEventData>| {
            keyboard_shorcuts
                .write()
                .run(e.data(), &mut editor_commands.write(), radio_app_state);
        };

        let focused_view = radio_app_state.read().focused_view;
        let side_panel = radio_app_state.read().side_panel;

        // Build the editor panels container
        let panels_container = {
            let panels = radio_app_state.read();

            let mut container = ResizableContainer::new().direction(Direction::Horizontal);
            for (panel_index, _) in panels.panels().iter().enumerate() {
                container = container.panel(
                    ResizablePanel::new(PanelSize::percent(50.))
                        .key(&panel_index)
                        .order(panel_index)
                        .child(EditorPanel { panel_index }),
                );
            }
            container
        };

        // Build the main horizontal layout with optional side panel
        let main_container = {
            let mut container = ResizableContainer::new().direction(Direction::Horizontal);

            // Add side panel if visible
            if let Some(panel) = side_panel {
                container = container.panel(
                    ResizablePanel::new(PanelSize::px(225.))
                        .order(0usize)
                        .min_size(10.)
                        .child(match panel {
                            EditorSidePanel::FileExplorer => FileExplorer,
                        }),
                );
            }

            // Add the panels container
            container = container.panel(
                ResizablePanel::new(PanelSize::percent(100.))
                    .order(1usize)
                    .child(panels_container),
            );

            container
        };

        rect()
            .font_size(14.)
            .color(Color::WHITE)
            .background((17, 20, 21))
            .expanded()
            .on_global_key_down(on_global_key_down)
            .maybe_child(if focused_view == EditorView::Commander {
                Some(Commander { editor_commands })
            } else {
                None
            })
            .child(
                rect()
                    .height(Size::func(|ctx| Some(ctx.parent - 35.)))
                    .child(main_container),
            )
            .child(StatusBar { focused_view })
    }
}
