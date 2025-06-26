use freya::prelude::*;

#[component]
#[allow(non_snake_case)]
pub fn Sidepanel(children: Element) -> Element {
    rsx!(rect {
        width: "fill",
        height: "100%",
        direction: "vertical",
        {children}
    })
}
