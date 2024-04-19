use freya::prelude::*;

#[allow(non_snake_case)]
pub fn WelcomeTab() -> Element {
    rsx!(
        rect {
            height: "100%",
            width: "100%",
            background: "rgb(35, 35, 35)",
            padding: "20",
            Link {
                to: "https://github.com/marc2332/freya",
                tooltip: LinkTooltip::None,
                label {
                    "freya source code"
                }
            }
            Link {
                to: "https://github.com/marc2332/freya-editor",
                tooltip: LinkTooltip::None,
                label {
                    "freya-editor source code"
                }
            }
        }
    )
}
