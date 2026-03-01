use crate::state::{AppState, Channel};
use freya::prelude::*;
use freya::radio::use_radio;

#[derive(PartialEq)]
pub struct Overlay {
    children: Vec<Element>,
}

impl Overlay {
    pub fn new() -> Self {
        Self {
            children: Vec::new(),
        }
    }
}

impl ChildrenExt for Overlay {
    fn get_children(&mut self) -> &mut Vec<Element> {
        &mut self.children
    }
}

impl Component for Overlay {
    fn render(&self) -> impl IntoElement {
        let mut radio_app_state = use_radio::<AppState, Channel>(Channel::Global);

        let on_global_mouse_down = move |_: Event<MouseEventData>| {
            let mut app_state = radio_app_state.write_channel(Channel::Global);
            app_state.focus_previous_view();
        };

        let on_mouse_down = move |e: Event<MouseEventData>| {
            e.stop_propagation();
            e.prevent_default();
        };

        rect()
            .width(Size::fill())
            .height(Size::px(0.))
            .layer(Layer::Overlay)
            .on_global_mouse_down(on_global_mouse_down)
            .child(
                rect()
                    .width(Size::fill())
                    .height(Size::window_percent(100.))
                    .position(Position::new_global())
                    .center()
                    .child(
                        rect()
                            .background((35, 38, 39))
                            .shadow(
                                Shadow::default()
                                    .x(0.)
                                    .y(4.)
                                    .blur(15.)
                                    .spread(8.)
                                    .color((0, 0, 0, 77)),
                            )
                            .corner_radius(12.)
                            .on_mouse_down(on_mouse_down)
                            .width(Size::px(500.))
                            .padding(5.)
                            .children(self.children.clone()),
                    ),
            )
    }
}
