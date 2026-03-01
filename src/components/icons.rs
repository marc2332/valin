use freya::prelude::*;

static LOGO_ENABLED: &[u8] = include_bytes!("../icons/logo_enabled.svg");
static LOGO_DISABLED: &[u8] = include_bytes!("../icons/logo_disabled.svg");

#[derive(Clone, PartialEq)]
pub struct Logo {
    pub width: f32,
    pub height: f32,
    pub enabled: bool,
}

impl Default for Logo {
    fn default() -> Self {
        Self {
            width: 100.0,
            height: 100.0,
            enabled: true,
        }
    }
}

impl Component for Logo {
    fn render(&self) -> impl IntoElement {
        let logo = if self.enabled {
            LOGO_ENABLED
        } else {
            LOGO_DISABLED
        };

        svg(Bytes::from_static(logo))
            .width(Size::px(self.width))
            .height(Size::px(self.height))
    }
}
