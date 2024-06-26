use freya::prelude::*;

#[derive(Clone, Copy)]
pub struct KeyboardNavigationCallback(Signal<Option<Box<dyn FnOnce()>>>);

impl KeyboardNavigationCallback {
    /// This will be called after all the other keyboard events have been emitted,
    /// and thus preventing any conflict between them
    pub fn callback(&mut self, ovewrite: bool, cb: impl FnOnce() + 'static) {
        let is_empty = self.0.peek().is_none();
        if ovewrite || is_empty {
            *self.0.write() = Some(Box::new(cb));
        }
    }
}

pub fn use_keyboard_navigation() -> KeyboardNavigationCallback {
    use_context::<KeyboardNavigationCallback>()
}

#[allow(non_snake_case)]
#[component]
pub fn KeyboardNavigationProvider(children: Element) -> Element {
    let mut keyboard_navigation =
        use_context_provider(|| KeyboardNavigationCallback(Signal::new(None)));

    let onkeydown = move |_| {
        if let Some(keyboard_navigation_cb) = keyboard_navigation.0.write().take() {
            (keyboard_navigation_cb)();
        }
    };

    rsx!(
        rect {
            width: "100%",
            height: "100%",
            onkeydown,
            {children}
        }
    )
}
