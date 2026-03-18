/// GitHub Dark Default color theme for Valin.
///
/// Colors sourced from primer/github-vscode-theme (v6.3.5)
/// using @primer/primitives@7.10.0 dark color tokens,
/// shifted one tier darker for a deeper feel.
use freya::code_editor::{EditorTheme, SyntaxTheme};
use freya::prelude::*;

// ---------------------------------------------------------------------------
// Freya component theme (buttons, scrollbars, inputs, sidebar, etc.)
// ---------------------------------------------------------------------------

pub fn github_dark_theme() -> Theme {
    let mut theme = Theme {
        name: "github_dark",
        colors: ColorsSheet {
            // Brand / Accent
            primary: Color::from_rgb(47, 129, 247), // #2f81f7 accent.fg
            secondary: Color::from_rgb(31, 111, 235), // #1f6feb accent.emphasis
            tertiary: Color::from_rgb(88, 166, 255), // #58a6ff scale.blue[3]

            // Status / Semantic
            success: Color::from_rgb(63, 185, 80),  // #3fb950
            warning: Color::from_rgb(210, 153, 34), // #d29922
            error: Color::from_rgb(248, 81, 73),    // #f85149
            info: Color::from_rgb(88, 166, 255),    // #58a6ff

            // Surfaces / Backgrounds  (shifted one tier darker)
            background: Color::from_rgb(1, 4, 9), // #010409 canvas.inset
            surface_primary: Color::from_rgb(13, 17, 23), // #0d1117 canvas.default
            surface_secondary: Color::from_rgb(22, 27, 34), // #161b22 canvas.overlay
            surface_tertiary: Color::from_rgb(1, 4, 9), // #010409
            surface_inverse: Color::from_rgb(240, 246, 252), // #f0f6fc
            surface_inverse_secondary: Color::from_rgb(201, 209, 217), // #c9d1d9
            surface_inverse_tertiary: Color::from_rgb(177, 186, 196), // #b1bac4

            // Borders
            border: Color::from_rgb(33, 38, 45), // #21262d border.muted (darker)
            border_focus: Color::from_rgb(47, 129, 247), // #2f81f7 accent.fg
            border_disabled: Color::from_rgb(22, 27, 34), // #161b22

            // Text / Content
            text_primary: Color::from_rgb(230, 237, 243), // #e6edf3 fg.default
            text_secondary: Color::from_rgb(125, 133, 144), // #7d8590 fg.muted
            text_placeholder: Color::from_rgb(110, 118, 129), // #6e7681 fg.subtle
            text_inverse: Color::from_rgb(1, 4, 9),       // #010409
            text_highlight: Color::from_rgb(88, 166, 255), // #58a6ff

            // States / Interaction  (shifted darker)
            hover: Color::from_rgb(33, 38, 45),    // #21262d
            focus: Color::from_rgb(31, 111, 235),  // #1f6feb accent.emphasis
            active: Color::from_rgb(22, 27, 34),   // #161b22
            disabled: Color::from_rgb(13, 17, 23), // #0d1117

            // Utility
            overlay: Color::from_af32rgb(0.5, 1, 4, 9),
            shadow: Color::from_af32rgb(0.6, 1, 4, 9),
        },
        ..DARK_THEME
    };

    // Scrollbar: grey-ish thumb on transparent track
    theme.scrollbar.background = Color::TRANSPARENT.into();
    theme.scrollbar.thumb_background = Color::from_rgb(72, 79, 88).into(); // #484F58
    theme.scrollbar.hover_thumb_background = Color::from_rgb(110, 118, 129).into(); // #6E7681
    theme.scrollbar.active_thumb_background = Color::from_rgb(139, 148, 158).into(); // #8B949E
    theme.scrollbar.size = 8.0f32.into();

    theme
}

// ---------------------------------------------------------------------------
// Code editor chrome (gutter, cursor, selection, background)
// ---------------------------------------------------------------------------

