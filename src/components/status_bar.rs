use freya::radio::use_radio;
use freya::{prelude::*, text_edit::TextEditor};

use crate::{
    state::{Channel, EditorSidePanel, EditorView},
    views::panels::tabs::settings::Settings,
};

#[derive(Clone, PartialEq)]
pub struct StatusBar {
    pub focused_view: EditorView,
}

impl Component for StatusBar {
    fn render(&self) -> impl IntoElement {
        let mut radio_app_state = use_radio(Channel::ActiveTab);

        let open_settings = move |_| {
            let mut app_state = radio_app_state.write_channel(Channel::Global);
            Settings::open_with(radio_app_state, &mut app_state);
        };

        let toggle_file_explorer = move |_| {
            let mut app_state = radio_app_state.write_channel(Channel::Global);
            app_state.toggle_side_panel(EditorSidePanel::FileExplorer);
        };

        let app_state = radio_app_state.read();
        let panel = &app_state.panels[app_state.focused_panel];
        let tab_data = if let Some(active_tab) = panel.active_tab {
            app_state
                .tab(&active_tab)
                .as_text_editor()
                .map(|editor_tab| {
                    (
                        (editor_tab.data.cursor_row(), editor_tab.data.cursor_col()),
                        editor_tab.data.language_id,
                    )
                })
        } else {
            None
        };

        rect()
            .horizontal()
            .cross_align(Alignment::Center)
            .content(Content::Flex)
            .padding((0., 6.))
            .expanded()
            .background((8, 8, 12))
            .child(
                rect()
                    .horizontal()
                    .width(Size::flex(1.))
                    .spacing(4.)
                    .child(
                        Button::new()
                            .flat()
                            .compact()
                            .on_press(toggle_file_explorer)
                            .child("📁"),
                    )
                    .child(
                        Button::new()
                            .flat()
                            .compact()
                            .on_press(open_settings)
                            .child("⚙️"),
                    ),
            )
            .maybe_child(tab_data.map(|((row, col), language_id)| {
                rect()
                    .horizontal()
                    .spacing(4.)
                    .child(format!("Ln {}, Col {}", row + 1, col + 1))
                    .child(format!("{}", language_id))
            }))
    }
}
