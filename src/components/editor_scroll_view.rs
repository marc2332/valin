use std::ops::Range;

use freya::prelude::*;
use freya::prelude::{dioxus_elements, keyboard::Key, use_applied_theme};

use crate::{
    get_container_size, get_corrected_scroll_position, get_scroll_position_from_cursor,
    get_scrollbar_pos_and_size, is_scrollbar_visible, Axis,
};

pub fn get_scroll_position_from_wheel(
    wheel_movement: f32,
    inner_size: f32,
    viewport_size: f32,
    scroll_position: f32,
) -> i32 {
    if viewport_size >= inner_size {
        return 0;
    }

    let new_position = scroll_position + (wheel_movement * 2.0);

    if new_position >= 0.0 && wheel_movement > 0.0 {
        return 0;
    }

    if new_position <= -(inner_size - viewport_size) && wheel_movement < 0.0 {
        return -(inner_size - viewport_size) as i32;
    }

    new_position as i32
}

/// Properties for the EditorScrollView component.
#[derive(Props, Clone)]
pub struct EditorScrollViewProps<
    Builder: 'static + Clone + Fn(usize, &BuilderArgs) -> Element,
    BuilderArgs: Clone + 'static + PartialEq = (),
> {
    length: usize,
    item_size: f32,
    #[props(default = "100%".to_string(), into)]
    pub height: String,
    #[props(default = "100%".to_string(), into)]
    pub width: String,
    #[props(default = "0".to_string(), into)]
    pub padding: String,
    #[props(default = true, into)]
    pub show_scrollbar: bool,
    pub offset_y: i32,
    pub offset_x: i32,
    pub onscroll: Option<EventHandler<(Axis, i32)>>,

    builder_args: BuilderArgs,
    builder: Builder,
}

impl<BuilderArgs: Clone + PartialEq, Builder: Clone + Fn(usize, &BuilderArgs) -> Element> PartialEq
    for EditorScrollViewProps<Builder, BuilderArgs>
{
    fn eq(&self, other: &Self) -> bool {
        self.length == other.length
            && self.offset_x == other.offset_x
            && self.offset_y == other.offset_y
            && self.item_size == other.item_size
            && self.show_scrollbar == other.show_scrollbar
            && self.builder_args == other.builder_args
    }
}

fn get_render_range(
    viewport_size: f32,
    scroll_position: f32,
    item_size: f32,
    item_length: f32,
) -> Range<usize> {
    let render_index_start = (-scroll_position) / item_size;
    let potentially_visible_length = viewport_size / item_size;
    let remaining_length = item_length - render_index_start;

    let render_index_end = if remaining_length <= potentially_visible_length {
        item_length
    } else {
        render_index_start + potentially_visible_length
    };

    render_index_start as usize..(render_index_end as usize)
}

/// A controlled ScrollView with virtual scrolling.
#[allow(non_snake_case)]
pub fn EditorScrollView<
    Builder: Clone + Fn(usize, &BuilderArgs) -> Element,
    BuilderArgs: Clone + PartialEq,
