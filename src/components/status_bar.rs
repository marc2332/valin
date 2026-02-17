use freya::radio::use_radio;
use freya::{prelude::*, text_edit::TextEditor};

use crate::{
    state::{Channel, EditorSidePanel, EditorView},
    views::panels::tabs::{editor::TabEditorUtils, settings::Settings},
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
        let panel = app_state.panel(app_state.focused_panel);
        let tab_data = if let Some(active_tab) = panel.active_tab() {
            app_state
                .tab(&active_tab)
                .as_text_editor()
                .map(|editor_tab| {
                    (
                        (
                            editor_tab.editor.cursor_row(),
                            editor_tab.editor.cursor_col(),
                        ),
                        editor_tab.editor.editor_type(),
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
            .background((13, 16, 17))
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
            .maybe_child(tab_data.map(|((row, col), editor_type)| {
                rect()
                    .horizontal()
                    .spacing(4.)
                    .child(format!("Ln {}, Col {}", row + 1, col + 1))
                    .child(format!("{}", editor_type.language_id()))
            }))
    }
}
