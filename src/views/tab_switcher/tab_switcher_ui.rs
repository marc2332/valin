use freya::prelude::*;

use crate::{components::Overlay, state::RadioAppState};

const ITEM_HEIGHT: f32 = 32.;
const MAX_LIST_HEIGHT: f32 = 420.;

#[derive(PartialEq)]
pub struct TabSwitcher {
    pub radio_app_state: RadioAppState,
}

impl Component for TabSwitcher {
    fn render(&self) -> impl IntoElement {
        let app_state = self.radio_app_state.read();
        let rows: Vec<TabSwitcherRow> = app_state
            .tab_switcher
            .iter()
            .flat_map(|switcher| {
                switcher
                    .order
                    .iter()
                    .enumerate()
                    .filter_map(|(index, tab_id)| {
                        let data = app_state.tabs.get(tab_id)?.get_data();
                        Some(TabSwitcherRow {
                            key_id: index,
                            title: data.title.clone(),
                            icon: data.icon.clone(),
                            is_selected: index == switcher.selected,
                        })
                    })
            })
            .collect();

        let list_height = (rows.len() as f32 * ITEM_HEIGHT).clamp(ITEM_HEIGHT, MAX_LIST_HEIGHT);
        let body: Element = if rows.is_empty() {
            rect()
                .height(Size::px(ITEM_HEIGHT))
                .padding((8., 6.))
                .child(label().text("No tabs to switch to"))
                .into()
        } else {
            rect()
                .children(rows.into_iter().map(Into::into).collect::<Vec<_>>())
                .into()
        };

        Overlay::new().child(
            rect().padding(4.).child(
                ScrollView::new()
                    .height(Size::px(list_height))
                    .scroll_with_arrows(false)
                    .child(body),
            ),
        )
    }
}

#[derive(Clone, PartialEq)]
struct TabSwitcherRow {
    key_id: usize,
    title: String,
    icon: Option<Bytes>,
    is_selected: bool,
}

impl Component for TabSwitcherRow {
    fn render(&self) -> impl IntoElement {
        let background = if self.is_selected {
            Color::from((22, 27, 34))
        } else {
            Color::TRANSPARENT
        };

        rect()
            .key(&self.key_id)
            .background(background)
            .corner_radius(8.)
            .padding((8., 6.))
            .width(Size::fill())
            .height(Size::px(ITEM_HEIGHT))
            .horizontal()
            .cross_align(Alignment::Center)
            .maybe_child(self.icon.clone().map(|bytes| {
                svg(bytes)
                    .width(Size::px(14.))
                    .height(Size::px(14.))
                    .fill(Color::from_rgb(180, 180, 180))
                    .margin((0., 6., 0., 0.))
                    .into_element()
            }))
            .child(
                label()
                    .max_lines(1)
                    .text_overflow(TextOverflow::Ellipsis)
                    .text(self.title.clone()),
            )
    }
}
