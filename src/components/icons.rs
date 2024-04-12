use freya::prelude::*;

static LOGO_ENABLED: &str = include_str!("../icons/logo_enabled.svg");
static LOGO_DISABLED: &str = include_str!("../icons/logo_disabled.svg");

#[derive(Props, PartialEq, Clone)]
pub struct IconProps {
    #[props(default = "auto".to_string(), into)]
    width: String,
    #[props(default = "auto".to_string(), into)]
    height: String,
    enabled: bool,
}

#[allow(non_snake_case)]
pub fn Logo(props: IconProps) -> Element {
    let width = &props.width;
    let height = &props.height;

    let logo = if props.enabled {
        LOGO_ENABLED
    } else {
        LOGO_DISABLED
    };

    rsx!(svg {
        width: "{width}",
        height: "{height}",
        svg_content: logo,
    })
}

#[derive(Props, Clone, PartialEq)]
pub struct ExpandedIconProps {
    children: Element,
}

#[allow(non_snake_case)]
pub fn ExpandedIcon(props: ExpandedIconProps) -> Element {
    rsx!(
        rect {
            main_align: "center",
            cross_align: "center",
            width: "100%",
            height: "100%",
            {props.children}
        }
    )
}
