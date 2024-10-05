use std::{borrow::Cow, ops::Range};

use fxhash::FxHashMap;
use ropey::Rope;
use smallvec::SmallVec;

const LARGE_FILE: usize = 45_000_000;

#[derive(Clone, Debug)]
pub enum SyntaxType {
    String,
    Keyword,
    SpecialKeyword,
    Punctuation,
    Punctuation2,
    Unknown,
    Property,
    Module,
    Comment,
    SpaceMark,
}

impl SyntaxType {
    pub fn color(&self) -> &str {
        match self {
            SyntaxType::Keyword => "rgb(251, 60, 44)",
            SyntaxType::String => "rgb(151, 151, 26)",
            SyntaxType::Punctuation => "rgb(104, 157, 96)",
            SyntaxType::Punctuation2 => "rgb(252, 188, 61)",
            SyntaxType::Unknown => "rgb(223, 191, 142)",
            SyntaxType::Property => "rgb(152, 192, 124)",
            SyntaxType::Module => "rgb(250, 189, 40)",
            SyntaxType::SpecialKeyword => "rgb(211, 134, 155)",
            SyntaxType::Comment => "gray",
            SyntaxType::SpaceMark => "rgb(223, 191, 142, 0.2)",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
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

pub enum TextNode {
    Range(Range<usize>),
    LineOfChars { len: usize, char: char },
}

pub type SyntaxLine = SmallVec<[(SyntaxType, TextNode); 4]>;

#[derive(Default)]
pub struct SyntaxBlocks {
    blocks: FxHashMap<usize, SyntaxLine>,
}

impl SyntaxBlocks {
    pub fn push_line(&mut self, line: SyntaxLine) {
        self.blocks.insert(self.len(), line);
    }

    pub fn get_line(&self, line: usize) -> &[(SyntaxType, TextNode)] {
        self.blocks.get(&line).unwrap()
    }

    pub fn len(&self) -> usize {
        self.blocks.len()
    }

    pub fn clear(&mut self) {
        self.blocks.clear();
    }
}

const GENERIC_KEYWORDS: &[&str] = &[
    "mod", "use", "impl", "if", "let", "fn", "struct", "enum", "const", "pub", "crate", "else",
    "mut", "for", "i8", "u8", "i16", "u16", "i32", "u32", "f32", "i64", "u64", "f64", "i128",
    "u128", "usize", "isize", "move", "async", "in", "of", "dyn", "type", "match",
];

const SPECIAL_KEYWORDS: &[&str] = &["self", "Self", "false", "true"];

const SPECIAL_CHARACTER: &[char] = &['.', '=', ';', ':', '\'', ',', '#', '&', '-', '+', '^', '\\'];

const SPECIAL_CHARACTER_2: &[char] = &['{', '}', '(', ')', '>', '<', '[', ']'];

#[derive(PartialEq, Clone, Debug)]
enum CommentTracking {
    None,
    OneLine,
    MultiLine,
}

fn flush_generic_stack(
    rope: &Rope,
    generic_stack: &mut Option<Range<usize>>,
    syntax_blocks: &mut SyntaxLine,
    last_semantic: &mut SyntaxSemantic,
    ch: char,
) {
    if let Some(word_pos) = generic_stack {
        let word: Cow<str> = rope.slice(word_pos.clone()).into();
        let trimmed = word.trim();
        if trimmed.is_empty() {
            return;
        }

        let word_pos = generic_stack.take().unwrap();
        let next_char = rope
            .get_slice(word_pos.end + 1..word_pos.end + 2)
            .and_then(|s| s.as_str());

        if ch == ':' && Some(":") == next_char {
            syntax_blocks.push((SyntaxType::Module, TextNode::Range(word_pos)));
        }
        // Match special keywords
        else if GENERIC_KEYWORDS.contains(&trimmed) {
            syntax_blocks.push((SyntaxType::Keyword, TextNode::Range(word_pos)));
        }
        // Match other special keyword, CONSTANTS and numbers
        else if SPECIAL_KEYWORDS.contains(&trimmed) || word.to_uppercase() == word {
            syntax_blocks.push((SyntaxType::SpecialKeyword, TextNode::Range(word_pos)));
        }
        // Match anything else
        else {
            syntax_blocks.push(((*last_semantic).into(), TextNode::Range(word_pos)));
        }

        *last_semantic = SyntaxSemantic::Unknown;
    }
}

fn flush_spaces_stack(
    rope: &Rope,
    generic_stack: &mut Option<Range<usize>>,
    syntax_blocks: &mut SyntaxLine,
    begining_of_line: bool,
    line_is_ending: bool,
) {
    if let Some(word_pos) = &generic_stack {
        let word: Cow<str> = rope.slice(word_pos.clone()).into();
        let trimmed = word.trim();
        if trimmed.is_empty() {
            let range = generic_stack.take().unwrap();
            if !line_is_ending && begining_of_line {
                syntax_blocks.push((
                    SyntaxType::SpaceMark,
                    TextNode::LineOfChars {
                        len: range.end - range.start,
                        char: '·',
                    },
                ));
            } else {
                syntax_blocks.push((SyntaxType::Unknown, TextNode::Range(range)));
            }
        }
    }
}

pub fn parse(rope: &Rope, syntax_blocks: &mut SyntaxBlocks) {
    // Clear any blocks from before
    syntax_blocks.clear();

    if rope.len_chars() >= LARGE_FILE {
        for (n, line) in rope.lines().enumerate() {
            let mut line_blocks = SmallVec::default();
            let start = rope.line_to_char(n);
            let end = line.len_chars();
            line_blocks.push((SyntaxType::Unknown, TextNode::Range(start..start + end)));
            syntax_blocks.push_line(line_blocks);
        }
        return;
    }

    // Track comments
    let mut tracking_comment = CommentTracking::None;
    let mut comment_stack: Option<Range<usize>> = None;

    // Track strings
    let mut tracking_string = false;
    let mut string_stack: Option<Range<usize>> = None;

    // Track anything else
    let mut generic_stack: Option<Range<usize>> = None;
    let mut last_semantic = SyntaxSemantic::Unknown;

    // Elements of the current line
    let mut line = SyntaxLine::new();
    let mut begining_of_line = true;

    for (i, ch) in rope.chars().enumerate() {
        let is_last_character = rope.len_chars() - 1 == i;

        // Ignore the return
        if ch == '\r' {
            continue;
        }

        // Flush all whitespaces from the backback if the character is not an space
        if !ch.is_whitespace() {
            flush_spaces_stack(
                rope,
                &mut generic_stack,
                &mut line,
                begining_of_line,
                is_last_character,
            );

            begining_of_line = false;
        }

        // Stop tracking a string
        if tracking_string && ch == '"' {
            flush_generic_stack(rope, &mut generic_stack, &mut line, &mut last_semantic, ch);

            let mut st = string_stack.take().unwrap_or_default();
            st.end += 1;

            // Strings
            line.push((SyntaxType::String, TextNode::Range(st)));
            tracking_string = false;
        }
        // Start tracking a string
        else if tracking_comment == CommentTracking::None && ch == '"' {
            string_stack = Some(i..i + 1);
            tracking_string = true;
        }
        // While tracking a comment
        else if tracking_comment != CommentTracking::None {
            if let Some(ct) = comment_stack.as_mut() {
                ct.end = i + 1;

                let current_comment: Cow<str> = rope.slice(ct.clone()).into();

                // Stop a multi line comment
                if ch == '/' && current_comment.ends_with("*/") {
                    generic_stack.take();
                    line.push((
                        SyntaxType::Comment,
                        TextNode::Range(comment_stack.take().unwrap()),
                    ));
                    tracking_comment = CommentTracking::None;
                }
            } else {
                comment_stack = Some(i..i + 1);
            }
        }
        // While tracking a string
        else if tracking_string {
            push_to_stack(&mut string_stack, i);
        }
        // If is a special character
        else if SPECIAL_CHARACTER.contains(&ch) {
            flush_generic_stack(rope, &mut generic_stack, &mut line, &mut last_semantic, ch);

            if ch == '.' {
                last_semantic = SyntaxSemantic::PropertyAccess;
            }

            // Punctuation
            line.push((SyntaxType::Punctuation, TextNode::Range(i..i + 1)));
        }
        // If is a special character 2
        else if SPECIAL_CHARACTER_2.contains(&ch) {
            flush_generic_stack(rope, &mut generic_stack, &mut line, &mut last_semantic, ch);

            if ch == '.' {
                last_semantic = SyntaxSemantic::PropertyAccess;
            }

            // Punctuation
            line.push((SyntaxType::Punctuation2, TextNode::Range(i..i + 1)));
        }
        // Unknown (for now at least) characters
        else {
            // Start tracking a comment (both one line and multine)
            if tracking_comment == CommentTracking::None && (ch == '*' || ch == '/') {
                if let Some(us) = generic_stack.as_mut() {
                    let generic_stack_text: Cow<str> = rope.slice(us.clone()).into();
                    if generic_stack_text == "/" {
                        comment_stack = generic_stack.take();

                        push_to_stack(&mut comment_stack, i);

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
                flush_generic_stack(rope, &mut generic_stack, &mut line, &mut last_semantic, ch);
            }

            push_to_stack(&mut generic_stack, i);
        }

        if ch == '\n' || is_last_character {
            // Flush OneLine and MultiLine comments
            if tracking_comment != CommentTracking::None {
                if let Some(ct) = comment_stack.take() {
                    line.push((SyntaxType::Comment, TextNode::Range(ct)));
                }

                // Stop tracking one line comments on line ending
                if tracking_comment == CommentTracking::OneLine {
                    tracking_comment = CommentTracking::None
                }
            }

            flush_generic_stack(rope, &mut generic_stack, &mut line, &mut last_semantic, ch);
            flush_spaces_stack(rope, &mut generic_stack, &mut line, begining_of_line, true);

            if let Some(st) = string_stack.take() {
                line.push((SyntaxType::String, TextNode::Range(st)));
            }

            syntax_blocks.push_line(line.drain(0..).collect());

            // Leave an empty line at the end
            if ch == '\n' && is_last_character {
                syntax_blocks.push_line(SmallVec::default());
            }

            begining_of_line = true;
        }
    }
}

// Push if exists otherwise create the stack
#[inline(always)]
fn push_to_stack(stack: &mut Option<Range<usize>>, idx: usize) {
    if let Some(stack) = stack.as_mut() {
        stack.end = idx + 1;
    } else {
        stack.replace(idx..idx + 1);
    }
}
