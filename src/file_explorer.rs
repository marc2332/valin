use std::path::PathBuf;

use dioxus::prelude::*;
use dioxus_std::utils::channel::*;
use freya::elements as dioxus_elements;
use freya::prelude::*;
use tokio::fs::read_to_string;
use tokio::{
    fs::{self},
    io,
};

use crate::panels::PanelTab;
use crate::panels::PanelsManager;
use crate::use_editable::EditorData;

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

    pub fn set_folder_state(&mut self, folder_path: &PathBuf, folder_state: &FolderState) {
        if let TreeItem::Folder { path, state } = self {
            if path == folder_path {
                *state = folder_state.clone(); // Ugly
            } else if folder_path.starts_with(path) {
                if let FolderState::Opened(items) = state {
                    for item in items {
                        item.set_folder_state(folder_path, folder_state)
                    }
                }
            }
        }
    }

    pub fn flat(&self, depth: usize) -> Vec<FlatItem> {
        let mut flat_items = vec![self.clone().into_flat(depth)];
        if let TreeItem::Folder {
            state: FolderState::Opened(items),
            ..
        } = self
        {
            for item in items {
                let inner_items = item.flat(depth + 1);
                flat_items.extend(inner_items);
            }
        }
        flat_items
    }

    fn into_flat(self, depth: usize) -> FlatItem {
        match self {
            TreeItem::File { path } => FlatItem {
                path,
                is_file: true,
                is_opened: false,
                depth,
            },
            TreeItem::Folder { path, state } => FlatItem {
                path,
                is_file: false,
                is_opened: state != FolderState::Closed,
                depth,
            },
        }
    }
}

#[derive(Clone, Debug)]
pub struct FlatItem {
    path: PathBuf,
    is_opened: bool,
    is_file: bool,
    depth: usize,
}

async fn read_folder_as_items(dir: &PathBuf) -> io::Result<Vec<TreeItem>> {
    let mut paths = fs::read_dir(dir).await?;
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

#[derive(Debug, Clone)]
enum TreeTask {
    OpenFolder(PathBuf),
    CloseFolder(PathBuf),
    OpenFile(PathBuf),
}

#[inline_props]
#[allow(non_snake_case)]
pub fn FileExplorer(cx: Scope, panels_manager: UseState<PanelsManager>) -> Element {
    let channel = use_channel::<TreeTask>(cx, 5);
    let tree = use_ref::<Option<TreeItem>>(cx, || None);

    let items = use_memo(cx, tree, move |tree| {
        if let Some(tree) = tree.read().as_ref() {
            tree.flat(0)
        } else {
            vec![]
        }
    });

    use_listen_channel(cx, &channel, {
        to_owned![tree, panels_manager];
        move |task| {
            to_owned![tree, panels_manager];
            async move {
                if let Ok(task) = task {
                    match task {
                        TreeTask::OpenFolder(folder_path) => {
                            if let Ok(items) = read_folder_as_items(&folder_path).await {
                                if let Some(tree) = tree.write().as_mut() {
                                    tree.set_folder_state(
                                        &folder_path,
                                        &FolderState::Opened(items),
                                    );
                                }
                            }
                        }
                        TreeTask::CloseFolder(folder_path) => {
                            if let Some(tree) = tree.write().as_mut() {
                                tree.set_folder_state(&folder_path, &FolderState::Closed);
                            }
                        }
                        TreeTask::OpenFile(file_path) => {
                            let content = read_to_string(&file_path).await;
                            if let Ok(content) = content {
                                let root_path = tree.read().as_ref().unwrap().path().clone();
                                panels_manager.with_mut(|panels_manager| {
                                    panels_manager.push_tab(
                                        PanelTab::TextEditor(EditorData::new(
                                            file_path.to_path_buf(),
                                            Rope::from(content),
                                            (0, 0),
                                            "Rust".to_string(),
                                            root_path,
                                        )),
                                        panels_manager.focused_panel(),
                                        true,
                                    );
                                });
                            } else if let Err(err) = content {
                                println!("Error reading file: {err:?}");
                            }
                        }
                    }
                }
            }
        }
    });

    let open_dialog = |_| {
        cx.spawn({
            to_owned![tree];
            async {
                let tree = tree;
                let task = rfd::AsyncFileDialog::new().pick_folder();
                let folder = task.await;

                if let Some(folder) = folder {
                    let path = folder.path().to_owned();
                    let items = read_folder_as_items(&path).await.unwrap_or_default();
                    *tree.write() = Some(TreeItem::Folder {
                        path,
                        state: FolderState::Opened(items),
                    });
                }
            }
        });
    };

    let tree_builder =
        |(_key, index, _cx, values): (_, _, _, &Option<(&Vec<FlatItem>, UseChannel<TreeTask>)>)| {
            let (items, channel) = values.as_ref().unwrap();
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

            if item.is_file {
                to_owned![channel, item];
                let onclick = move |_| {
                    channel
                        .try_send(TreeTask::OpenFile(item.path.clone()))
                        .unwrap();
                };
                rsx!(
                    FileExplorerItem {
                        key: "{path}",
                        depth: item.depth,
                        onclick: onclick,
                        label {
                            font_size: "14",
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
                        channel
                            .try_send(TreeTask::CloseFolder(item.path.clone()))
                            .unwrap();
                    } else {
                        channel
                            .try_send(TreeTask::OpenFolder(item.path.clone()))
                            .unwrap();
                    }
                };

                let icon = if item.is_opened { "📂" } else { "📁" };

                rsx!(
                    FileExplorerItem {
                        key: "{path}",
                        depth: item.depth,
                        onclick: onclick,
                        label {
                            font_size: "14",
                            max_lines: "1",
                            text_overflow: "ellipsis",
                            "{icon} {name}"
                        }
                    }
                )
            }
        };

    if items.is_empty() {
        render!(
            rect {
                width: "100%",
                height: "100%",
                display: "center",
                direction: "both",
                Button {
                    onclick: open_dialog,
                    label {
                        "Open folder"
                    }
                }
            }
        )
    } else {
        render!(rect {
            width: "100%",
            height: "100%",
            padding: "10",
            VirtualScrollView {
                width: "100%",
                height: "100%",
                length: items.len(),
                item_size: 26.0,
                builder_values: (items, channel),
                direction: "vertical",
                builder: Box::new(tree_builder)
            }
        })
    }
}

#[allow(non_snake_case)]
#[inline_props]
fn FileExplorerItem<'a>(
    cx: Scope<'a>,
    children: Element<'a>,
    onclick: EventHandler<'a, ()>,
    depth: usize,
) -> Element<'a> {
    let status = use_state(cx, || ButtonStatus::Idle);

    let onmouseenter = |_| status.set(ButtonStatus::Hovering);
    let onmouseleave = |_| status.set(ButtonStatus::Idle);

    let background = match status.get() {
        ButtonStatus::Idle => "transparent",
        ButtonStatus::Hovering => "rgb(35, 35, 35)",
    };

    render!(rect {
        onmouseenter: onmouseenter,
        onmouseleave: onmouseleave,
        onclick: move |_| onclick.call(()),
        background: "{background}",
        corner_radius: "5",
        margin: "0 0 0 {depth * 10}",
        direction: "horizontal",
        padding: "4 8",
        height: "26",
        children
    })
}
