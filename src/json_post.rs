use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Top-level JSON post structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonPost {
    #[serde(default)]
    pub date: String,
    #[serde(default)]
    pub alias: String,
    pub body: Vec<BlockNode>,
}

/// A block-level node in the document body.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum BlockNode {
    #[serde(rename = "heading")]
    Heading { level: u8, content: Vec<InlineNode> },

    #[serde(rename = "paragraph")]
    Paragraph { content: Vec<InlineNode> },

    #[serde(rename = "blockquote")]
    Blockquote { content: Vec<BlockNode> },

    #[serde(rename = "bullet_list")]
    BulletList { items: Vec<ListItem> },

    #[serde(rename = "ordered_list")]
    OrderedList { items: Vec<ListItem> },

    #[serde(rename = "table")]
    Table {
        columns: Vec<TableColumn>,
        rows: Vec<Vec<TableCell>>,
    },

    #[serde(rename = "code_block")]
    CodeBlock {
        #[serde(default)]
        language: String,
        text: String,
    },

    #[serde(rename = "image")]
    Image {
        src: String,
        #[serde(default)]
        alt: String,
    },

    #[serde(rename = "video")]
    Video {
        src: String,
        #[serde(default)]
        alt: String,
    },

    #[serde(rename = "divider")]
    Divider {
        #[serde(default = "default_divider_style")]
        style: String,
    },

    #[serde(rename = "comment")]
    Comment { text: String },

    #[serde(rename = "footnote_def")]
    FootnoteDef { id: u32, content: Vec<InlineNode> },
}

fn default_divider_style() -> String {
    "three_star".to_string()
}

/// An inline node that appears inside paragraphs, list items, etc.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum InlineNode {
    #[serde(rename = "text")]
    Text { text: String },

    #[serde(rename = "bold")]
    Bold { content: Vec<InlineNode> },

    #[serde(rename = "italic")]
    Italic { content: Vec<InlineNode> },

    #[serde(rename = "underline")]
    Underline { content: Vec<InlineNode> },

    #[serde(rename = "strikethrough")]
    Strikethrough { content: Vec<InlineNode> },

    #[serde(rename = "superscript")]
    Superscript { content: Vec<InlineNode> },

    #[serde(rename = "highlight")]
    Highlight { content: Vec<InlineNode> },

    #[serde(rename = "spoiler")]
    Spoiler { content: Vec<InlineNode> },

    #[serde(rename = "code_inline")]
    CodeInline { text: String },

    #[serde(rename = "link")]
    Link {
        href: String,
        content: Vec<InlineNode>,
    },

    #[serde(rename = "footnote_ref")]
    FootnoteRef { id: u32 },

    #[serde(rename = "footnote_inline")]
    FootnoteInline { content: Vec<InlineNode> },

    #[serde(rename = "line_break")]
    LineBreak,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListItem {
    pub content: Vec<InlineNode>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableColumn {
    pub header: String,
    #[serde(default = "default_align")]
    pub align: String,
}

fn default_align() -> String {
    "left".to_string()
}

/// A table cell can be either a plain string or an object with rich content.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TableCell {
    Plain(String),
    Rich { content: Vec<InlineNode> },
}

// ─── Conversion: JSON → Nonograph Markdown ──────────────────────────────────

impl JsonPost {
    /// Parse a JSON string into a `JsonPost`.
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    /// Convert this post into nonograph markdown.
    pub fn to_markup(&self) -> String {
        let mut out = String::new();

        // Header line: "date | alias" or just "date"
        if !self.date.is_empty() {
            out.push_str(&self.date);
            if !self.alias.is_empty() {
                out.push_str(" | ");
                out.push_str(&self.alias);
            }
            out.push('\n');
        }

        // Collect footnote definitions to emit at the bottom
        let mut footnote_defs: Vec<(u32, Vec<InlineNode>)> = Vec::new();

        for (i, node) in self.body.iter().enumerate() {
            if let BlockNode::FootnoteDef { id, content } = node {
                footnote_defs.push((*id, content.clone()));
                continue;
            }

            // Separate blocks with blank lines
            if i > 0 {
                // Check if previous non-footnote-def block existed
                out.push('\n');
            }

            render_block(node, &mut out);
            out.push('\n');
        }

        // Emit footnote definitions at the end
        if !footnote_defs.is_empty() {
            out.push('\n');
            for (id, content) in &footnote_defs {
                out.push_str(&format!("[^{}]: {}\n", id, render_inline_nodes(content)));
            }
        }

        out
    }
}

