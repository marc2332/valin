use freya::prelude::*;

#[allow(non_snake_case)]
pub fn ConfigTab() -> Element {
    rsx!(
        rect {
            height: "100%",
            width: "100%",
            background: "rgb(35, 35, 35)",
            padding: "20",
            label {
                "Nothing to see here yet"
            }
        }
    )
}
