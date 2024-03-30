use freya::prelude::*;

#[component]
#[allow(non_snake_case)]
pub fn Sidepanel(children: Element) -> Element {
    rsx!(rect {
        width: "270",
        height: "100%",
        direction: "vertical",
        {children}
    })
}
