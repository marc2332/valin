use std::fmt::{Display, Write};

use ropey::Rope;
use smallvec::SmallVec;
use tokio::time::Instant;

#[derive(Clone, Debug)]
pub enum SyntaxType {
    String,
    Keyword,
    SpecialKeyword,
    Punctuation,
    Unknown,
    Property,
    Comment,
}

impl SyntaxType {
    pub fn color(&self) -> &str {
        match self {
            SyntaxType::Keyword => "rgb(215, 85, 67)",
            SyntaxType::String => "rgb(184, 187, 38)",
            SyntaxType::Punctuation => "rgb(104, 157, 96)",
            SyntaxType::Unknown => "rgb(189, 174, 147)",
            SyntaxType::Property => "rgb(168, 168, 37)",
            SyntaxType::SpecialKeyword => "rgb(211, 134, 155)",
            SyntaxType::Comment => "gray",
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
enum SyntaxSemantic {
    Unknown,
    PropertyAccess,
}

impl From<SyntaxSemantic> for SyntaxType {
    fn from(value: SyntaxSemantic) -> Self {
        match value {
            SyntaxSemantic::PropertyAccess => SyntaxType::Property,
            SyntaxSemantic::Unknown => SyntaxType::Unknown,
        }
    }
}

#[derive(PartialEq, Clone, Debug)]
pub enum TextType {
    String(String),
    Char(char),
}

impl Display for TextType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::String(s) => f.write_str(s),
            Self::Char(c) => f.write_char(*c),
        }
    }
}

pub type SyntaxLine = SmallVec<[(SyntaxType, TextType); 5]>;
pub type SyntaxBlocks = Vec<SyntaxLine>;

const GENERIC_KEYWORDS: &[&str] = &[
    "use", "impl", "if", "let", "fn", "struct", "enum", "const", "pub", "crate", "else", "mut",
    "for", "i8", "u8", "i16", "u16", "i32", "u32", "f32", "i64", "u64", "f64", "i128", "u128",
    "usize", "isize", "move", "async", "in", "of", "dyn", "type",
];

const SPECIAL_KEYWORDS: &[&str] = &["self", "Self", "false", "true"];

const SPECIAL_CHARACTER: &[char] = &[
    '.', '{', '}', '(', ')', '=', ';', '\'', ',', '>', '<', ']', '[', '#', '&', '-', '+', '^', '\\',
];

#[derive(PartialEq, Clone, Debug)]
enum CommentTracking {
    None,
    OneLine,
    MultiLine,
}

fn flush_generic_stack(
    generic_stack: &mut Option<String>,
    syntax_blocks: &mut SyntaxLine,
    last_semantic: &mut SyntaxSemantic,
) {
    if let Some(word) = generic_stack {
        let trimmed = word.trim();
        if trimmed.is_empty() {
            return;
        }
        let word = generic_stack.take().unwrap();

        // Match special keywords
        if GENERIC_KEYWORDS.contains(&word.as_str().trim()) {
            syntax_blocks.push((SyntaxType::Keyword, TextType::String(word)));
        }
        // Match other special keyword, CONSTANTS and numbers
        else if SPECIAL_KEYWORDS.contains(&word.as_str().trim()) || word.to_uppercase() == word {
            syntax_blocks.push((SyntaxType::SpecialKeyword, TextType::String(word)));
        }
        // Match anything else
        else {
            syntax_blocks.push(((*last_semantic).into(), TextType::String(word)));
        }

        *last_semantic = SyntaxSemantic::Unknown;
    }
}

fn flush_spaces_stack(generic_stack: &mut Option<String>, syntax_blocks: &mut SyntaxLine) {
    if let Some(word) = &generic_stack {
        let trimmed = word.trim();
        if trimmed.is_empty() {
            syntax_blocks.push((
                SyntaxType::Unknown,
                TextType::String(generic_stack.take().unwrap()),
            ));
        }
    }
}

