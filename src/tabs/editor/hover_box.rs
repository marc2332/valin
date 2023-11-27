use freya::prelude::*;

#[allow(non_snake_case)]
#[inline_props]
pub fn HoverBox(cx: Scope, content: String) -> Element {
    let height = match content.trim().lines().count() {
        x if x < 2 => 65,
        x if x < 5 => 100,
        x if x < 7 => 135,
        _ => 170,
    };

    render!( rect {
        width: "300",
        height: "{height}",
        background: "rgb(60, 60, 60)",
        corner_radius: "8",
        layer: "-50",
        padding: "10",
        shadow: "0 5 10 0 rgb(0, 0, 0, 50)",
        border: "1 solid rgb(50, 50, 50)",
        ScrollView {
            label {
                width: "100%",
                color: "rgb(245, 245, 245)",
                "{content}"
            }
        }
    })
}
