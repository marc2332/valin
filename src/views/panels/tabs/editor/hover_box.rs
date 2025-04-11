use freya::prelude::*;

#[allow(non_snake_case)]
#[component]
pub fn HoverBox(content: String) -> Element {
    let height = match content.trim().lines().count() {
        x if x < 2 => 65,
        x if x < 5 => 100,
        x if x < 7 => 135,
        _ => 170,
    };

    rsx!( rect {
        width: "300",
        height: "{height}",
        background: "rgb(60, 60, 60)",
        corner_radius: "6",
        layer: "-50",
        padding: "8",
        shadow: "0 2 10 0 rgb(0, 0, 0, 40)",
        border: "1 solid rgb(45, 45, 45)",
        ScrollView {
            label {
                width: "100%",
                color: "rgb(245, 245, 245)",
                {content}
            }
        }
    })
}
