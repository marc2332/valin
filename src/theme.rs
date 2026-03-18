/// GitHub Dark Default color theme for Valin.
use freya::code_editor::{EditorTheme, SyntaxTheme};
use freya::prelude::*;

pub fn github_dark_theme() -> Theme {
    let mut theme = Theme {
        name: "github_dark",
        colors: ColorsSheet {
            primary: Color::from_rgb(47, 129, 247),
            secondary: Color::from_rgb(31, 111, 235),
            tertiary: Color::from_rgb(88, 166, 255),

            success: Color::from_rgb(63, 185, 80),
            warning: Color::from_rgb(210, 153, 34),
            error: Color::from_rgb(248, 81, 73),
            info: Color::from_rgb(88, 166, 255),

            background: Color::from_rgb(8, 8, 12),
            surface_primary: Color::from_rgb(13, 17, 23),
            surface_secondary: Color::from_rgb(22, 27, 34),
            surface_tertiary: Color::from_rgb(33, 38, 45),
            surface_inverse: Color::from_rgb(240, 246, 252),
            surface_inverse_secondary: Color::from_rgb(201, 209, 217),
            surface_inverse_tertiary: Color::from_rgb(177, 186, 196),

            border: Color::from_rgb(33, 38, 45),
            border_focus: Color::from_rgb(47, 129, 247),
            border_disabled: Color::from_rgb(22, 27, 34),

            text_primary: Color::from_rgb(230, 237, 243),
            text_secondary: Color::from_rgb(125, 133, 144),
            text_placeholder: Color::from_rgb(110, 118, 129),
            text_inverse: Color::from_rgb(1, 4, 9),
            text_highlight: Color::from_rgb(88, 166, 255),

            hover: Color::from_rgb(33, 38, 45),
            focus: Color::from_rgb(31, 111, 235),
            active: Color::from_rgb(22, 27, 34),
            disabled: Color::from_rgb(13, 17, 23),

            overlay: Color::from_af32rgb(0.5, 8, 8, 12),
            shadow: Color::from_af32rgb(0.6, 8, 8, 12),
        },
        ..DARK_THEME
    };

    theme.scrollbar.background = Color::TRANSPARENT.into();
    theme.scrollbar.thumb_background = Color::from_rgb(72, 79, 88).into();
    theme.scrollbar.hover_thumb_background = Color::from_rgb(110, 118, 129).into();
    theme.scrollbar.active_thumb_background = Color::from_rgb(139, 148, 158).into();
    theme.scrollbar.size = 8.0f32.into();

    theme
}

pub const GITHUB_DARK_EDITOR_THEME: EditorTheme = EditorTheme {
    background: Color::from_rgb(13, 17, 23),
    gutter_selected: Color::from_rgb(230, 237, 243),
    gutter_unselected: Color::from_rgb(139, 148, 158),
    line_selected_background: Color::from_rgb(13, 17, 23),
    cursor: Color::from_rgb(230, 237, 243),
    highlight: Color::from_af32rgb(0.4, 56, 139, 253),
    text: Color::from_rgb(230, 237, 243),
    whitespace: Color::from_af32rgb(0.2, 110, 118, 129),
};

pub const GITHUB_DARK_SYNTAX_THEME: SyntaxTheme = SyntaxTheme {
    text: Color::from_rgb(230, 237, 243),
    whitespace: Color::from_af32rgb(0.2, 110, 118, 129),
    comment: Color::from_rgb(139, 148, 158),
    keyword: Color::from_rgb(255, 110, 100),
    constant: Color::from_rgb(100, 180, 255),
    boolean: Color::from_rgb(100, 180, 255),
    number: Color::from_rgb(100, 180, 255),
    string: Color::from_rgb(140, 200, 255),
    string_escape: Color::from_rgb(100, 180, 255),
    string_special: Color::from_rgb(140, 200, 255),
    function: Color::from_rgb(200, 150, 255),
    function_macro: Color::from_rgb(200, 150, 255),
    function_method: Color::from_rgb(200, 150, 255),
    type_: Color::from_rgb(255, 150, 65),
    constructor: Color::from_rgb(255, 150, 65),
    variable: Color::from_rgb(255, 150, 65),
    variable_builtin: Color::from_rgb(100, 180, 255),
    variable_parameter: Color::from_rgb(230, 237, 243),
    tag: Color::from_rgb(110, 225, 120),
    attribute: Color::from_rgb(100, 180, 255),
    module: Color::from_rgb(255, 150, 65),
    label: Color::from_rgb(100, 180, 255),
    property: Color::from_rgb(100, 180, 255),
    operator: Color::from_rgb(230, 237, 243),
    punctuation: Color::from_rgb(230, 237, 243),
    punctuation_bracket: Color::from_rgb(230, 237, 243),
    punctuation_delimiter: Color::from_rgb(230, 237, 243),
    punctuation_special: Color::from_rgb(255, 110, 100),
    escape: Color::from_rgb(100, 180, 255),
    text_literal: Color::from_rgb(230, 237, 243),
    text_reference: Color::from_rgb(100, 180, 255),
    text_title: Color::from_rgb(100, 180, 255),
    text_uri: Color::from_rgb(140, 200, 255),
    text_emphasis: Color::from_rgb(230, 237, 243),
};
