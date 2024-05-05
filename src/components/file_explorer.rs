use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use dioxus_radio::hooks::use_radio;
use dioxus_sdk::clipboard::use_clipboard;
use freya::elements as dioxus_elements;
use freya::prelude::keyboard::Code;
use freya::prelude::*;
use futures::StreamExt;
use tokio::io;

use crate::{
    fs::FSTransport,
    state::{AppState, Channel, EditorView},
};

#[derive(Debug, Clone, PartialEq)]
pub enum FolderState {
    Opened(Vec<TreeItem>),
    Closed,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TreeItem {
    Folder { path: PathBuf, state: FolderState },
    File { path: PathBuf },
}

impl TreeItem {
    pub fn path(&self) -> &PathBuf {
        match self {
            Self::Folder { path, .. } => path,
            Self::File { path } => path,
        }
    }

    pub fn set_folder_state(&mut self, folder_path: &PathBuf, folder_state: &FolderState) -> bool {
        if let TreeItem::Folder { path, state } = self {
            if path == folder_path {
                *state = folder_state.clone(); // Ugly
                return true;
            } else if folder_path.starts_with(path) {
                if let FolderState::Opened(items) = state {
                    for item in items {
                        let res = item.set_folder_state(folder_path, folder_state);
                        if res {
                            return res;
                        }
                    }
                }
            }
        }
        return false;
    }

