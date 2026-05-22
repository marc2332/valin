use std::path::PathBuf;

use freya::prelude::*;
use ignore::WalkBuilder;

use crate::{
    components::Overlay,
    state::{Channel, RadioAppState},
    views::panels::tabs::editor::EditorTab,
};

const ITEM_HEIGHT: f32 = 28.;

#[derive(Clone, PartialEq)]
struct FoundFile {
    path: PathBuf,
    display: String,
}

#[derive(PartialEq)]
pub struct FileSearch {
    pub radio_app_state: RadioAppState,
}

impl Component for FileSearch {
    fn render(&self) -> impl IntoElement {
        let mut radio_app_state = self.radio_app_state;
        let value = use_state(String::new);
        let mut selected = use_state(|| 0usize);

        let files_task = use_future(move || async move {
            let folders: Vec<PathBuf> = radio_app_state
                .read()
                .file_explorer
                .folders
                .iter()
                .map(|item| item.path().clone())
                .collect();
            smol::unblock(move || discover_files(&folders)).await
        });

        let query = value.read().to_lowercase();
        let filtered_files: Vec<FoundFile> = match &*files_task.state() {
            FutureState::Fulfilled(files) => {
                if query.is_empty() {
                    files.clone()
                } else {
                    files
                        .iter()
                        .filter(|f| f.display.to_lowercase().contains(&query))
                        .cloned()
                        .collect()
                }
            }
            _ => Vec::new(),
        };

        let filtered_count = filtered_files.len();
        let list_height = (filtered_count as f32 * ITEM_HEIGHT).clamp(ITEM_HEIGHT, 380.);
        let selected_path = filtered_files.get(*selected.read()).map(|f| f.path.clone());

        let on_submit = move |_: String| {
            let Some(path) = selected_path.clone() else {
                return;
            };
            let transport = radio_app_state.read().default_transport.clone();
            let mut app_state = radio_app_state.write_channel(Channel::Global);
            EditorTab::open_with(radio_app_state, &mut app_state, path, transport.as_read());
        };

        let onkeydown = move |e: Event<KeyboardEventData>| {
            e.stop_propagation();
            if filtered_count == 0 {
                return;
            }
            let current = *selected.read();
            match e.code {
                Code::ArrowDown => selected.set((current + 1) % filtered_count),
                Code::ArrowUp => selected.set((current + filtered_count - 1) % filtered_count),
                _ => {}
            }
        };

        use_side_effect(move || {
            let _ = value.read();
            selected.set_if_modified(0);
        });

        Overlay::new().child(
            rect()
                .on_key_down(onkeydown)
                .spacing(5.)
                .child(
                    Input::new(value)
                        .width(Size::fill())
                        .auto_focus(true)
                        .inner_margin(12.)
                        .placeholder("Search files...")
                        .on_submit(on_submit)
                        .on_pre_key_down(|e: Event<KeyboardEventData>| match e.code {
                            Code::ArrowUp | Code::ArrowDown => false,
                            _ => match &e.key {
                                Key::Named(NamedKey::Enter) | Key::Named(NamedKey::Escape) => true,
                                Key::Named(NamedKey::Tab) => false,
                                _ => {
                                    e.stop_propagation();
                                    e.prevent_default();
                                    true
                                }
                            },
                        }),
                )
                .child(if filtered_count == 0 {
                    rect()
                        .height(Size::px(ITEM_HEIGHT))
                        .padding((8., 6.))
                        .child("No files found")
                } else {
                    rect().height(Size::px(list_height)).child(
                        VirtualScrollView::new_with_data(
                            (filtered_files, selected, radio_app_state),
                            file_search_item_builder,
                        )
                        .length(filtered_count)
                        .item_size(ITEM_HEIGHT)
                        .scroll_with_arrows(false),
                    )
                }),
        )
    }
}

fn file_search_item_builder(
    index: usize,
    (files, selected, radio_app_state): &(Vec<FoundFile>, State<usize>, RadioAppState),
) -> Element {
    let file = files[index].clone();
    let is_selected = *selected.read() == index;
    let mut radio_app_state = *radio_app_state;

    let icon_svg = radio_app_state
        .read()
        .file_icons
        .get_file(&file.path)
        .svg
        .clone();

    let on_press = {
        let path = file.path.clone();
        move |_: Event<PressEventData>| {
            let transport = radio_app_state.read().default_transport.clone();
            let mut app_state = radio_app_state.write_channel(Channel::Global);
            EditorTab::open_with(
                radio_app_state,
                &mut app_state,
                path.clone(),
                transport.as_read(),
            );
        }
    };

    let display = file.display;
    FileSearchOption {
        key_id: display.clone(),
        text: display,
        icon: icon_svg,
        is_selected,
        on_press: on_press.into(),
    }
    .into()
}

#[derive(PartialEq, Clone)]
struct FileSearchOption {
    key_id: String,
    text: String,
    icon: Bytes,
    is_selected: bool,
    on_press: EventHandler<Event<PressEventData>>,
}

impl Component for FileSearchOption {
    fn render_key(&self) -> DiffKey {
        DiffKey::from(&self.key_id)
    }

    fn render(&self) -> impl IntoElement {
        let background = if self.is_selected {
            Color::from((22, 27, 34))
        } else {
            Color::TRANSPARENT
        };

        rect()
            .background(background)
            .padding((8., 6.))
            .width(Size::fill())
            .height(Size::px(ITEM_HEIGHT))
            .corner_radius(8.)
            .horizontal()
            .cross_align(Alignment::Center)
            .on_press(self.on_press.clone())
            .child(
                svg(self.icon.clone())
                    .width(Size::px(14.))
                    .height(Size::px(14.))
                    .fill(Color::from_rgb(180, 180, 180))
                    .margin((0., 6., 0., 0.)),
            )
            .child(
                label()
                    .max_lines(1)
                    .text_overflow(TextOverflow::Ellipsis)
                    .text(self.text.clone()),
            )
    }
}

fn discover_files(folders: &[PathBuf]) -> Vec<FoundFile> {
    let mut all_files = Vec::new();
    for folder in folders {
        let root_label = folder
            .file_name()
            .and_then(|s| s.to_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| folder.display().to_string());

        let walker = WalkBuilder::new(folder).hidden(false).build();
        for entry in walker.flatten() {
            if !entry.file_type().is_some_and(|t| t.is_file()) {
                continue;
            }
            let path = entry.into_path();
            let relative = path
                .strip_prefix(folder)
                .map(|p| p.to_string_lossy().into_owned())
                .unwrap_or_else(|_| path.display().to_string());
            let display = format!("{root_label}/{relative}");
            all_files.push(FoundFile { path, display });
        }
    }
    all_files
}
