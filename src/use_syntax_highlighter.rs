use freya::prelude::*;
use tree_sitter_highlight::*;

#[derive(Clone)]
pub enum SyntaxType {
    Number,
    String,
    Keyword,
    Operator,
    Variable,
    Unknown,
}

impl From<&str> for SyntaxType {
    fn from(s: &str) -> Self {
        match s {
            "keyword" => SyntaxType::Keyword,
            "variable" => SyntaxType::Variable,
            "operator" => SyntaxType::Operator,
            "string" => SyntaxType::String,
            "number" => SyntaxType::Number,
            _ => SyntaxType::Unknown,
        }
    }
}

pub type SyntaxBlocks = Vec<Vec<(SyntaxType, String)>>;

pub fn use_syntax_highlighter<'a>(
    cx: &'a ScopeState,
    content: &EditableText,
) -> &'a UseState<SyntaxBlocks> {
    let syntax_blocks = use_state::<SyntaxBlocks>(cx, Vec::new);
    let highlighter = cx.use_hook(Highlighter::new);

    // Not proud of using .to_string() here tbh
    use_effect(cx, &content.to_string(), move |_| {
        let highlight_names = &mut [
            "attribute",
            "constant",
            "function.builtin",
            "function",
            "keyword",
            "operator",
            "property",
            "punctuation",
            "punctuation.bracket",
            "punctuation.delimiter",
            "string",
            "string.special",
            "tag",
            "type",
            "type.builtin",
            "variable",
            "variable.builtin",
            "variable.parameter",
            "number",
        ];

        let mut javascript_config = HighlightConfiguration::new(
            tree_sitter_javascript::language(),
            tree_sitter_javascript::HIGHLIGHT_QUERY,
            tree_sitter_javascript::INJECTION_QUERY,
            tree_sitter_javascript::LOCALS_QUERY,
        )
        .unwrap();

        javascript_config.configure(highlight_names);

        let data = content.to_string();
        let highlights = highlighter
            .highlight(&javascript_config, data.as_bytes(), None, |_| None)
            .unwrap();

        syntax_blocks.with_mut(|syntax_blocks| {
            syntax_blocks.clear();
            let mut prepared_block: (SyntaxType, Vec<(usize, String)>) =
                (SyntaxType::Unknown, vec![]);

            for event in highlights {
                match event.unwrap() {
                    HighlightEvent::Source { start, end } => {
                        // Prepare the whole block even if it's splitted across multiple lines.
                        let data = content.get().lines(start..end);
                        let starting_line = content.get().line_of_offset(start);

                        for (i, d) in data.enumerate() {
                            prepared_block.1.push((starting_line + i, d.to_string()));
                        }
                    }
                    HighlightEvent::HighlightStart(s) => {
                        // Specify the type of the block
                        prepared_block.0 = SyntaxType::from(highlight_names[s.0]);
                    }
                    HighlightEvent::HighlightEnd => {
                        // Push all the block chunks to their specified line
                        for (i, d) in prepared_block.1 {
                            if syntax_blocks.get(i).is_none() {
                                syntax_blocks.push(vec![]);
                            }
                            let line = syntax_blocks.last_mut().unwrap();
                            line.push((prepared_block.0.clone(), d));
                        }
                        // Clear the prepared block
                        prepared_block = (SyntaxType::Unknown, vec![]);
                    }
                }
            }

            // Mark all the remaining text as not readable
            if !prepared_block.1.is_empty() {
                for (i, d) in prepared_block.1 {
                    if syntax_blocks.get(i).is_none() {
                        syntax_blocks.push(vec![]);
                    }
                    let line = syntax_blocks.last_mut().unwrap();
                    line.push((SyntaxType::Unknown, d));
                }
            }
        });

        async move {}
    });

    syntax_blocks
}

pub fn get_color_from_type<'a>(t: &SyntaxType) -> &'a str {
    match t {
        SyntaxType::Keyword => "rgb(248, 73, 52)",
        SyntaxType::Variable => "rgb(189, 174, 147)",
        SyntaxType::Operator => "rgb(189, 174, 147)",
        SyntaxType::String => "rgb(184, 187, 38)",
        SyntaxType::Number => "rgb(211, 134, 155)",
        SyntaxType::Unknown => "rgb(189, 174, 147)",
    }
}