    pub fn flat(&self, depth: usize, root_path: &PathBuf) -> Vec<FlatItem> {
        let mut flat_items = vec![self.clone().into_flat(depth, root_path.clone())];
        if let TreeItem::Folder {
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

    fn into_flat(self, depth: usize, root_path: PathBuf) -> FlatItem {
        match self {
            TreeItem::File { path } => FlatItem {
                path,
                is_file: true,
                is_opened: false,
                depth,
                root_path,
            },
            TreeItem::Folder { path, state } => FlatItem {
                path,
                is_file: false,
                is_opened: state != FolderState::Closed,
                depth,
                root_path,
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
) -> io::Result<Vec<TreeItem>> {
    let mut paths = transport.read_dir(dir).await?;
    let mut folder_items = Vec::default();
    let mut files_items = Vec::default();

    while let Ok(Some(entry)) = paths.next_entry().await {
        let file_type = entry.file_type().await?;
        let is_file = file_type.is_file();
        let path = entry.path();

        if is_file {
            files_items.push(TreeItem::File { path })
        } else {
            folder_items.push(TreeItem::Folder {
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
    OpenFolder(PathBuf),
    CloseFolder(PathBuf),
    OpenFile { path: PathBuf, root_path: PathBuf },
}

#[derive(Clone, Props)]
pub struct FileExplorerProps {
    transport: FSTransport,
}

impl PartialEq for FileExplorerProps {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.transport, &other.transport)
    }
}

#[allow(non_snake_case)]
pub fn FileExplorer(FileExplorerProps { transport }: FileExplorerProps) -> Element {
    let clipboard = use_clipboard();
    let mut radio_app_state = use_radio::<AppState, Channel>(Channel::FileExplorer);
    let app_state = radio_app_state.read();
    let is_focused_files_explorer = *app_state.focused_view() == EditorView::FilesExplorer;
    let mut focused_item = use_signal(|| 0);

    let items = app_state
        .file_explorer_folders
        .iter()
        .flat_map(|tree| tree.flat(0, tree.path()))
        .collect::<Vec<FlatItem>>();
    let items_len = items.len();

    let channel = use_coroutine({
        to_owned![transport];
        move |mut rx| {
            async move {
                while let Some((task, item_index)) = rx.next().await {
                    // Focus the FilesExplorer view if it wasn't focused already
                    let focused_view = *radio_app_state.read().focused_view();
                    if focused_view != EditorView::FilesExplorer {
                        radio_app_state
                            .write_channel(Channel::Global)
                            .set_focused_view(EditorView::FilesExplorer);
                    }

                    match task {
                        TreeTask::OpenFolder(folder_path) => {
                            if let Ok(items) = read_folder_as_items(&folder_path, &transport).await
                            {
                                let mut app_state = radio_app_state.write();
                                for folder in &mut app_state.file_explorer_folders {
                                    let res = folder.set_folder_state(
                                        &folder_path,
                                        &FolderState::Opened(items.clone()), // Ugly
                                    );

                                    if res {
                                        break;
                                    }
                                }
                            }
                        }
                        TreeTask::CloseFolder(folder_path) => {
                            let mut app_state = radio_app_state.write();
                            for folder in &mut app_state.file_explorer_folders {
                                let res =
                                    folder.set_folder_state(&folder_path, &FolderState::Closed);

                                if res {
                                    break;
                                }
                            }
                        }
                        TreeTask::OpenFile { path, root_path } => {
                            let content = transport.read_to_string(&path).await;
                            if let Ok(content) = content {
                                let mut app_state = radio_app_state.write_channel(Channel::Global);
                                app_state.open_file(
                                    path,
                                    root_path,
                                    clipboard,
                                    content,
                                    transport.clone(),
                                );
                            } else if let Err(err) = content {
                                println!("Error reading file: {err:?}");
                            }
                        }
                    }
                    focused_item.set(item_index);
                }
            }
        }
    });

    let open_dialog = move |_| {
        to_owned![transport];
        spawn(async move {
            let folder = rfd::AsyncFileDialog::new().pick_folder().await;

            if let Some(folder) = folder {
                let path = folder.path().to_owned();
                let items = read_folder_as_items(&path, &transport)
                    .await
                    .unwrap_or_default();
                let mut app_state = radio_app_state.write_channel(Channel::Global);
                app_state.open_folder(TreeItem::Folder {
                    path,
                    state: FolderState::Opened(items),
                });

                app_state.set_focused_view(EditorView::FilesExplorer);
            }
        });
    };

    let onkeydown = move |ev: KeyboardEvent| {
        let is_focused_files_explorer =
            *radio_app_state.read().focused_view() == EditorView::FilesExplorer;
        if is_focused_files_explorer {
            match ev.code {
                Code::ArrowDown => {
                    focused_item.with_mut(|i| {
                        if *i < items_len - 1 {
                            *i += 1
                        }
                    });
                }
                Code::ArrowUp => {
                    focused_item.with_mut(|i| {
                        if *i > 0 {
                            *i -= 1
                        }
                    });
                }
                _ => {}
            }
        }
    };

    if items.is_empty() {
        rsx!(
            rect {
                width: "100%",
                height: "100%",
                main_align: "center",
                cross_align: "center",
                Button {
                    onclick: open_dialog,
                    label {
                        "Open folder"
                    }
                }
            }
        )
    } else {
        rsx!(rect {
            width: "100%",
            height: "100%",
            padding: "10",
            onkeydown,
            VirtualScrollView {
                theme: theme_with!(ScrollViewTheme {
                    width: "100%".into(),
                    height: "100%".into(),
                }),
                length: items.len(),
                item_size: 25.0,
                builder_args: (items, channel, focused_item, is_focused_files_explorer),
                direction: "vertical",
                scroll_with_arrows: false,
                builder: file_explorer_item_builder
            }
        })
    }
}

type TreeBuilderOptions = (
    Vec<FlatItem>,
    Coroutine<(TreeTask, usize)>,
    Signal<usize>,
    bool,
);

fn file_explorer_item_builder(index: usize, values: &Option<TreeBuilderOptions>) -> Element {
    let (items, channel, focused_item, is_focused_files_explorer) = values.as_ref().unwrap();
    let item: &FlatItem = &items[index];

    let path = item.path.to_str().unwrap().to_owned();
    let name = item
        .path
        .file_name()
        .unwrap()
        .to_owned()
        .to_str()
        .unwrap()
        .to_string();
    let is_focused = *focused_item.read() == index;
    let is_focused_files_explorer = *is_focused_files_explorer;

    if item.is_file {
        to_owned![channel, item];
        let onclick = move |_| {
            channel.send((
                TreeTask::OpenFile {
                    path: item.path.clone(),
                    root_path: item.root_path.clone(),
                },
                index,
            ));
        };
        rsx!(
            FileExplorerItem {
                key: "{path}",
                depth: item.depth,
                onclick: onclick,
                is_focused: is_focused,
                is_focused_files_explorer: is_focused_files_explorer,
                label {
                    max_lines: "1",
                    text_overflow: "ellipsis",
                    "📃 {name}"
                }
            }
        )
    } else {
        to_owned![channel, item];
        let onclick = move |_| {
            if item.is_opened {
                channel.send((TreeTask::CloseFolder(item.path.clone()), index));
            } else {
                channel.send((TreeTask::OpenFolder(item.path.clone()), index));
            }
        };

        let icon = if item.is_opened { "📂" } else { "📁" };

        rsx!(
            FileExplorerItem {
                key: "{path}",
                depth: item.depth,
                onclick: onclick,
                is_focused: is_focused,
                is_focused_files_explorer: is_focused_files_explorer,
                label {
                    max_lines: "1",
                    text_overflow: "ellipsis",
                    "{icon} {name}"
                }
            }
        )
    }
}

#[allow(non_snake_case)]
#[component]
fn FileExplorerItem(
    children: Element,
    onclick: EventHandler<()>,
    depth: usize,
    is_focused: bool,
    is_focused_files_explorer: bool,
) -> Element {
    let mut status = use_signal(|| ButtonStatus::Idle);

    let onmouseenter = move |_| status.set(ButtonStatus::Hovering);
    let onmouseleave = move |_| status.set(ButtonStatus::Idle);

    let background = match *status.read() {
        ButtonStatus::Idle if is_focused && !is_focused_files_explorer => "rgb(35, 35, 35, 150)",
        ButtonStatus::Idle => "transparent",
        ButtonStatus::Hovering => "rgb(35, 35, 35)",
    };

    let border = if is_focused && is_focused_files_explorer {
        "2 solid rgb(255, 255, 255, 100)"
    } else {
        "none"
    };

    rsx!(rect {
        onmouseenter: onmouseenter,
        onmouseleave: onmouseleave,
        onclick: move |_| onclick.call(()),
        onkeydown: move |e| if e.code == Code::Enter && is_focused && is_focused_files_explorer {
            onclick.call(());
        },
        background: "{background}",
        corner_radius: "5",
        margin: "0 0 0 {depth * 10}",
        direction: "horizontal",
        padding: "3 8",
        height: "25",
        border: border,
        {children}
    })
}