fn render_block(node: &BlockNode, out: &mut String) {
    match node {
        BlockNode::Heading { level, content } => {
            let prefix = "#".repeat(*level as usize);
            out.push_str(&prefix);
            out.push(' ');
            out.push_str(&render_inline_nodes(content));
        }
        BlockNode::Paragraph { content } => {
            out.push_str(&render_inline_nodes(content));
        }
        BlockNode::Blockquote { content } => {
            for block in content {
                match block {
                    BlockNode::Paragraph { content: inlines } => {
                        out.push_str("> ");
                        out.push_str(&render_inline_nodes(inlines));
                    }
                    other => {
                        // For non-paragraph blocks inside blockquotes, prefix each line
                        let mut inner = String::new();
                        render_block(other, &mut inner);
                        for (j, line) in inner.lines().enumerate() {
                            if j > 0 {
                                out.push('\n');
                            }
                            out.push_str("> ");
                            out.push_str(line);
                        }
                    }
                }
            }
        }
        BlockNode::BulletList { items } => {
            for (j, item) in items.iter().enumerate() {
                if j > 0 {
                    out.push('\n');
                }
                out.push_str("- ");
                out.push_str(&render_inline_nodes(&item.content));
            }
        }
        BlockNode::OrderedList { items } => {
            for (j, item) in items.iter().enumerate() {
                if j > 0 {
                    out.push('\n');
                }
                out.push_str(&format!("{}. ", j + 1));
                out.push_str(&render_inline_nodes(&item.content));
            }
        }
        BlockNode::Table { columns, rows } => {
            // Header row
            out.push_str("| ");
            for (j, col) in columns.iter().enumerate() {
                if j > 0 {
                    out.push_str(" | ");
                }
                out.push_str(&col.header);
            }
            out.push_str(" |");

            // Separator row
            out.push('\n');
            out.push_str("|");
            for col in columns {
                match col.align.as_str() {
                    "center" => out.push_str(":---:|"),
                    "right" => out.push_str("---:|"),
                    _ => out.push_str("---|"),
                }
            }

            // Data rows
            for row in rows {
                out.push('\n');
                out.push_str("| ");
                for (j, cell) in row.iter().enumerate() {
                    if j > 0 {
                        out.push_str(" | ");
                    }
                    match cell {
                        TableCell::Plain(s) => out.push_str(s),
                        TableCell::Rich { content } => {
                            out.push_str(&render_inline_nodes(content));
                        }
                    }
                }
                out.push_str(" |");
            }
        }
        BlockNode::CodeBlock { language, text } => {
            out.push_str("```");
            out.push_str(language);
            out.push('\n');
            out.push_str(text);
            out.push('\n');
            out.push_str("```");
        }
        BlockNode::Image { src, alt } => {
            out.push_str("![");
            out.push_str(alt);
            out.push_str("](");
            out.push_str(src);
            out.push(')');
        }
        BlockNode::Video { src, alt } => {
            out.push_str("![");
            out.push_str(alt);
            out.push_str("](");
            out.push_str(src);
            out.push(')');
        }
        BlockNode::Divider { style } => match style.as_str() {
            "single_star" => out.push_str("-*-"),
            "line" => out.push_str("---"),
            "double_line" => out.push_str("==="),
            _ => out.push_str("***"),
        },
        BlockNode::Comment { text } => {
            out.push_str("// ");
            out.push_str(text);
        }
        BlockNode::FootnoteDef { .. } => {
            // Handled at the top level in to_markup()
        }
    }
}

fn render_inline_nodes(nodes: &[InlineNode]) -> String {
    let mut out = String::new();
    for node in nodes {
        render_inline(node, &mut out);
    }
    out
}

