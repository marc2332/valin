use std::path::{Path, PathBuf};

use freya::prelude::*;
use freya::radio::use_radio;
use futures::StreamExt;
use futures_channel::mpsc::UnboundedSender;

use crate::{
    components::ButtonStatus,
    fs::FSTransport,
    state::{AppState, Channel, EditorView, RadioAppState},
    views::panels::tabs::editor::EditorTab,
};

#[derive(Debug, Clone, PartialEq)]
pub enum FolderState {
    Opened(Vec<ExplorerItem>),
    Closed,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExplorerItem {
    Folder { path: PathBuf, state: FolderState },
    File { path: PathBuf },
}

impl ExplorerItem {
    pub fn path(&self) -> &PathBuf {
        match self {
            Self::Folder { path, .. } => path,
            Self::File { path } => path,
        }
    }

    pub fn set_folder_state(&mut self, folder_path: &PathBuf, folder_state: FolderState) {
        let ExplorerItem::Folder { path, state } = self else {
            return;
        };

        if path == folder_path {
            *state = folder_state;
            return;
        }

        if !folder_path.starts_with(path) {
            return;
        }

        let FolderState::Opened(items) = state else {
            return;
        };
        for item in items.iter_mut() {
            item.set_folder_state(folder_path, folder_state.clone());
        }
    }

    pub fn flat(&self, depth: usize, root_path: &PathBuf) -> Vec<FlatItem> {
        let mut flat_items = vec![self.clone().into_flat(depth, root_path)];
        if let ExplorerItem::Folder {
            state: FolderState::Opened(items),
            ..
        } = self
        {
            for item in items {
                let inner_items = item.flat(depth + 1, root_path);
                flat_items.extend(inner_items);
            }
        }
        flat_items
    }

