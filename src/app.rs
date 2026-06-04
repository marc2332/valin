use crate::Args;
use crate::components::StatusBar;
use crate::components::{EditorTabButton, EmptyPanel};
use crate::settings::watch_settings;
use crate::state::{EditorSidePanel, EditorView, TabProps};
use crate::theme::github_dark_theme;
use crate::views::commander::commander_ui::Commander;
use crate::views::file_explorer::FileExplorer;
use crate::views::file_explorer::file_explorer_ui::{
    ExplorerItem, FolderState, read_folder_as_items,
};
use crate::views::file_search::file_search_ui::FileSearch;
use crate::views::panels::tabs::editor::EditorTab;
use crate::views::panels::tabs::welcome::WelcomeTab;
use crate::views::tab_switcher::tab_switcher_ui::TabSwitcher;
use crate::{
    fs::{FSLocal, FSTransport},
    state::EditorCommands,
};
use crate::{global_defaults::GlobalDefaults, state::KeyboardShortcuts};
use freya::helpers::from_fn;
use freya::prelude::*;
use freya::radio::*;
use futures::StreamExt;
use std::rc::Rc;
use std::sync::Arc;
use tracing::info;

use crate::state::{AppState, AppTask, Channel as AppChannel, PanelId, TabId};

#[derive(PartialEq)]
pub struct AppView(pub Args);
impl App for AppView {
    fn render(&self) -> impl IntoElement {
        use_init_theme(github_dark_theme);
        use_provide_context(|| Rc::new(self.0.clone()));

        // Initialize the State Manager
        let mut radio_app_state = use_hook(move || {
            let default_transport: FSTransport = Arc::new(Box::new(FSLocal));
            let (task_sender, mut task_receiver) = futures_channel::mpsc::unbounded::<AppTask>();

            let mut app_state = AppState::new(default_transport, task_sender);

            if self.0.paths.is_empty() {
                WelcomeTab::open_with(&mut app_state);
            }

            let station = RadioStation::create(app_state);
            provide_context(station);
            let mut radio_app_state = Radio::new(State::create(RadioAntenna::new(
                AppChannel::Global,
                station,
            )));

            spawn(async move {
                while let Some(task) = task_receiver.next().await {
                    match task {
                        AppTask::OpenFile { path, panel_id } => {
                            let transport = radio_app_state.read().default_transport.clone();
                            let mut app_state = radio_app_state.write();
                            app_state.focused_panel = Some(panel_id);
                            EditorTab::open_with(
                                radio_app_state,
                                &mut app_state,
                                path,
                                transport.as_read(),
                            );
                        }
                    }
                }
            });

            radio_app_state
        });

        // Load specified files and folders asynchronously
        use_hook(move || {
            let args = self.0.clone();
            spawn(async move {
                for path in args.paths {
                    if path.is_file() {
                        let transport = radio_app_state.read().default_transport.clone();

                        let mut app_state = radio_app_state.write();
                        EditorTab::open_with(
                            radio_app_state,
                            &mut app_state,
                            path.clone(),
                            transport.as_read(),
                        )
                    } else if path.is_dir() {
                        let mut app_state = radio_app_state.write_channel(AppChannel::FileExplorer);
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

        let mut editor_commands = use_hook(|| State::create(EditorCommands::default()));
        let mut keyboard_shorcuts = use_hook(|| State::create(KeyboardShortcuts::default()));

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

        let on_global_key_down = move |e: Event<KeyboardEventData>| {
            let handled = keyboard_shorcuts.write().run(
                e.data(),
                &mut editor_commands.write(),
                radio_app_state,
            );
            if handled {
                e.stop_propagation();
                e.prevent_default();
            }
        };

        let on_global_key_up = move |e: Event<KeyboardEventData>| {
            let data = e.data();
            let is_ctrl = matches!(data.code, Code::ControlLeft | Code::ControlRight);
            if !is_ctrl {
                return;
            }
            if radio_app_state.read().tab_switcher.is_none() {
                return;
            }
            let mut app_state = radio_app_state.write_channel(AppChannel::Global);
            app_state.commit_tab_switcher();
        };

        let (focused_view, side_panel) = {
            let app_state = radio_app_state.read();
            (app_state.focused_view, app_state.side_panel)
        };

        let app_state_writable = radio_app_state.slice_mut_current(|s| s).into_writable();

        let docking_area = DockingArea::new(
            app_state_writable.clone(),
            {
                let app_state_writable = app_state_writable.clone();
                move |ctx: ContentContext<TabId, PanelId>| {
                    let panel_id = ctx.panel_id;
                    let Some(tab_id) = ctx.tab_id else {
                        return EmptyPanel { panel_id }.into_element();
                    };

                    let render_fn = app_state_writable.peek().tab(&tab_id).render();
                    let content = from_fn(tab_id, TabProps { tab_id }, render_fn);
                    rect()
                        .expanded()
                        .background((8, 8, 12))
                        .on_pointer_down(move |_| {
                            if radio_app_state.read().focused_panel == Some(panel_id) {
                                return;
                            }
                            let mut state = radio_app_state.write_channel(AppChannel::Global);
                            state.focused_panel = Some(panel_id);
                            if state.focused_view != EditorView::Panels {
                                state.focused_view = EditorView::Panels;
                            }
                        })
                        .child(content)
                        .into_element()
                }
            },
            {
                let app_state_writable = app_state_writable.clone();
                move |ctx: TabContext<TabId>| {
                    let tab_id = ctx.tab_id;
                    let (tab_data, is_active) = {
                        let state = app_state_writable.peek();
                        let tab_data = state.tab(&tab_id).get_data();
                        let is_active = state
                            .panel_tree
                            .as_ref()
                            .and_then(|tree| {
                                let (panel_id, _) = tree.find_tab(&tab_id)?;
                                tree.panel(&panel_id)
                                    .map(|p| p.active_tab_id == Some(tab_id))
                            })
                            .unwrap_or(false);
                        (tab_data, is_active)
                    };

                    EditorTabButton {
                        tab_id,
                        on_close: (move |_: ()| {
                            radio_app_state
                                .write_channel(AppChannel::Global)
                                .close_tab(tab_id);
                        })
                        .into(),
                        value: tab_data.title,
                        is_selected: is_active || ctx.is_drop_target,
                        icon: tab_data.icon,
                    }
                    .into_element()
                }
            },
            {
                let app_state_writable = app_state_writable.clone();
                move |tab_id: TabId| {
                    let tab_data = app_state_writable.peek().tab(&tab_id).get_data();
                    rect()
                        .interactive(false)
                        .child(EditorTabButton {
                            tab_id,
                            on_close: (|_: ()| {}).into(),
                            value: tab_data.title,
                            is_selected: true,
                            icon: tab_data.icon,
                        })
                        .into_element()
                }
            },
            move |ctx: TabBarContext<PanelId>| {
                let panel_id = ctx.panel_id;
                let children = ctx.tab_children;
                let show_close_panel = app_state_writable.peek().panels_in_order().len() > 1;

                let split_panel = move |_| {
                    radio_app_state
                        .write_channel(AppChannel::Global)
                        .split_panel(panel_id);
                };
                let close_panel = move |e: Event<PressEventData>| {
                    e.stop_propagation();
                    e.prevent_default();
                    radio_app_state
                        .write_channel(AppChannel::Global)
                        .close_panel(panel_id);
                };

                rect()
                    .horizontal()
                    .height(Size::px(32.))
                    .width(Size::fill())
                    .cross_align(Alignment::Center)
                    .content(Content::Flex)
                    .child(
                        ScrollView::new()
                            .direction(Direction::Horizontal)
                            .width(Size::flex(1.))
                            .show_scrollbar(false)
                            .child(
                                rect()
                                    .horizontal()
                                    .cross_align(Alignment::Center)
                                    .children(children),
                            ),
                    )
                    .child(
                        rect()
                            .horizontal()
                            .cross_align(Alignment::Center)
                            .main_align(Alignment::End)
                            .height(Size::fill())
                            .spacing(4.0)
                            .padding(4.0)
                            .maybe_child(show_close_panel.then(|| {
                                Button::new()
                                    .flat()
                                    .height(Size::fill())
                                    .padding((0., 8.))
                                    .on_press(close_panel)
                                    .child(
                                        svg(freya::icons::lucide::x())
                                            .width(Size::px(16.0))
                                            .height(Size::px(16.0))
                                            .color((200, 200, 200)),
                                    )
                            }))
                            .child(
                                Button::new()
                                    .flat()
                                    .height(Size::fill())
                                    .padding((0., 8.))
                                    .on_press(split_panel)
                                    .child(
                                        svg(freya::icons::lucide::columns_2())
                                            .width(Size::px(20.0))
                                            .height(Size::px(20.0))
                                            .color((200, 200, 200)),
                                    ),
                            ),
                    )
                    .into_element()
            },
        )
        .preview_element(
            rect()
                .interactive(false)
                .expanded()
                .background((255, 255, 255, 0.08)),
        );

        // Wrap docking area with optional side panel
        let main_container = {
            let mut container = ResizableContainer::new().direction(Direction::Horizontal);

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

            container = container.panel(
                ResizablePanel::new(PanelSize::percent(100.))
                    .key(side_panel.is_some())
                    .order(1usize)
                    .child(docking_area),
            );

            container
        };

        rect()
            .font_size(14.)
            .color(Color::from((230, 237, 243)))
            .background((8, 8, 12))
            .expanded()
            .on_global_key_down(on_global_key_down)
            .on_global_key_up(on_global_key_up)
            .maybe_child(
                (focused_view == EditorView::Commander).then_some(Commander { editor_commands }),
            )
            .maybe_child(
                (focused_view == EditorView::FileSearch).then_some(FileSearch { radio_app_state }),
            )
            .maybe_child(
                (focused_view == EditorView::TabSwitcher)
                    .then_some(TabSwitcher { radio_app_state }),
            )
            .child(
                rect()
                    .height(Size::func(|ctx| Some(ctx.parent - 31.)))
                    .child(main_container),
            )
            .child(StatusBar { focused_view })
    }
}
