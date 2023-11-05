use freya::prelude::*;

static LOGO_ENABLED: &str = include_str!("./logo_enabled.svg");
static LOGO_DISABLED: &str = include_str!("./logo_disabled.svg");

#[derive(Props, PartialEq)]
pub struct IconProps {
    #[props(default = "auto".to_string(), into)]
    width: String,
    #[props(default = "auto".to_string(), into)]
    height: String,
    enabled: bool,
}

#[allow(non_snake_case)]
pub fn Logo(cx: Scope<IconProps>) -> Element {
    let width = &cx.props.width;
    let height = &cx.props.height;

    let logo = if cx.props.enabled {
        LOGO_ENABLED
    } else {
        LOGO_DISABLED
    };

    render!(svg {
        width: "{width}",
        height: "{height}",
        svg_content: logo,
    })
}

#[derive(Props)]
pub struct ExpandedIconProps<'a> {
    children: Element<'a>,
}

#[allow(non_snake_case)]
pub fn ExpandedIcon<'a>(cx: Scope<'a, ExpandedIconProps<'a>>) -> Element<'a> {
    render!(
        rect {
            main_align: "center",
            cross_align: "center",
            width: "100%",
            height: "100%",
            &cx.props.children
        }
    )
}