>(
    props: EditorScrollViewProps<Builder, BuilderArgs>,
) -> Element {
    let mut clicking_shift = use_signal(|| false);
    let mut clicking_alt = use_signal(|| false);
    let mut clicking_scrollbar = use_signal::<Option<(Axis, f64)>>(|| None);
    let scrolled_y = props.offset_y;
    let scrolled_x = props.offset_x;
    let onscroll = props.onscroll.unwrap();
    let mut focus = use_focus();
    let (node_ref, size) = use_node();
    let scrollbar_theme = use_applied_theme!(&None, scroll_bar);

    let padding = &props.padding;
    let user_container_width = &props.width;
    let user_container_height = &props.height;
    let show_scrollbar = props.show_scrollbar;
    let items_length = props.length;
    let items_size = props.item_size;

    let inner_size = items_size + (items_size * items_length as f32);

    let vertical_scrollbar_is_visible =
        is_scrollbar_visible(show_scrollbar, inner_size, size.area.height());
    let horizontal_scrollbar_is_visible =
        is_scrollbar_visible(show_scrollbar, size.inner.width, size.area.width());

    let container_width = get_container_size(vertical_scrollbar_is_visible, &scrollbar_theme.size);
    let container_height =
        get_container_size(horizontal_scrollbar_is_visible, &scrollbar_theme.size);

    let corrected_scrolled_y =
        get_corrected_scroll_position(inner_size, size.area.height(), scrolled_y as f32);
    let corrected_scrolled_x =
        get_corrected_scroll_position(size.inner.width, size.area.width(), scrolled_x as f32);

    let (scrollbar_y, scrollbar_height) =
        get_scrollbar_pos_and_size(inner_size, size.area.height(), corrected_scrolled_y);
    let (scrollbar_x, scrollbar_width) =
        get_scrollbar_pos_and_size(size.inner.width, size.area.width(), corrected_scrolled_x);

    // Moves the Y axis when the user scrolls in the container
    let onwheel = move |e: WheelEvent| {
        let speed_multiplier = if *clicking_alt.read() {
            SCROLL_SPEED_MULTIPLIER
        } else {
            1.0
        };

        if !*clicking_shift.read() {
            let wheel_y = e.get_delta_y() as f32 * speed_multiplier;

            let scroll_position_y = get_scroll_position_from_wheel(
                wheel_y,
                inner_size,
                size.area.height(),
                scrolled_y as f32,
            );

            onscroll.call((Axis::Y, scroll_position_y));
        }

        let wheel_x = if *clicking_shift.read() {
            e.get_delta_y() as f32
        } else {
            e.get_delta_x() as f32
        } * speed_multiplier;

        let scroll_position_x = get_scroll_position_from_wheel(
            wheel_x,
            size.inner.width,
            size.area.width(),
            corrected_scrolled_x,
        );

        onscroll.call((Axis::X, scroll_position_x));

        focus.focus();
    };

    // Drag the scrollbars
    let onmouseover = move |e: MouseEvent| {
        let clicking_scrollbar = clicking_scrollbar.read();

        if let Some((Axis::Y, y)) = *clicking_scrollbar {
            let coordinates = e.get_element_coordinates();
            let cursor_y = coordinates.y - y - size.area.min_y() as f64;

            let scroll_position =
                get_scroll_position_from_cursor(cursor_y as f32, inner_size, size.area.height());

            onscroll.call((Axis::Y, scroll_position))
        } else if let Some((Axis::X, x)) = *clicking_scrollbar {
            let coordinates = e.get_element_coordinates();
            let cursor_x = coordinates.x - x - size.area.min_x() as f64;

            let scroll_position = get_scroll_position_from_cursor(
                cursor_x as f32,
                size.inner.width,
                size.area.width(),
            );

            onscroll.call((Axis::X, scroll_position))
        }

        if clicking_scrollbar.is_some() {
            focus.focus();
        }
    };

    let onkeydown = move |e: KeyboardEvent| {
        match &e.key {
            Key::Shift => {
                clicking_shift.set(true);
            }
            Key::Alt => {
                clicking_alt.set(true);
            }
            _ => {
                // TODO: Support other keys with `manage_key_event`
            }
        };
    };

    let onkeyup = move |e: KeyboardEvent| {
        if e.key == Key::Shift {
            clicking_shift.set(false);
        } else if e.key == Key::Alt {
            clicking_alt.set(false);
        }
    };

    // Mark the Y axis scrollbar as the one being dragged
    let onmousedown_y = move |e: MouseEvent| {
        let coordinates = e.get_element_coordinates();
        *clicking_scrollbar.write() = Some((Axis::Y, coordinates.y));
    };

    // Mark the X axis scrollbar as the one being dragged
    let onmousedown_x = move |e: MouseEvent| {
        let coordinates = e.get_element_coordinates();
        *clicking_scrollbar.write() = Some((Axis::X, coordinates.x));
    };

    // Unmark any scrollbar
    let onclick = move |_: MouseEvent| {
        if clicking_scrollbar.read().is_some() {
            *clicking_scrollbar.write() = None;
        }
    };

    let horizontal_scrollbar_size = if horizontal_scrollbar_is_visible {
        &scrollbar_theme.size
    } else {
        "0"
    };
    let vertical_scrollbar_size = if vertical_scrollbar_is_visible {
        &scrollbar_theme.size
    } else {
        "0"
    };

    // Calculate from what to what items must be rendered
    let render_range = get_render_range(
        size.area.height(),
        corrected_scrolled_y,
        items_size,
        items_length as f32,
    );

    let children = use_memo(use_reactive(
        &(render_range, props.builder_args),
        move |(render_range, builder_args)| {
            rsx!({ render_range.map(|i| (props.builder)(i, &builder_args)) })
        },
    ));

    let is_scrolling_x = clicking_scrollbar
        .read()
        .as_ref()
        .map(|f| f.0 == Axis::X)
        .unwrap_or_default();
    let is_scrolling_y = clicking_scrollbar
        .read()
        .as_ref()
        .map(|f| f.0 == Axis::Y)
        .unwrap_or_default();

    let offset_y_min = (-corrected_scrolled_y / items_size).floor() * items_size;
    let offset_y = -corrected_scrolled_y - offset_y_min;

    rsx!(
        rect {
            overflow: "clip",
            direction: "horizontal",
            width: "{user_container_width}",
            height: "{user_container_height}",
            onclick: onclick,
            onglobalmouseover: onmouseover,
            onkeydown: onkeydown,
            onkeyup: onkeyup,
            rect {
                direction: "vertical",
                width: "{container_width}",
                height: "{container_height}",
                rect {
                    overflow: "clip",
                    padding: "{padding}",
                    height: "100%",
                    width: "100%",
                    direction: "vertical",
                    offset_x: "{corrected_scrolled_x}",
                    reference: node_ref,
                    onwheel: onwheel,
                    offset_y: "{-offset_y}",
                    {children}
                }
                ScrollBar {
                    width: "100%",
                    height: "{horizontal_scrollbar_size}",
                    offset_x: "{scrollbar_x}",
                    clicking_scrollbar: is_scrolling_x,
                    ScrollThumb {
                        clicking_scrollbar: is_scrolling_x,
                        onmousedown: onmousedown_x,
                        width: "{scrollbar_width}",
                        height: "100%",
                    },
                }
            }
            ScrollBar {
                width: "{vertical_scrollbar_size}",
                height: "100%",
                offset_y: "{scrollbar_y}",
                clicking_scrollbar: is_scrolling_y,
                ScrollThumb {
                    clicking_scrollbar: is_scrolling_y,
                    onmousedown: onmousedown_y,
                    width: "100%",
                    height: "{scrollbar_height}",
                }
            }
        }
    )
}