    fn into_flat(self, depth: usize, root_path: &Path) -> FlatItem {
        match self {
            ExplorerItem::File { path } => FlatItem {
                path,
                is_file: true,
                is_opened: false,
                depth,
                root_path: root_path.to_path_buf(),
            },
            ExplorerItem::Folder { path, state } => FlatItem {
                path,
                is_file: false,
                is_opened: state != FolderState::Closed,
                depth,
                root_path: root_path.to_path_buf(),
            },
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct FlatItem {
    path: PathBuf,
    is_opened: bool,
    is_file: bool,
    depth: usize,
    root_path: PathBuf,
}

pub async fn read_folder_as_items(
    dir: &Path,
    transport: &FSTransport,
) -> smol::io::Result<Vec<ExplorerItem>> {
    let mut paths = transport.read_dir(dir).await?;
    let mut folder_items = Vec::default();
    let mut files_items = Vec::default();

    while let Some(Ok(entry)) = paths.next().await {
        let file_type = entry.file_type().await?;
        let is_file = file_type.is_file();
        let path = entry.path();

        if is_file {
            files_items.push(ExplorerItem::File { path })
        } else {
            folder_items.push(ExplorerItem::Folder {
                path,
                state: FolderState::Closed,
            })
        }
    }

    folder_items.extend(files_items);

    Ok(folder_items)
}

#[derive(Debug, Clone, PartialEq)]
enum TreeTask {
    OpenFolder {
        folder_path: PathBuf,
        root_path: PathBuf,
    },
    CloseFolder {
        folder_path: PathBuf,
        root_path: PathBuf,
    },
    OpenFile {
        file_path: PathBuf,
        root_path: PathBuf,
    },
}

#[derive(Clone, PartialEq)]
pub struct FileExplorer;

impl Component for FileExplorer {
    fn render(&self) -> impl IntoElement {
        let mut radio_app_state = use_radio::<AppState, Channel>(Channel::FileExplorer);
        let app_state = radio_app_state.read();
        let focus = Focus::new_for_id(app_state.file_explorer.focus_id);
        let mut focused_item_index = use_state(|| 0);

        let items = app_state
            .file_explorer
            .folders
            .iter()
            .flat_map(|tree| tree.flat(0, tree.path()))
            .collect::<Vec<FlatItem>>();
        let items_len = items.len();
        let focused_item = items.get(focused_item_index()).cloned();

        let channel = use_hook(move || {
            let (tx, mut rx) = futures_channel::mpsc::unbounded();
            spawn(async move {
                while let Some((task, item_index)) = rx.next().await {
                    // Focus the FilesExplorer view if it wasn't focused already
                    let focused_view = radio_app_state.read().focused_view();
                    if focused_view != EditorView::FilesExplorer {
                        radio_app_state
                            .write_channel(Channel::Global)
                            .focus_view(EditorView::FilesExplorer);
                    }

                    match task {
                        TreeTask::OpenFolder {
                            folder_path,
                            root_path,
                        } => {
                            let transport = radio_app_state.read().default_transport.clone();
                            if let Ok(items) = read_folder_as_items(&folder_path, &transport).await
                            {
                                let mut app_state = radio_app_state.write();
                                let folder = app_state
                                    .file_explorer
                                    .folders
                                    .iter_mut()
                                    .find(|folder| folder.path() == &root_path)
                                    .unwrap();
                                folder.set_folder_state(&folder_path, FolderState::Opened(items));
                            }
                        }
                        TreeTask::CloseFolder {
                            folder_path,
                            root_path,
                        } => {
                            let mut app_state = radio_app_state.write();
                            let folder = app_state
                                .file_explorer
                                .folders
                                .iter_mut()
                                .find(|folder| folder.path() == &root_path)
                                .unwrap();
                            folder.set_folder_state(&folder_path, FolderState::Closed);
                        }
                        TreeTask::OpenFile { file_path, .. } => {
                            let transport = radio_app_state.read().default_transport.clone();
                            let mut app_state = radio_app_state.write_channel(Channel::Global);
                            EditorTab::open_with(
                                radio_app_state,
                                &mut app_state,
                                file_path,
                                transport.as_read(),
                            );
                        }
                    }
                    focused_item_index.set(item_index);
                }
            });
            tx
        });

        let open_dialog = move |_| {
            spawn(async move {
                let folder = rfd::AsyncFileDialog::new().pick_folder().await;

                if let Some(folder) = folder {
                    let transport = radio_app_state.read().default_transport.clone();

                    let path = folder.path().to_owned();
                    let items = read_folder_as_items(&path, &transport)
                        .await
                        .unwrap_or_default();

                    let mut app_state = radio_app_state.write();

                    app_state.file_explorer.open_folder(ExplorerItem::Folder {
                        path,
                        state: FolderState::Opened(items),
                    });

                    app_state.focus_view(EditorView::FilesExplorer);
                }
            });
        };

        let on_key_down = {
            let channel = channel.clone();
            move |ev: Event<KeyboardEventData>| {
                let is_focused_files_explorer =
                    radio_app_state.read().focused_view() == EditorView::FilesExplorer;
                if is_focused_files_explorer {
                    match ev.code {
                        Code::ArrowDown => {
                            focused_item_index.with_mut(|mut i| {
                                if *i < items_len - 1 {
                                    *i += 1
                                }
                            });
                        }
                        Code::ArrowUp => {
                            focused_item_index.with_mut(|mut i| {
                                if *i > 0 {
                                    *i -= 1
                                }
                            });
                        }
                        Code::Enter => {
                            if let Some(item) = &focused_item {
                                let task = match (item.is_file, item.is_opened) {
                                    (true, _) => TreeTask::OpenFile {
                                        file_path: item.path.clone(),
                                        root_path: item.root_path.clone(),
                                    },
                                    (false, true) => TreeTask::CloseFolder {
                                        folder_path: item.path.clone(),
                                        root_path: item.root_path.clone(),
                                    },
                                    (false, false) => TreeTask::OpenFolder {
                                        folder_path: item.path.clone(),
                                        root_path: item.root_path.clone(),
                                    },
                                };
                                let _ = channel.unbounded_send((task, focused_item_index()));
                            }
                        }
                        _ => {}
                    }
                }
            }
        };

        let on_press = move |e: Event<MouseEventData>| {
            e.stop_propagation();
            focus.request_focus();
        };

        let length = items.len();

        if items.is_empty() {
            rect()
                .expanded()
                .center()
                .child(Button::new().on_press(open_dialog).child("Open Folder"))
        } else {
            rect()
                .expanded()
                .on_key_down(on_key_down)
                .on_mouse_up(on_press)
                .a11y_id(focus.a11y_id())
                .child(
                    VirtualScrollView::new_with_data(
                        (items, focused_item_index, radio_app_state),
                        move |a, b| file_explorer_item_builder(a, channel.clone(), b),
                    )
                    .length(length as i32)
                    .item_size(27.)
                    .scroll_with_arrows(false),
                )
        }
    }
}

fn file_explorer_item_builder(
    index: usize,
    channel: UnboundedSender<(TreeTask, usize)>,
    (items, focused_item, radio_app_state): &(Vec<FlatItem>, State<usize>, RadioAppState),
) -> Element {
    let item: &FlatItem = &items[index];

    let name = item
        .path
        .file_name()
        .unwrap()
        .to_owned()
        .to_str()
        .unwrap()
        .to_string();
    let is_focused = *focused_item.read() == index;

    let item = item.clone();
    let icon_svg = {
        let app_state = radio_app_state.read();
        if item.is_file {
            app_state.file_icons.get_file(&item.path).svg.clone()
        } else {
            app_state.file_icons.get_folder(item.is_opened).svg.clone()
        }
    };

    let on_press = move |_| {
        let task = match (item.is_file, item.is_opened) {
            (true, _) => TreeTask::OpenFile {
                file_path: item.path.clone(),
                root_path: item.root_path.clone(),
            },
            (false, true) => TreeTask::CloseFolder {
                folder_path: item.path.clone(),
                root_path: item.root_path.clone(),
            },
            (false, false) => TreeTask::OpenFolder {
                folder_path: item.path.clone(),
                root_path: item.root_path.clone(),
            },
        };
        let _ = channel.unbounded_send((task, index));
    };

    FileExplorerItem {
        depth: item.depth,
        radio_app_state: *radio_app_state,
        on_press: on_press.into(),
        is_focused,
        children: rect()
            .horizontal()
            .cross_align(Alignment::Center)
            .expanded()
            .child(
                svg(icon_svg)
                    .width(Size::px(14.0))
                    .height(Size::px(14.0))
                    .fill(Color::from_rgb(180, 180, 180))
                    .margin(Gaps::new(0., 5., 0., 0.)),
            )
            .child(
                label()
                    .max_lines(1)
                    .text_overflow(TextOverflow::Ellipsis)
                    .text(name),
            )
            .into(),
    }
    .into()
}

#[derive(Clone, PartialEq)]
pub struct FileExplorerItem {
    pub children: Element,
    pub on_press: EventHandler<()>,
    pub depth: usize,
    pub is_focused: bool,
    pub radio_app_state: RadioAppState,
}

impl Component for FileExplorerItem {
    fn render(&self) -> impl IntoElement {
        let mut status = use_state(|| ButtonStatus::Idle);

        let on_mouseenter = move |_| status.set(ButtonStatus::Hovering);

        let on_mouse_leave = move |_| status.set(ButtonStatus::Idle);

        let on_press_handler = self.on_press.clone();
        let on_press = move |_: Event<PressEventData>| {
            on_press_handler.call(());
        };

        let background = match *status.read() {
            ButtonStatus::Idle | ButtonStatus::Hovering if self.is_focused => (29, 32, 33).into(),
            ButtonStatus::Hovering => (29, 32, 33, 0.7).into(),
            ButtonStatus::Idle => Color::TRANSPARENT,
        };

        let color: Color = if self.is_focused {
            (245, 245, 245).into()
        } else {
            (210, 210, 210).into()
        };

        let padding_left = (self.depth * 10) + 10;

        rect()
            .on_pointer_enter(on_mouseenter)
            .on_pointer_leave(on_mouse_leave)
            .on_press(on_press)
            .background(background)
            .width(Size::fill())
            .padding(Gaps::new(0., 0., 0., padding_left as f32))
            .main_align(Alignment::Center)
            .height(Size::px(27.0))
            .color(color)
            .font_size(14.0)
            .child(self.children.clone())
    }
}