pub fn parse(rope: &Rope, syntax_blocks: &mut SyntaxBlocks) {
    // Clear any blocks from before
    syntax_blocks.clear();

    let timer = Instant::now();

    // Track comments
    let mut tracking_comment = CommentTracking::None;
    let mut comment_stack: Option<String> = None;

    // Track strings
    let mut tracking_string = false;
    let mut string_stack: Option<String> = None;

    // Track anything else
    let mut generic_stack: Option<String> = None;
    let mut last_semantic = SyntaxSemantic::Unknown;

    // Elements of the current line
    let mut line = SyntaxLine::new();

    for (i, ch) in rope.chars().enumerate() {
        let is_last_character = rope.len_chars() - 1 == i;

        // Ignore the return
        if ch == '\r' {
            continue;
        }

        // Flush all whitespaces from the backback if the character is not an space
        if !ch.is_whitespace() {
            flush_spaces_stack(&mut generic_stack, &mut line);
        }

        // Stop tracking a string
        if tracking_string && ch == '"' {
            flush_generic_stack(&mut generic_stack, &mut line, &mut last_semantic);

            let mut st = string_stack.take().unwrap_or_default();

            // Strings
            st.push('"');
            line.push((SyntaxType::String, TextType::String(st)));
            tracking_string = false;
        }
        // Start tracking a string
        else if tracking_comment == CommentTracking::None && ch == '"' {
            string_stack = Some(String::from('"'));
            tracking_string = true;
        }
        // While tracking a comment
        else if tracking_comment != CommentTracking::None {
            if let Some(ct) = comment_stack.as_mut() {
                ct.push(ch);

                // Stop a multi line comment
                if ch == '/' && ct.ends_with("*/") {
                    generic_stack.take().unwrap();
                    line.push((
                        SyntaxType::Comment,
                        TextType::String(comment_stack.take().unwrap()),
                    ));
                    tracking_comment = CommentTracking::None;
                }
            } else {
                comment_stack = Some(String::from(ch));
            }
        }
        // While tracking a string
        else if tracking_string {
            push_to_stack(&mut string_stack, ch);
        }
        // If is a special character
        else if SPECIAL_CHARACTER.contains(&ch) {
            flush_generic_stack(&mut generic_stack, &mut line, &mut last_semantic);

            if ch == '.' && last_semantic != SyntaxSemantic::PropertyAccess {
                last_semantic = SyntaxSemantic::PropertyAccess;
            }
            // Punctuation
            line.push((SyntaxType::Punctuation, TextType::Char(ch)));
        }
        // Unknown (for now at least) characters
        else {
            // Start tracking a comment (both one line and multine)
            if tracking_comment == CommentTracking::None && (ch == '*' || ch == '/') {
                if let Some(us) = generic_stack.as_mut() {
                    if us == "/" {
                        comment_stack = generic_stack.take();

                        push_to_stack(&mut comment_stack, ch);

                        if ch == '*' {
                            tracking_comment = CommentTracking::MultiLine
                        } else if ch == '/' {
                            tracking_comment = CommentTracking::OneLine
                        }
                    }
                }
            }

            // Flush the generic stack before adding the space
            if ch.is_whitespace() {
                flush_generic_stack(&mut generic_stack, &mut line, &mut last_semantic);
            }

            push_to_stack(&mut generic_stack, ch);
        }

        if ch == '\n' || is_last_character {
            // Flush OneLine and MultiLine comments
            if tracking_comment != CommentTracking::None {
                if let Some(ct) = comment_stack.take() {
                    line.push((SyntaxType::Comment, TextType::String(ct)));
                }

                // Stop tracking one line comments on line ending
                if tracking_comment == CommentTracking::OneLine {
                    tracking_comment = CommentTracking::None
                }
            }

            flush_generic_stack(&mut generic_stack, &mut line, &mut last_semantic);
            flush_spaces_stack(&mut generic_stack, &mut line);

            if let Some(st) = string_stack.take() {
                line.push((SyntaxType::String, TextType::String(st)));
            }

            syntax_blocks.push(line.drain(0..).collect());

            // Leave an empty line at the end
            if ch == '\n' && is_last_character {
                syntax_blocks.push(SmallVec::default());
            }
        }
    }

    println!("{:?}", timer.elapsed().as_millis());
}

// Push if exists otherwise create the stack
fn push_to_stack(stack: &mut Option<String>, ch: char) {
    if let Some(stack) = stack.as_mut() {
        stack.push(ch);
    } else {
        stack.replace(ch.to_string());
    }
}
