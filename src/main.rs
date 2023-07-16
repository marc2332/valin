mod code_editor;
mod commander;
mod controlled_virtual_scroll_view;
mod file_explorer;
mod file_tab;
mod parser;
mod text_area;
mod use_editable;
mod use_syntax_highlighter;

use code_editor::*;
use file_explorer::*;
use file_tab::*;
use freya::prelude::{keyboard::Code, *};
use text_area::*;
use tokio::fs::read_to_string;
use use_editable::{EditorData, EditorManager, Panel};

use crate::commander::*;

fn main() {
    launch_cfg(
        app,
        LaunchConfig::<()>::builder()
            .with_width(900.0)
            .with_height(600.0)
            .with_title("Editor")
            .build(),
    );
}

fn app(cx: Scope) -> Element {
    use_init_focus(cx);
    render!(
        ThemeProvider {
            theme: DARK_THEME,
            Body {}
        }
    )
}

#[allow(non_snake_case)]
fn Body(cx: Scope) -> Element {
    let theme = use_theme(cx);
    let theme = &theme.read();
    let editor_manager = use_state::<EditorManager>(cx, EditorManager::new);
    let show_commander = use_state(cx, || false);
    let commands = cx.use_hook(|| {
        vec![Command::new("fs".to_string(), {
            to_owned![editor_manager];
            Box::new(move |size: &str| {
                if let Ok(size) = size.parse::<f32>() {
                    editor_manager.with_mut(|editor_manager| {
                        editor_manager.set_fontsize(size);
                    })
                }
            })
        })]
    });

    let open_file = move |_: MouseEvent| {
        to_owned![editor_manager];
        cx.spawn(async move {
            let task = rfd::AsyncFileDialog::new().pick_file();
            let file = task.await;

            if let Some(file) = file {
                let path = file.path();
                let content = read_to_string(&path).await.unwrap();
                editor_manager.with_mut(|editor_manager| {
                    editor_manager.push_editor(
                        EditorData::new(path.to_path_buf(), Rope::from(content), (0, 0)),
                        editor_manager.focused_panel(),
                        true,
                    );
                });
            }
        });
    };

    let create_panel = move |_| {
        to_owned![editor_manager];
        editor_manager.with_mut(|editor_manager| {
            editor_manager.push_panel(Panel::new());
            editor_manager.set_focused_panel(editor_manager.panels().len() - 1);
        });
    };

    let onsubmit = {
        move |_| {
            editor_manager.with_mut(|editor_manager| {
                editor_manager.set_focused(true);
                show_commander.set(false);
            });
        }
    };

    let onkeydown = move |e: KeyboardEvent| {
        editor_manager.with_mut(|editor_manager| {
            if e.code == Code::Escape {
                if *show_commander.current() {
                    editor_manager.set_focused(true);
                    show_commander.set(false);
                } else {
                    editor_manager.set_focused(false);
                    show_commander.set(true);
                }
            }
        })
    };

    let onglobalclick = |_| {
        show_commander.set(false);
    };

    let panes_width = 100.0 / editor_manager.get().panels().len() as f32;

    render!(
        rect {
            color: "white",
            background: "rgb(20, 20, 20)",
            direction: "horizontal",
            width: "100%",
            height: "100%",
            onkeydown: onkeydown,
            onglobalclick: onglobalclick,
            rect {
                direction: "vertical",
                width: "60",
                height: "100%",
                Button {
                    onclick: open_file,
                    label {
                        "Open"
                    }
                }
                Button {
                    onclick: create_panel,
                    label {
                        "Split"
                    }
                }
            }
            rect {
                background: "rgb(100, 100, 100)",
                height: "100%",
                width: "2",
            }
            rect {
                width: "270",
                height: "100%",
                direction: "vertical",
                FileExplorer {

                }
            }
            rect {
                direction: "vertical",
                width: "calc(100% - 332)",
                height: "100%",
                if *show_commander.current(){
                    rsx!(
                        Commander {
                            onsubmit: onsubmit,
                            commands: commands
                        }
                    )
                }
                rect {
                    height: "100%",
                    width: "100%",
                    direction: "horizontal",
                    editor_manager.get().panels().iter().enumerate().map(move |(panel_index, panel)| {
                        let is_focused = editor_manager.get().focused_panel() == panel_index;
                        let active_editor = panel.active_editor();
                        let bg = if is_focused {
                            "rgb(247, 127, 0)"
                        } else {
                            "transparent"
                        };

                        let close_panel = move |_: MouseEvent| {
                            editor_manager.with_mut(|editor_manager| {
                                editor_manager.close_panel(panel_index);
                            });
                        };

                        rsx!(
                            rect {
                                direction: "vertical",
                                height: "100%",
                                width: "{panes_width}%",
                                rect {
                                    direction: "horizontal",
                                    height: "50",
                                    width: "100%",
                                    padding: "2.5",
                                    editor_manager.get().panel(panel_index).editors().iter().enumerate().map(|(i, editor)| {
                                        let is_selected = active_editor == Some(i);
                                        let file_name = editor.path().file_name().unwrap().to_str().unwrap().to_owned();
                                        rsx!(
                                            FileTab {
                                                key: "{i}",
                                                onclick: move |_| {
                                                    editor_manager.with_mut(|editor_manager| {
                                                        editor_manager.set_focused_panel(panel_index);
                                                        editor_manager.panel_mut(panel_index).set_active_editor(i);
                                                    });
                                                },
                                                value: "{file_name}",
                                                is_selected: is_selected
                                            }
                                        )
                                    })
                                }
                                rect {
                                    height: "calc(100%-50)",
                                    width: "100%",
                                    background: "{bg}",
                                    padding: "1.5",
                                    onclick: move |_| {
                                        editor_manager.with_mut(|editor_manager| {
                                            editor_manager.set_focused_panel(panel_index);
                                        });
                                    },
                                    if let Some(active_editor) = active_editor {
                                        rsx!(
                                            Editor {
                                                key: "{active_editor}",
                                                manager: editor_manager,
                                                panel_index: panel_index,
                                                editor: active_editor,
                                            }
                                        )
                                    } else {
                                        rsx!(
                                            rect {
                                                display: "center",
                                                width: "100%",
                                                height: "100%",
                                                direction: "both",
                                                background: "{theme.body.background}",
                                                Button {
                                                    onclick: close_panel,
                                                    label {
                                                        "Close panel"
                                                    }
                                                }
                                            }
                                        )
                                    }
                                }
                            }
                        )
                    })
                }
            }
        }
    )
}
