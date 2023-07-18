use freya::prelude::*;

#[allow(non_snake_case)]
pub fn Divider(cx: Scope) -> Element {
    render!(rect {
        background: "rgb(60, 60, 60)",
        height: "100%",
        width: "2",
    })
}