fn render_inline(node: &InlineNode, out: &mut String) {
    match node {
        InlineNode::Text { text } => out.push_str(text),
        InlineNode::Bold { content } => {
            out.push_str("**");
            out.push_str(&render_inline_nodes(content));
            out.push_str("**");
        }
        InlineNode::Italic { content } => {
            out.push('*');
            out.push_str(&render_inline_nodes(content));
            out.push('*');
        }
        InlineNode::Underline { content } => {
            out.push('_');
            out.push_str(&render_inline_nodes(content));
            out.push('_');
        }
        InlineNode::Strikethrough { content } => {
            out.push('~');
            out.push_str(&render_inline_nodes(content));
            out.push('~');
        }
        InlineNode::Superscript { content } => {
            out.push('^');
            out.push_str(&render_inline_nodes(content));
            out.push('^');
        }
        InlineNode::Highlight { content } => {
            out.push_str("==");
            out.push_str(&render_inline_nodes(content));
            out.push_str("==");
        }
        InlineNode::Spoiler { content } => {
            out.push('#');
            out.push_str(&render_inline_nodes(content));
            out.push('#');
        }
        InlineNode::CodeInline { text } => {
            out.push('`');
            out.push_str(text);
            out.push('`');
        }
        InlineNode::Link { href, content } => {
            let inner = render_inline_nodes(content);
            // If the link text equals the href, use bare link syntax [url]
            if inner == *href {
                out.push('[');
                out.push_str(href);
                out.push(']');
            } else {
                out.push('[');
                out.push_str(&inner);
                out.push_str("](");
                out.push_str(href);
                out.push(')');
            }
        }
        InlineNode::FootnoteRef { id } => {
            out.push_str(&format!("[^{}]", id));
        }
        InlineNode::FootnoteInline { content } => {
            out.push_str("^[");
            out.push_str(&render_inline_nodes(content));
            out.push(']');
        }
        InlineNode::LineBreak => {
            out.push('\n');
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::render_markdown;

    // ── Deserialization tests ────────────────────────────────────────────

    #[test]
    fn test_deserialize_minimal() {
        let json = r#"{"body": []}"#;
        let post = JsonPost::from_json(json).unwrap();
        assert!(post.body.is_empty());
        assert_eq!(post.date, "");
        assert_eq!(post.alias, "");
    }

    #[test]
    fn test_deserialize_heading() {
        let json = r#"{"body": [{"type": "heading", "level": 2, "content": [{"type": "text", "text": "Hello"}]}]}"#;
        let post = JsonPost::from_json(json).unwrap();
        assert_eq!(post.body.len(), 1);
        if let BlockNode::Heading { level, content } = &post.body[0] {
            assert_eq!(*level, 2);
            assert_eq!(content.len(), 1);
        } else {
            panic!("expected heading");
        }
    }

    #[test]
    fn test_deserialize_table_mixed_cells() {
        let json = r#"{"body": [{"type": "table", "columns": [{"header": "A"}, {"header": "B"}], "rows": [["plain", {"content": [{"type": "bold", "content": [{"type": "text", "text": "rich"}]}]}]]}]}"#;
        let post = JsonPost::from_json(json).unwrap();
        if let BlockNode::Table { rows, .. } = &post.body[0] {
            assert_eq!(rows.len(), 1);
            assert!(matches!(&rows[0][0], TableCell::Plain(s) if s == "plain"));
            assert!(matches!(&rows[0][1], TableCell::Rich { .. }));
        } else {
            panic!("expected table");
        }
    }

    // ── Markup generation tests ──────────────────────────────────────────

    #[test]
    fn test_heading_to_markup() {
        let post = JsonPost {
            date: String::new(),
            alias: String::new(),
            body: vec![BlockNode::Heading {
                level: 1,
                content: vec![InlineNode::Text {
                    text: "Title".into(),
                }],
            }],
        };
        let markup = post.to_markup();
        assert!(markup.contains("# Title"));
    }

    #[test]
    fn test_h2_to_markup() {
        let post = JsonPost {
            date: String::new(),
            alias: String::new(),
            body: vec![BlockNode::Heading {
                level: 2,
                content: vec![InlineNode::Text {
                    text: "Section".into(),
                }],
            }],
        };
        let markup = post.to_markup();
        assert!(markup.contains("## Section"));
    }

    #[test]
    fn test_bold_text_to_markup() {
        let post = JsonPost {
            date: String::new(),
            alias: String::new(),
            body: vec![BlockNode::Paragraph {
                content: vec![
                    InlineNode::Text {
                        text: "Hello ".into(),
                    },
                    InlineNode::Bold {
                        content: vec![InlineNode::Text {
                            text: "world".into(),
                        }],
                    },
                ],
            }],
        };
        let markup = post.to_markup();
        assert!(markup.contains("Hello **world**"));
    }

    #[test]
    fn test_all_inline_formatting() {
        let post = JsonPost {
            date: String::new(),
            alias: String::new(),
            body: vec![BlockNode::Paragraph {
                content: vec![
                    InlineNode::Bold {
                        content: vec![InlineNode::Text { text: "b".into() }],
                    },
                    InlineNode::Text { text: " ".into() },
                    InlineNode::Italic {
                        content: vec![InlineNode::Text { text: "i".into() }],
                    },
                    InlineNode::Text { text: " ".into() },
                    InlineNode::Underline {
                        content: vec![InlineNode::Text { text: "u".into() }],
                    },
                    InlineNode::Text { text: " ".into() },
                    InlineNode::Strikethrough {
                        content: vec![InlineNode::Text { text: "s".into() }],
                    },
                    InlineNode::Text { text: " ".into() },
                    InlineNode::Superscript {
                        content: vec![InlineNode::Text { text: "sup".into() }],
                    },
                    InlineNode::Text { text: " ".into() },
                    InlineNode::Highlight {
                        content: vec![InlineNode::Text {
                            text: "mark".into(),
                        }],
                    },
                    InlineNode::Text { text: " ".into() },
                    InlineNode::CodeInline {
                        text: "code".into(),
                    },
                ],
            }],
        };
        let markup = post.to_markup();
        assert!(markup.contains("**b**"));
        assert!(markup.contains("*i*"));
        assert!(markup.contains("_u_"));
        assert!(markup.contains("~s~"));
        assert!(markup.contains("^sup^"));
        assert!(markup.contains("==mark=="));
        assert!(markup.contains("`code`"));
    }

    #[test]
    fn test_spoiler_to_markup() {
        let post = JsonPost {
            date: String::new(),
            alias: String::new(),
            body: vec![BlockNode::Paragraph {
                content: vec![InlineNode::Spoiler {
                    content: vec![InlineNode::Text {
                        text: "hidden".into(),
                    }],
                }],
            }],
        };
        let markup = post.to_markup();
        assert!(markup.contains("#hidden#"));
    }

    #[test]
    fn test_labeled_link_to_markup() {
        let post = JsonPost {
            date: String::new(),
            alias: String::new(),
            body: vec![BlockNode::Paragraph {
                content: vec![InlineNode::Link {
                    href: "https://example.com".into(),
                    content: vec![InlineNode::Text {
                        text: "click me".into(),
                    }],
                }],
            }],
        };
        let markup = post.to_markup();
        assert!(markup.contains("[click me](https://example.com)"));
    }

    #[test]
    fn test_bare_link_to_markup() {
        let post = JsonPost {
            date: String::new(),
            alias: String::new(),
            body: vec![BlockNode::Paragraph {
                content: vec![InlineNode::Link {
                    href: "https://example.com".into(),
                    content: vec![InlineNode::Text {
                        text: "https://example.com".into(),
                    }],
                }],
            }],
        };
        let markup = post.to_markup();
        assert!(markup.contains("[https://example.com]"));
        assert!(!markup.contains("]("));
    }

    #[test]
    fn test_image_to_markup() {
        let post = JsonPost {
            date: String::new(),
            alias: String::new(),
            body: vec![BlockNode::Image {
                src: "https://example.com/photo.jpg".into(),
                alt: "A photo".into(),
            }],
        };
        let markup = post.to_markup();
        assert!(markup.contains("![A photo](https://example.com/photo.jpg)"));
    }

    #[test]
    fn test_image_no_caption_to_markup() {
        let post = JsonPost {
            date: String::new(),
            alias: String::new(),
            body: vec![BlockNode::Image {
                src: "https://example.com/pic.png".into(),
                alt: String::new(),
            }],
        };
        let markup = post.to_markup();
        assert!(markup.contains("![](https://example.com/pic.png)"));
    }

    #[test]
    fn test_video_to_markup() {
        let post = JsonPost {
            date: String::new(),
            alias: String::new(),
            body: vec![BlockNode::Video {
                src: "https://example.com/clip.mp4".into(),
                alt: "My video".into(),
            }],
        };
        let markup = post.to_markup();
        assert!(markup.contains("![My video](https://example.com/clip.mp4)"));
    }

    #[test]
    fn test_bullet_list_to_markup() {
        let post = JsonPost {
            date: String::new(),
            alias: String::new(),
            body: vec![BlockNode::BulletList {
                items: vec![
                    ListItem {
                        content: vec![InlineNode::Text {
                            text: "first".into(),
                        }],
                    },
                    ListItem {
                        content: vec![InlineNode::Text {
                            text: "second".into(),
                        }],
                    },
                ],
            }],
        };
        let markup = post.to_markup();
        assert!(markup.contains("- first\n- second"));
    }

    #[test]
    fn test_ordered_list_to_markup() {
        let post = JsonPost {
            date: String::new(),
            alias: String::new(),
            body: vec![BlockNode::OrderedList {
                items: vec![
                    ListItem {
                        content: vec![InlineNode::Text { text: "one".into() }],
                    },
                    ListItem {
                        content: vec![InlineNode::Text { text: "two".into() }],
                    },
                ],
            }],
        };
        let markup = post.to_markup();
        assert!(markup.contains("1. one\n2. two"));
    }

    #[test]
    fn test_code_block_to_markup() {
        let post = JsonPost {
            date: String::new(),
            alias: String::new(),
            body: vec![BlockNode::CodeBlock {
                language: "rust".into(),
                text: "fn main() {}".into(),
            }],
        };
        let markup = post.to_markup();
        assert!(markup.contains("```rust\nfn main() {}\n```"));
    }

    #[test]
    fn test_blockquote_to_markup() {
        let post = JsonPost {
            date: String::new(),
            alias: String::new(),
            body: vec![BlockNode::Blockquote {
                content: vec![BlockNode::Paragraph {
                    content: vec![InlineNode::Text {
                        text: "quoted text".into(),
                    }],
                }],
            }],
        };
        let markup = post.to_markup();
        assert!(markup.contains("> quoted text"));
    }

    #[test]
    fn test_dividers_to_markup() {
        let post = JsonPost {
            date: String::new(),
            alias: String::new(),
            body: vec![
                BlockNode::Divider {
                    style: "three_star".into(),
                },
                BlockNode::Divider {
                    style: "single_star".into(),
                },
                BlockNode::Divider {
                    style: "line".into(),
                },
                BlockNode::Divider {
                    style: "double_line".into(),
                },
            ],
        };
        let markup = post.to_markup();
        assert!(markup.contains("***"));
        assert!(markup.contains("-*-"));
        assert!(markup.contains("---"));
        assert!(markup.contains("==="));
    }

    #[test]
    fn test_comment_to_markup() {
        let post = JsonPost {
            date: String::new(),
            alias: String::new(),
            body: vec![BlockNode::Comment {
                text: "This is hidden".into(),
            }],
        };
        let markup = post.to_markup();
        assert!(markup.contains("// This is hidden"));
    }

    #[test]
    fn test_footnote_ref_to_markup() {
        let post = JsonPost {
            date: String::new(),
            alias: String::new(),
            body: vec![
                BlockNode::Paragraph {
                    content: vec![
                        InlineNode::Text {
                            text: "See here".into(),
                        },
                        InlineNode::FootnoteRef { id: 1 },
                    ],
                },
                BlockNode::FootnoteDef {
                    id: 1,
                    content: vec![InlineNode::Text {
                        text: "The source.".into(),
                    }],
                },
            ],
        };
        let markup = post.to_markup();
        assert!(markup.contains("See here[^1]"));
        assert!(markup.contains("[^1]: The source."));
    }

    #[test]
    fn test_inline_footnote_to_markup() {
        let post = JsonPost {
            date: String::new(),
            alias: String::new(),
            body: vec![BlockNode::Paragraph {
                content: vec![
                    InlineNode::Text {
                        text: "Something".into(),
                    },
                    InlineNode::FootnoteInline {
                        content: vec![InlineNode::Text {
                            text: "inline note".into(),
                        }],
                    },
                ],
            }],
        };
        let markup = post.to_markup();
        assert!(markup.contains("Something^[inline note]"));
    }

    #[test]
    fn test_table_to_markup() {
        let post = JsonPost {
            date: String::new(),
            alias: String::new(),
            body: vec![BlockNode::Table {
                columns: vec![
                    TableColumn {
                        header: "Name".into(),
                        align: "left".into(),
                    },
                    TableColumn {
                        header: "Age".into(),
                        align: "right".into(),
                    },
                ],
                rows: vec![vec![
                    TableCell::Plain("Alice".into()),
                    TableCell::Plain("30".into()),
                ]],
            }],
        };
        let markup = post.to_markup();
        assert!(markup.contains("| Name | Age |"));
        assert!(markup.contains("|---|---:|"));
        assert!(markup.contains("| Alice | 30 |"));
    }

    #[test]
    fn test_table_center_align() {
        let post = JsonPost {
            date: String::new(),
            alias: String::new(),
            body: vec![BlockNode::Table {
                columns: vec![TableColumn {
                    header: "Mid".into(),
                    align: "center".into(),
                }],
                rows: vec![vec![TableCell::Plain("x".into())]],
            }],
        };
        let markup = post.to_markup();
        assert!(markup.contains("|:---:|"));
    }

    #[test]
    fn test_line_break_to_markup() {
        let post = JsonPost {
            date: String::new(),
            alias: String::new(),
            body: vec![BlockNode::Paragraph {
                content: vec![
                    InlineNode::Text {
                        text: "line one".into(),
                    },
                    InlineNode::LineBreak,
                    InlineNode::Text {
                        text: "line two".into(),
                    },
                ],
            }],
        };
        let markup = post.to_markup();
        assert!(markup.contains("line one\nline two"));
    }

    #[test]
    fn test_date_and_alias_header() {
        let post = JsonPost {
            date: "July 10, 2026".into(),
            alias: "formatting showcase".into(),
            body: vec![],
        };
        let markup = post.to_markup();
        assert!(markup.starts_with("July 10, 2026 | formatting showcase\n"));
    }

    #[test]
    fn test_date_only_header() {
        let post = JsonPost {
            date: "March 1, 2026".into(),
            alias: String::new(),
            body: vec![],
        };
        let markup = post.to_markup();
        assert!(markup.starts_with("March 1, 2026\n"));
        assert!(!markup.contains(" | "));
    }

    // ── Round-trip: JSON → markup → HTML should match direct markdown → HTML ─

    #[test]
    fn test_bold_roundtrip_html() {
        let json = r#"{"body": [{"type": "paragraph", "content": [{"type": "text", "text": "Hello "}, {"type": "bold", "content": [{"type": "text", "text": "world"}]}]}]}"#;
        let post = JsonPost::from_json(json).unwrap();
        let markup = post.to_markup();
        let html = render_markdown(&markup);
        assert!(html.contains("<strong>world</strong>"));
    }

    #[test]
    fn test_italic_roundtrip_html() {
        let json = r#"{"body": [{"type": "paragraph", "content": [{"type": "italic", "content": [{"type": "text", "text": "emphasis"}]}]}]}"#;
        let post = JsonPost::from_json(json).unwrap();
        let markup = post.to_markup();
        let html = render_markdown(&markup);
        assert!(html.contains("<em>emphasis</em>"));
    }

    #[test]
    fn test_underline_roundtrip_html() {
        let json = r#"{"body": [{"type": "paragraph", "content": [{"type": "underline", "content": [{"type": "text", "text": "under"}]}]}]}"#;
        let post = JsonPost::from_json(json).unwrap();
        let markup = post.to_markup();
        let html = render_markdown(&markup);
        assert!(html.contains("<u>under</u>"));
    }

    #[test]
    fn test_strikethrough_roundtrip_html() {
        let json = r#"{"body": [{"type": "paragraph", "content": [{"type": "strikethrough", "content": [{"type": "text", "text": "gone"}]}]}]}"#;
        let post = JsonPost::from_json(json).unwrap();
        let markup = post.to_markup();
        let html = render_markdown(&markup);
        assert!(html.contains("<del>gone</del>"));
    }

    #[test]
    fn test_superscript_roundtrip_html() {
        let json = r#"{"body": [{"type": "paragraph", "content": [{"type": "superscript", "content": [{"type": "text", "text": "2"}]}]}]}"#;
        let post = JsonPost::from_json(json).unwrap();
        let markup = post.to_markup();
        let html = render_markdown(&markup);
        assert!(html.contains("<sup>2</sup>"));
    }

    #[test]
    fn test_highlight_roundtrip_html() {
        let json = r#"{"body": [{"type": "paragraph", "content": [{"type": "highlight", "content": [{"type": "text", "text": "important"}]}]}]}"#;
        let post = JsonPost::from_json(json).unwrap();
        let markup = post.to_markup();
        let html = render_markdown(&markup);
        assert!(html.contains("<mark>important</mark>"));
    }

    #[test]
    fn test_spoiler_roundtrip_html() {
        let json = r#"{"body": [{"type": "paragraph", "content": [{"type": "spoiler", "content": [{"type": "text", "text": "secret"}]}]}]}"#;
        let post = JsonPost::from_json(json).unwrap();
        let markup = post.to_markup();
        let html = render_markdown(&markup);
        assert!(html.contains("class=\"secret\""));
        assert!(html.contains("secret"));
    }

    #[test]
    fn test_inline_code_roundtrip_html() {
        let json = r#"{"body": [{"type": "paragraph", "content": [{"type": "code_inline", "text": "let x = 1;"}]}]}"#;
        let post = JsonPost::from_json(json).unwrap();
        let markup = post.to_markup();
        let html = render_markdown(&markup);
        assert!(html.contains("<code>"));
        assert!(html.contains("let x = 1;"));
    }

    #[test]
    fn test_link_roundtrip_html() {
        let json = r#"{"body": [{"type": "paragraph", "content": [{"type": "link", "href": "https://example.com", "content": [{"type": "text", "text": "click"}]}]}]}"#;
        let post = JsonPost::from_json(json).unwrap();
        let markup = post.to_markup();
        let html = render_markdown(&markup);
        assert!(html.contains("href=\"https://example.com\""));
        assert!(html.contains("click</a>"));
    }

    #[test]
    fn test_bare_link_roundtrip_html() {
        let json = r#"{"body": [{"type": "paragraph", "content": [{"type": "link", "href": "https://example.com", "content": [{"type": "text", "text": "https://example.com"}]}]}]}"#;
        let post = JsonPost::from_json(json).unwrap();
        let markup = post.to_markup();
        let html = render_markdown(&markup);
        assert!(html.contains("href=\"https://example.com\""));
    }

    #[test]
    fn test_heading_roundtrip_html() {
        let json = r#"{"body": [{"type": "heading", "level": 2, "content": [{"type": "text", "text": "Section"}]}]}"#;
        let post = JsonPost::from_json(json).unwrap();
        let markup = post.to_markup();
        let html = render_markdown(&markup);
        assert!(html.contains("<h2"));
        assert!(html.contains("Section"));
    }

    #[test]
    fn test_blockquote_roundtrip_html() {
        let json = r#"{"body": [{"type": "blockquote", "content": [{"type": "paragraph", "content": [{"type": "text", "text": "wisdom"}]}]}]}"#;
        let post = JsonPost::from_json(json).unwrap();
        let markup = post.to_markup();
        let html = render_markdown(&markup);
        assert!(html.contains("<blockquote>"));
        assert!(html.contains("wisdom"));
    }

    #[test]
    fn test_bullet_list_roundtrip_html() {
        let json = r#"{"body": [{"type": "bullet_list", "items": [{"content": [{"type": "text", "text": "alpha"}]}, {"content": [{"type": "text", "text": "beta"}]}]}]}"#;
        let post = JsonPost::from_json(json).unwrap();
        let markup = post.to_markup();
        let html = render_markdown(&markup);
        assert!(html.contains("<ul>"));
        assert!(html.contains("<li>alpha</li>"));
        assert!(html.contains("<li>beta</li>"));
    }

    #[test]
    fn test_ordered_list_roundtrip_html() {
        let json = r#"{"body": [{"type": "ordered_list", "items": [{"content": [{"type": "text", "text": "first"}]}, {"content": [{"type": "text", "text": "second"}]}]}]}"#;
        let post = JsonPost::from_json(json).unwrap();
        let markup = post.to_markup();
        let html = render_markdown(&markup);
        assert!(html.contains("<ol>"));
        assert!(html.contains("<li>first</li>"));
        assert!(html.contains("<li>second</li>"));
    }

    #[test]
    fn test_image_roundtrip_html() {
        let json = r#"{"body": [{"type": "image", "src": "https://example.com/pic.jpg", "alt": "A picture"}]}"#;
        let post = JsonPost::from_json(json).unwrap();
        let markup = post.to_markup();
        let html = render_markdown(&markup);
        assert!(html.contains("<img src="));
        assert!(html.contains("pic.jpg"));
    }

    #[test]
    fn test_image_with_caption_roundtrip_html() {
        let json = r#"{"body": [{"type": "image", "src": "https://example.com/pic.jpg", "alt": "Nice photo"}]}"#;
        let post = JsonPost::from_json(json).unwrap();
        let markup = post.to_markup();
        let html = render_markdown(&markup);
        assert!(html.contains("media-caption"));
        assert!(html.contains("Nice photo"));
    }

    #[test]
    fn test_video_roundtrip_html() {
        let json =
            r#"{"body": [{"type": "video", "src": "https://example.com/clip.mp4", "alt": ""}]}"#;
        let post = JsonPost::from_json(json).unwrap();
        let markup = post.to_markup();
        let html = render_markdown(&markup);
        assert!(html.contains("<video"));
        assert!(html.contains("clip.mp4"));
    }

    #[test]
    fn test_code_block_roundtrip_html() {
        let json =
            r#"{"body": [{"type": "code_block", "language": "rust", "text": "fn main() {}"}]}"#;
        let post = JsonPost::from_json(json).unwrap();
        let markup = post.to_markup();
        let html = render_markdown(&markup);
        assert!(html.contains("<pre"));
        assert!(html.contains("language-rust"));
    }

    #[test]
    fn test_table_roundtrip_html() {
        let json = r#"{"body": [{"type": "table", "columns": [{"header": "A"}, {"header": "B"}], "rows": [["1", "2"]]}]}"#;
        let post = JsonPost::from_json(json).unwrap();
        let markup = post.to_markup();
        let html = render_markdown(&markup);
        assert!(html.contains("<table>"));
        assert!(html.contains("<th>A</th>"));
        assert!(html.contains("<td>1</td>"));
    }

    #[test]
    fn test_divider_three_star_roundtrip_html() {
        let json = r#"{"body": [{"type": "divider", "style": "three_star"}]}"#;
        let post = JsonPost::from_json(json).unwrap();
        let markup = post.to_markup();
        let html = render_markdown(&markup);
        assert!(html.contains("divider-stars"));
    }

    #[test]
    fn test_divider_line_roundtrip_html() {
        let json = r#"{"body": [{"type": "divider", "style": "line"}]}"#;
        let post = JsonPost::from_json(json).unwrap();
        let markup = post.to_markup();
        let html = render_markdown(&markup);
        assert!(html.contains("divider-thin"));
    }

    #[test]
    fn test_divider_double_roundtrip_html() {
        let json = r#"{"body": [{"type": "divider", "style": "double_line"}]}"#;
        let post = JsonPost::from_json(json).unwrap();
        let markup = post.to_markup();
        let html = render_markdown(&markup);
        assert!(html.contains("divider-double"));
    }

    #[test]
    fn test_divider_single_star_roundtrip_html() {
        let json = r#"{"body": [{"type": "divider", "style": "single_star"}]}"#;
        let post = JsonPost::from_json(json).unwrap();
        let markup = post.to_markup();
        let html = render_markdown(&markup);
        assert!(html.contains("divider-asterisk"));
    }

    #[test]
    fn test_comment_roundtrip_html() {
        let json = r#"{"body": [{"type": "comment", "text": "invisible"}, {"type": "paragraph", "content": [{"type": "text", "text": "visible"}]}]}"#;
        let post = JsonPost::from_json(json).unwrap();
        let markup = post.to_markup();
        let html = render_markdown(&markup);
        // Comments are stripped from HTML
        assert!(!html.contains("invisible"));
        assert!(html.contains("visible"));
    }

    #[test]
    fn test_footnote_roundtrip_html() {
        let json = r#"{"body": [{"type": "paragraph", "content": [{"type": "text", "text": "See"}, {"type": "footnote_ref", "id": 1}]}, {"type": "footnote_def", "id": 1, "content": [{"type": "text", "text": "A note."}]}]}"#;
        let post = JsonPost::from_json(json).unwrap();
        let markup = post.to_markup();
        let html = render_markdown(&markup);
        assert!(html.contains("footnote"));
    }

    #[test]
    fn test_inline_footnote_roundtrip_html() {
        let json = r#"{"body": [{"type": "paragraph", "content": [{"type": "text", "text": "fact"}, {"type": "footnote_inline", "content": [{"type": "text", "text": "source here"}]}]}]}"#;
        let post = JsonPost::from_json(json).unwrap();
        let markup = post.to_markup();
        let html = render_markdown(&markup);
        assert!(html.contains("footnote"));
        assert!(html.contains("source here"));
    }

    #[test]
    fn test_list_with_formatting_roundtrip_html() {
        let json = r#"{"body": [{"type": "bullet_list", "items": [{"content": [{"type": "text", "text": "item with "}, {"type": "bold", "content": [{"type": "text", "text": "bold"}]}]}]}]}"#;
        let post = JsonPost::from_json(json).unwrap();
        let markup = post.to_markup();
        let html = render_markdown(&markup);
        assert!(html.contains("<strong>bold</strong>"));
        assert!(html.contains("<li>"));
    }

    // ── Full example-post.json round-trip ────────────────────────────────

    #[test]
    fn test_example_post_deserialize() {
        let json = include_str!("../example-post.json");
        let post = JsonPost::from_json(json).unwrap();

        // Verify top-level fields
        assert_eq!(post.date, "July 10, 2026");
        assert_eq!(post.alias, "formatting showcase");

        // Count body nodes (excluding footnote defs)
        assert!(
            post.body.len() > 40,
            "expected many body nodes, got {}",
            post.body.len()
        );
    }

    #[test]
    fn test_example_post_to_markup() {
        let json = include_str!("../example-post.json");
        let post = JsonPost::from_json(json).unwrap();
        let markup = post.to_markup();

        // Verify header line
        assert!(markup.starts_with("July 10, 2026 | formatting showcase\n"));

        // Verify headings
        assert!(markup.contains("# The Complete Nonograph Formatting Showcase"));
        assert!(markup.contains("## Text Formatting"));
        assert!(markup.contains("### Heading Level 3"));
        assert!(markup.contains("#### Heading Level 4"));

        // Verify inline formatting
        assert!(markup.contains("**bold text**"));
        assert!(markup.contains("*italic text*"));
        assert!(markup.contains("_underlined text_"));
        assert!(markup.contains("~strikethrough text~"));
        assert!(markup.contains("^superscript text^"));
        assert!(markup.contains("==highlighted text=="));
        assert!(markup.contains("`inline code`"));
        assert!(markup.contains("#hidden spoiler message#"));

        // Verify blockquotes
        assert!(markup.contains("> This is a simple blockquote."));

        // Verify footnotes
        assert!(markup.contains("[^1]"));
        assert!(markup.contains("[^2]"));
        assert!(markup.contains("^[This is defined right here in the text]"));
        assert!(markup.contains("[^1]: First footnote"));
        assert!(markup.contains("[^2]: Second footnote"));

        // Verify links
        assert!(markup.contains("[labeled link](https://example.com)"));
        assert!(markup.contains("[https://nonograph.net]"));

        // Verify images
        assert!(markup.contains("![A photo with a caption](https://example.com/photo.jpg)"));
        assert!(markup.contains("![](https://example.com/no-caption.png)"));

        // Verify videos
        assert!(markup.contains("![Video with caption](https://example.com/clip.mp4)"));
        assert!(markup.contains("![](https://example.com/silent.webm)"));

        // Verify lists
        assert!(markup.contains("- First bullet item"));
        assert!(markup.contains("- Second bullet item with **bold**"));
        assert!(markup.contains("1. First step"));

        // Verify tables
        assert!(markup.contains("| Language | Year | Paradigm |"));

        // Verify code blocks
        assert!(markup.contains("```rust\n"));
        assert!(markup.contains("```json\n"));
        assert!(markup.contains("```py\n"));

        // Verify comments
        assert!(markup.contains("// This is a comment"));

        // Verify spoilers
        assert!(markup.contains("#this is a spoiler#"));

        // Verify dividers
        assert!(markup.contains("\n***\n"));
        assert!(markup.contains("\n-*-\n"));
        assert!(markup.contains("\n---\n"));
        assert!(markup.contains("\n===\n"));
    }

    #[test]
    fn test_example_post_roundtrip_html() {
        let json = include_str!("../example-post.json");
        let post = JsonPost::from_json(json).unwrap();
        let markup = post.to_markup();
        let html = render_markdown(&markup);

        // All text formatting should be in the HTML
        assert!(html.contains("<strong>bold text</strong>"));
        assert!(html.contains("<em>italic text</em>"));
        assert!(html.contains("<u>underlined text</u>"));
        assert!(html.contains("<del>strikethrough text</del>"));
        assert!(html.contains("<sup>superscript text</sup>"));
        assert!(html.contains("<mark>highlighted text</mark>"));
        assert!(html.contains("<code>inline code</code>"));
        assert!(html.contains("class=\"secret\""));

        // Headings
        assert!(html.contains("<h1"));
        assert!(html.contains("<h2"));
        assert!(html.contains("<h3"));
        assert!(html.contains("<h4"));

        // Blockquotes
        assert!(html.contains("<blockquote>"));

        // Links
        assert!(html.contains("href=\"https://example.com\""));
        assert!(html.contains("labeled link</a>"));

        // Images
        assert!(html.contains("<img src="));
        assert!(html.contains("photo.jpg"));
        assert!(html.contains("media-caption"));

        // Videos
        assert!(html.contains("<video"));

        // Lists
        assert!(html.contains("<ul>"));
        assert!(html.contains("<ol>"));
        assert!(html.contains("<li>"));

        // Tables
        assert!(html.contains("<table>"));
        assert!(html.contains("<th"));
        assert!(html.contains("<td"));

        // Code blocks
        assert!(html.contains("<pre"));
        assert!(html.contains("language-rust"));

        // Dividers
        assert!(html.contains("divider-stars"));
        assert!(html.contains("divider-asterisk"));
        assert!(html.contains("divider-thin"));
        assert!(html.contains("divider-double"));

        // Comments should NOT be in HTML
        assert!(!html.contains("visible only in the .md source"));

        // Footnotes
        assert!(html.contains("footnote"));
    }

    #[test]
    fn test_table_rich_cells_roundtrip_html() {
        let json = r#"{
            "body": [{
                "type": "table",
                "columns": [{"header": "Feature"}, {"header": "Status"}],
                "rows": [
                    ["Bold", {"content": [{"type": "bold", "content": [{"type": "text", "text": "yes"}]}]}]
                ]
            }]
        }"#;
        let post = JsonPost::from_json(json).unwrap();
        let markup = post.to_markup();
        let html = render_markdown(&markup);
        assert!(html.contains("<strong>yes</strong>"));
        assert!(html.contains("<table>"));
    }
}
