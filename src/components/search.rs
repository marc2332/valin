use freya::prelude::*;

use crate::Overlay;

#[component]
pub fn Search() -> Element {
    rsx!(
        Overlay {
            label {
                "Search"
            }
        }
    )
}
