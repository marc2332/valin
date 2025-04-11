use std::rc::Rc;

use dioxus_radio::hooks::{ChannelSelection, DataReducer};
use freya::{
    events::{Code, Key, KeyboardData, Modifiers, MouseData},
    prelude::{Readable, Signal, Writable},
};
use freya_hooks::EditableEvent;

use crate::views::panels::tabs::editor::AppStateEditorUtils;

use super::{AppState, Channel, EditorView, TabId};

pub struct EditorAction {
    pub tab_id: TabId,
    pub data: EditorActionData,
}

#[derive(Debug)]
pub enum EditorActionData {
    KeyUp {
        data: Rc<KeyboardData>,
    },
    KeyDown {
        data: Rc<KeyboardData>,
        scroll_offsets: Signal<(i32, i32)>,
        line_height: f32,
        lines_len: usize,
    },
    Click,
    MouseDown {
        data: Rc<MouseData>,
        line_index: usize,
    },
    MouseMove {
        data: Rc<MouseData>,
        line_index: usize,
    },
}

impl DataReducer for AppState {
    type Action = EditorAction;
    type Channel = Channel;

    fn reduce(
        &mut self,
        EditorAction { tab_id, data }: Self::Action,
    ) -> ChannelSelection<Self::Channel> {
        let (panel_index, panel) = self
            .panels
            .iter()
            .enumerate()
            .find(|(_, panel)| panel.tabs.contains(&tab_id))
            .unwrap();
        let is_panels_view_focused = self.focused_view() == EditorView::Panels;
        let is_panel_focused = self.focused_panel() == panel_index;
        let is_editor_focused = is_panel_focused && panel.active_tab() == Some(tab_id);

        match data {
            EditorActionData::MouseMove { data, line_index }
                if is_editor_focused && is_panel_focused =>
            {
                let editor_tab = self.editor_tab_mut(tab_id);
                editor_tab
                    .editor
                    .process_event(&EditableEvent::MouseMove(data, line_index));

                ChannelSelection::Silence
            }
            EditorActionData::MouseDown { data, line_index } => {
                let mut channel = ChannelSelection::Select(Channel::follow_tab(tab_id));

                let editor_tab = self.editor_tab_mut(tab_id);
                editor_tab
                    .editor
                    .process_event(&EditableEvent::MouseDown(data, line_index));

                if !is_editor_focused {
                    self.focus_panel(panel_index);
                    self.panel_mut(panel_index).set_active_tab(tab_id);
                    channel.select(Channel::AllTabs);
                }

                if !is_panels_view_focused {
                    self.focus_view(EditorView::Panels);
                    channel.select(Channel::Global)
                }

                channel
            }
            EditorActionData::Click => {
                let editor_tab = self.editor_tab_mut(tab_id);
                editor_tab.editor.process_event(&EditableEvent::Click);
                ChannelSelection::Silence
            }
            EditorActionData::KeyUp { data } if is_editor_focused && is_panel_focused => {
                let editor_tab = self.editor_tab_mut(tab_id);
                editor_tab.editor.process_event(&EditableEvent::KeyUp(data));
                ChannelSelection::Select(Channel::follow_tab(tab_id))
            }
            EditorActionData::KeyDown {
                data,
                mut scroll_offsets,
                line_height,
                lines_len,
            } if is_editor_focused && is_panel_focused => {
                const LINES_JUMP_ALT: usize = 5;
                const LINES_JUMP_CONTROL: usize = 3;

                let lines_jump = (line_height * LINES_JUMP_ALT as f32).ceil() as i32;
                let min_height = -(lines_len as f32 * line_height) as i32;
                let max_height = 0; // TODO, this should be the height of the viewport
                let current_scroll = scroll_offsets.read().1;

                let events = match &data.key {
                    Key::ArrowUp if data.modifiers.contains(Modifiers::ALT) => {
                        let jump = (current_scroll + lines_jump).clamp(min_height, max_height);
                        scroll_offsets.write().1 = jump;
                        (0..LINES_JUMP_ALT)
                            .map(|_| EditableEvent::KeyDown(data.clone()))
                            .collect::<Vec<EditableEvent>>()
                    }
                    Key::ArrowDown if data.modifiers.contains(Modifiers::ALT) => {
                        let jump = (current_scroll - lines_jump).clamp(min_height, max_height);
                        scroll_offsets.write().1 = jump;
                        (0..LINES_JUMP_ALT)
                            .map(|_| EditableEvent::KeyDown(data.clone()))
                            .collect::<Vec<EditableEvent>>()
                    }
                    Key::ArrowDown | Key::ArrowUp
                        if data.modifiers.contains(Modifiers::CONTROL) =>
                    {
                        (0..LINES_JUMP_CONTROL)
                            .map(|_| EditableEvent::KeyDown(data.clone()))
                            .collect::<Vec<EditableEvent>>()
                    }
                    _ if data.code == Code::Escape
                        || data.modifiers.contains(Modifiers::ALT)
                        || (data.modifiers.contains(Modifiers::CONTROL)
                            && data.code == Code::KeyS) =>
                    {
                        Vec::new()
                    }
                    _ => {
                        vec![EditableEvent::KeyDown(data.clone())]
                    }
                };

                let no_changes = events.is_empty();

                let editor_tab = self.editor_tab_mut(tab_id);
                for event in events {
                    editor_tab.editor.process_event(&event);
                }

                if no_changes {
                    ChannelSelection::Silence
                } else {
                    ChannelSelection::Select(Channel::follow_tab(tab_id))
                }
            }
            _ => ChannelSelection::Silence,
        }
    }
}