pub const GITHUB_DARK_EDITOR_THEME: EditorTheme = EditorTheme {
    background: Color::from_rgb(13, 17, 23), // #0d1117 slightly lighter editor bg
    gutter_selected: Color::from_rgb(230, 237, 243), // #e6edf3 fg.default
    gutter_unselected: Color::from_rgb(139, 148, 158), // #8b949e
    line_selected_background: Color::from_rgb(13, 17, 23), // #0d1117
    cursor: Color::from_rgb(230, 237, 243),  // #e6edf3
    highlight: Color::from_af32rgb(0.4, 56, 139, 253), // rgba(56,139,253,0.4) selection
    text: Color::from_rgb(230, 237, 243),    // #e6edf3
    whitespace: Color::from_af32rgb(0.2, 110, 118, 129),
};

// ---------------------------------------------------------------------------
// Syntax highlighting (tree-sitter token colors)
// ---------------------------------------------------------------------------

pub const GITHUB_DARK_SYNTAX_THEME: SyntaxTheme = SyntaxTheme {
    // Defaults
    text: Color::from_rgb(230, 237, 243), // #e6edf3 fg.default
    whitespace: Color::from_af32rgb(0.2, 110, 118, 129),

    // Comments
    comment: Color::from_rgb(139, 148, 158), // #8b949e scale.gray[3]

    // Keywords & storage
    keyword: Color::from_rgb(255, 123, 114), // #ff7b72 scale.red[3]

    // Constants & literals
    constant: Color::from_rgb(121, 192, 255), // #79c0ff scale.blue[2]
    boolean: Color::from_rgb(121, 192, 255),  // #79c0ff
    number: Color::from_rgb(121, 192, 255),   // #79c0ff

    // Strings
    string: Color::from_rgb(165, 214, 255), // #a5d6ff scale.blue[1]
    string_escape: Color::from_rgb(121, 192, 255), // #79c0ff
    string_special: Color::from_rgb(165, 214, 255), // #a5d6ff

    // Functions
    function: Color::from_rgb(210, 168, 255), // #d2a8ff scale.purple[2]
    function_macro: Color::from_rgb(210, 168, 255), // #d2a8ff
    function_method: Color::from_rgb(210, 168, 255), // #d2a8ff

    // Types & constructors
    type_: Color::from_rgb(255, 166, 87), // #ffa657 scale.orange[2]
    constructor: Color::from_rgb(255, 166, 87), // #ffa657

    // Variables
    variable: Color::from_rgb(255, 166, 87), // #ffa657 scale.orange[2]
    variable_builtin: Color::from_rgb(121, 192, 255), // #79c0ff variable.language
    variable_parameter: Color::from_rgb(230, 237, 243), // #e6edf3 default text

    // Tags & attributes
    tag: Color::from_rgb(126, 231, 135), // #7ee787 scale.green[1]
    attribute: Color::from_rgb(121, 192, 255), // #79c0ff

    // Modules & labels
    module: Color::from_rgb(255, 166, 87), // #ffa657
    label: Color::from_rgb(121, 192, 255), // #79c0ff

    // Properties
    property: Color::from_rgb(121, 192, 255), // #79c0ff meta.property-name

    // Operators & punctuation
    operator: Color::from_rgb(230, 237, 243), // #e6edf3 default text
    punctuation: Color::from_rgb(230, 237, 243), // #e6edf3
    punctuation_bracket: Color::from_rgb(230, 237, 243), // #e6edf3
    punctuation_delimiter: Color::from_rgb(230, 237, 243), // #e6edf3
    punctuation_special: Color::from_rgb(255, 123, 114), // #ff7b72 embedded

    // Escape sequences
    escape: Color::from_rgb(121, 192, 255), // #79c0ff

    // Markup / text
    text_literal: Color::from_rgb(230, 237, 243), // #e6edf3
    text_reference: Color::from_rgb(121, 192, 255), // #79c0ff
    text_title: Color::from_rgb(121, 192, 255),   // #79c0ff markup.heading
    text_uri: Color::from_rgb(165, 214, 255),     // #a5d6ff
    text_emphasis: Color::from_rgb(230, 237, 243), // #e6edf3
};
