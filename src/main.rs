mod commander;
mod controlled_virtual_scroll_view;
mod file_explorer;
mod panels;
mod parser;
mod sidebar;
mod sidepanel;
mod tab;
mod tabs;
mod text_area;
mod use_editable;
mod use_metrics;
mod utils;

use commander::*;
use file_explorer::*;
use freya::prelude::{keyboard::Code, *};
use panels::*;
use sidebar::*;
use sidepanel::*;
use tab::*;
use tabs::code_editor::*;
use tabs::config::*;
use text_area::*;
use utils::*;

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
        ThemeProvider { theme: DARK_THEME, Body {} }
    )
}

#[allow(non_snake_case)]
fn Body(cx: Scope) -> Element {
    let panels_manager = use_state::<PanelsManager>(cx, PanelsManager::new);
    let show_commander = use_state(cx, || false);
    let commands = cx.use_hook(|| {
        vec![Command::new("fs".to_string(), {
            to_owned![panels_manager];
            Box::new(move |size: &str| {
                if let Ok(size) = size.parse::<f32>() {
                    panels_manager.with_mut(|panels_manager| {
                        panels_manager.set_fontsize(size);
                    })
                }
            })
        })]
    });

    let split_panel = move |_| {
        to_owned![panels_manager];
        panels_manager.with_mut(|panels_manager| {
            panels_manager.push_panel(Panel::new());
            panels_manager.set_focused_panel(panels_manager.panels().len() - 1);
        });
    };

    let onsubmit = {
        move |_| {
            panels_manager.with_mut(|panels_manager| {
                panels_manager.set_focused(true);
                show_commander.set(false);
            });
        }
    };

    let onkeydown = move |e: KeyboardEvent| {
        panels_manager.with_mut(|panels_manager| {
            if e.code == Code::Escape {
                if *show_commander.current() {
                    panels_manager.set_focused(true);
                    show_commander.set(false);
                } else {
                    panels_manager.set_focused(false);
                    show_commander.set(true);
                }
            }
        })
    };

    let onglobalclick = |_| {
        show_commander.set(false);
    };

    let panels_len = panels_manager.get().panels().len();
    let panes_width = 100.0 / panels_len as f32;

    render!(
        rect {
            color: "white",
            background: "rgb(20, 20, 20)",
            direction: "horizontal",
            width: "100%",
            height: "100%",
            onkeydown: onkeydown,
            onglobalclick: onglobalclick,
            Sidebar { panels_manager: panels_manager.clone() }
            Divider {}
            Sidepanel { FileExplorer { panels_manager: panels_manager.clone() } }
            Divider {}
            rect {
                direction: "vertical",
                width: "calc(100% - 334)",
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
                    panels_manager.get().panels().iter().enumerate().map(move |(panel_index, panel)| {
                        let is_focused = panels_manager.get().focused_panel() == panel_index;
                        let active_tab_index = panel.active_tab();
                        let panel_background = if is_focused {
                            "rgb(247, 127, 0)"
                        } else {
                            "transparent"
                        };

                        let close_panel = move |_: MouseEvent| {
                            panels_manager.with_mut(|panels_manager| {
                                panels_manager.close_panel(panel_index);
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
                                    ScrollView {
                                        direction: "horizontal",
                                        width: "calc(100% - 55)",
                                        panels_manager.get().panel(panel_index).tabs().iter().enumerate().map(|(i, tab)| {
                                            let is_selected = active_tab_index == Some(i);
                                            let (tab_id, tab_title) = tab.get_data();
                                            rsx!(
                                                Tab {
                                                    key: "{tab_id}",
                                                    onclick: move |_| {
                                                        panels_manager.with_mut(|panels_manager| {
                                                            panels_manager.set_focused_panel(panel_index);
                                                            panels_manager.panel_mut(panel_index).set_active_tab(i);
                                                        });
                                                    },
                                                    onclickclose: move |_| {
                                                        panels_manager.with_mut(|panels_manager| {
                                                            panels_manager.close_editor(panel_index, i);
                                                        });
                                                    },
                                                    value: "{tab_title}",
                                                    is_selected: is_selected
                                                }
                                            )
                                        })
                                    }
                                    Button {
                                        onclick: split_panel,
                                        label {
                                            "Split"
                                        }
                                    }
                                }
                                rect {
                                    height: "calc(100%-50)",
                                    width: "100%",
                                    background: "{panel_background}",
                                    padding: "1.5",
                                    onclick: move |_| {
                                        panels_manager.with_mut(|panels_manager| {
                                            panels_manager.set_focused_panel(panel_index);
                                        });
                                    },
                                    if let Some(active_tab_index) = active_tab_index {
                                        let active_tab = panel.tab(active_tab_index);
                                        let (tab_id, _) = active_tab.get_data();
                                        match active_tab {
                                            PanelTab::TextEditor(_) => {
                                                rsx!(
                                                    CodeEditorTab {
                                                        key: "{tab_id}-{active_tab_index}",
                                                        manager: panels_manager.clone(),
                                                        panel_index: panel_index,
                                                        editor: active_tab_index,
                                                    }
                                                )
                                            }
                                            PanelTab::Config => {
                                                rsx!(
                                                    ConfigTab {
                                                        key: "{tab_id}",
                                                    }
                                                )
                                            }
                                        }
                                    } else {
                                        rsx!(
                                            rect {
                                                display: "center",
                                                width: "100%",
                                                height: "100%",
                                                direction: "both",
                                                background: "rgb(20, 20, 20)",
                                                if panels_len > 1 {
                                                    rsx!(
                                                        Button {
                                                            onclick: close_panel,
                                                            label {
                                                                "Close panel"
                                                            }
                                                        }
                                                    )
                                                } else {
                                                    rsx!(
                                                        label {
                                                            "Coding is fun"
                                                        }
                                                    )
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
