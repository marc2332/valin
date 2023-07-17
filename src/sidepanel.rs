use freya::prelude::*;

#[inline_props]
#[allow(non_snake_case)]
pub fn Sidepanel<'a>(cx: Scope<'a>, children: Element<'a>) -> Element<'a> {
    render!(rect {
        width: "270",
        height: "100%",
        direction: "vertical",
        children
    })
}
