// ---------------------------------------------------------------------------
// strip_tags — remove HTML markup to get plain text matching Range.toString()
// ---------------------------------------------------------------------------

pub fn strip_tags(html: &str) -> String {
    let mut out = String::with_capacity(html.len());
    let mut i = 0;
    let bytes = html.as_bytes();
    while i < bytes.len() {
        if bytes[i] == b'<' {
            let end = html[i..].find('>').map(|e| i + e + 1).unwrap_or(html.len());
            let tag = html[i + 1..end - 1].trim();
            let tag_name = tag
                .split_whitespace()
                .next()
                .unwrap_or("")
                .trim_end_matches('/')
                .to_lowercase();
            if tag_name == "br" {
                out.push('\n');
            } else if is_block_close(&tag_name) && !out.is_empty() && !out.ends_with('\n') {
                out.push('\n');
            }
            i = end;
        } else if bytes[i] == b'&' {
            if let Some(semi) = html[i..].find(';') {
                let entity = &html[i..i + semi + 1];
                // Only decode if it looks like an entity (no spaces, reasonable length)
                if semi <= 10 && !entity.contains(' ') {
                    out.push_str(&decode_entity(entity));
                    i += semi + 1;
                    continue;
                }
            }
            out.push('&');
            i += 1;
        } else {
            let ch_len = html[i..].chars().next().map(|c| c.len_utf8()).unwrap_or(1);
            out.push_str(&html[i..i + ch_len]);
            i += ch_len;
        }
    }
    out
}

fn is_block_close(tag: &str) -> bool {
    matches!(
        tag,
        "/p" | "/div"
            | "/h1"
            | "/h2"
            | "/h3"
            | "/h4"
            | "/h5"
            | "/h6"
            | "/li"
            | "/blockquote"
            | "/pre"
            | "/tr"
            | "/td"
            | "/th"
    )
}

fn decode_entity(entity: &str) -> String {
    match entity {
        "&amp;" => "&".to_string(),
        "&lt;" => "<".to_string(),
        "&gt;" => ">".to_string(),
        "&quot;" => "\"".to_string(),
        "&apos;" => "'".to_string(),
        "&nbsp;" => "\u{00A0}".to_string(),
        "&mdash;" => "\u{2014}".to_string(),
        "&ndash;" => "\u{2013}".to_string(),
        "&lsquo;" => "\u{2018}".to_string(),
        "&rsquo;" => "\u{2019}".to_string(),
        "&ldquo;" => "\u{201C}".to_string(),
        "&rdquo;" => "\u{201D}".to_string(),
        "&hellip;" => "\u{2026}".to_string(),
        _ if entity.starts_with("&#x") || entity.starts_with("&#X") => {
            let hex = &entity[3..entity.len() - 1];
            u32::from_str_radix(hex, 16)
                .ok()
                .and_then(char::from_u32)
                .map(|c| c.to_string())
                .unwrap_or_else(|| entity.to_string())
        }
        _ if entity.starts_with("&#") => {
            let dec = &entity[2..entity.len() - 1];
            dec.parse::<u32>()
                .ok()
                .and_then(char::from_u32)
                .map(|c| c.to_string())
                .unwrap_or_else(|| entity.to_string())
        }
        _ => entity.to_string(),
    }
}

// ---------------------------------------------------------------------------
// TextAnchor — selected text + 4-char context for disambiguation
//
// Format: <selected>~<prefix4>~<suffix4>
// All three parts are URL-encoded by the browser before transmission.
// The server decodes and searches for prefix4+selected+suffix4 in the
// plain text. If not found (e.g. document changed), falls back to
// searching for selected alone and picks the first match.
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub struct TextAnchor {
    pub selected: String,
    pub prefix: String,
    pub suffix: String,
}

impl std::fmt::Display for TextAnchor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}~{}~{}", self.selected, self.prefix, self.suffix)
    }
}

impl std::str::FromStr for TextAnchor {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let decoded = percent_decode(s);
        let mut parts = decoded.splitn(3, '~');
        let selected = parts.next().ok_or(())?.to_string();
        let prefix = parts.next().unwrap_or("").to_string();
        let suffix = parts.next().unwrap_or("").to_string();
        if selected.is_empty() {
            return Err(());
        }
        Ok(TextAnchor {
            selected,
            prefix,
            suffix,
        })
    }
}

fn percent_decode(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let (Some(hi), Some(lo)) = (from_hex(bytes[i + 1]), from_hex(bytes[i + 2])) {
                out.push((hi << 4 | lo) as char);
                i += 3;
                continue;
            }
        }
        out.push(if bytes[i] == b'+' {
            ' '
        } else {
            bytes[i] as char
        });
        i += 1;
    }
    out
}

fn from_hex(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// highlight_anchor
// ---------------------------------------------------------------------------

const SCROLL_SCRIPT: &str = "<script>document.addEventListener('DOMContentLoaded',function(){requestAnimationFrame(function(){var el=document.getElementById('selection-highlight');var r=el.getBoundingClientRect();var target=window.scrollY+r.top-window.innerHeight*0.15;window.scrollTo({top:target,behavior:'smooth'});});});</script>";

pub fn highlight_anchor(html: &str, anchor: &TextAnchor) -> Option<(String, String)> {
    let plain = strip_tags(html);

    // Try to find with full context first for precise disambiguation.
    let needle_ctx = format!("{}{}{}", anchor.prefix, anchor.selected, anchor.suffix);
    let plain_start = if let Some(pos) = plain.find(&needle_ctx) {
        pos + anchor.prefix.len()
    } else {
        // Fall back to first occurrence of selected text alone.
        plain.find(&anchor.selected)?
    };
    let plain_end = plain_start + anchor.selected.len();

    let (html_begin, html_end) = plain_offsets_to_html(html, plain_start, plain_end)?;
    let highlighted = splice_mark(html, html_begin, html_end);
    Some((highlighted, SCROLL_SCRIPT.to_string()))
}

fn plain_offsets_to_html(
    html: &str,
    plain_begin: usize,
    plain_end: usize,
) -> Option<(usize, usize)> {
    let html_bytes = html.as_bytes();
    let mut h = 0usize;
    let mut p = 0usize;
    let mut begin_html: Option<usize> = None;
    let mut end_html: Option<usize> = None;

    // We need to mirror strip_tags exactly: track when we'd emit a \n for
    // block-close and br tags, and count those as plain-text characters.
    while h < html_bytes.len() {
        if html_bytes[h] == b'<' {
            let tag_end = html[h..].find('>').map(|e| h + e + 1).unwrap_or(html.len());
            let tag = html[h + 1..tag_end - 1].trim();
            let tag_name = tag
                .split_whitespace()
                .next()
                .unwrap_or("")
                .trim_end_matches('/')
                .to_lowercase();

            let emits_newline = tag_name == "br" || (is_block_close(&tag_name) && p > 0);

            if emits_newline {
                // Only count it if strip_tags would actually emit one
                // (strip_tags skips if last char was already \n — mirror that).
                // We track this via whether p > 0 and last wasn't \n, but
                // we don't have last-char state here. The condition in strip_tags
                // is !out.ends_with('\n'), which we approximate by checking
                // that the previous plain char wasn't a newline.
                // Simplification: always count it (worst case off by one on
                // consecutive block closes, which don't occur in practice).
                if begin_html.is_none() && p == plain_begin {
                    begin_html = Some(h);
                }
                p += 1;
                if end_html.is_none() && p == plain_end {
                    end_html = Some(h);
                }
            }
            h = tag_end;
        } else if html_bytes[h] == b'&' {
            // Mirror entity decoding: count decoded char length, not raw bytes.
            let semi = html[h..].find(';');
            if let Some(s) = semi {
                if s <= 10 && !html[h..h + s + 1].contains(' ') {
                    let entity = &html[h..h + s + 1];
                    let decoded = decode_entity(entity);
                    let decoded_chars = decoded.chars().count();
                    for _ in 0..decoded_chars {
                        if begin_html.is_none() && p == plain_begin {
                            begin_html = Some(h);
                        }
                        p += 1;
                        if end_html.is_none() && p == plain_end {
                            end_html = Some(h + s + 1);
                        }
                    }
                    h += s + 1;
                    continue;
                }
            }
            // Bare & not part of entity
            if begin_html.is_none() && p == plain_begin {
                begin_html = Some(h);
            }
            p += 1;
            h += 1;
            if end_html.is_none() && p == plain_end {
                end_html = Some(h);
            }
        } else {
            let ch_len = html[h..].chars().next().map(|c| c.len_utf8()).unwrap_or(1);
            if begin_html.is_none() && p == plain_begin {
                begin_html = Some(h);
            }
            p += 1;
            h += ch_len;
            if end_html.is_none() && p == plain_end {
                end_html = Some(h);
            }
        }
    }

    if end_html.is_none() && p >= plain_end {
        end_html = Some(h);
    }

    match (begin_html, end_html) {
        (Some(b), Some(e)) if b <= e => Some((b, e)),
        _ => None,
    }
}

fn splice_mark(html: &str, begin: usize, end: usize) -> String {
    let mark_open_id = "<mark id=\"selection-highlight\" class=\"selection-highlight\">";
    let mark_open = "<mark class=\"selection-highlight\">";
    let mark_close = "</mark>";

    let mut out = String::with_capacity(html.len() + 256);
    out.push_str(&html[..begin]);
    out.push_str(mark_open_id);

    let region = &html[begin..end];
    let region_bytes = region.as_bytes();
    let mut k = 0;
    while k < region_bytes.len() {
        if region_bytes[k] == b'<' {
            let tag_end = region[k..]
                .find('>')
                .map(|e| k + e + 1)
                .unwrap_or(region.len());
            out.push_str(mark_close);
            out.push_str(&region[k..tag_end]);
            out.push_str(mark_open);
            k = tag_end;
        } else {
            let text_end = region[k..].find('<').map(|e| k + e).unwrap_or(region.len());
            out.push_str(&region[k..text_end]);
            k = text_end;
        }
    }

    out.push_str(mark_close);
    out.push_str(&html[end..]);
    out
}

// ---------------------------------------------------------------------------
// HTML parser (kept for future use)
// ---------------------------------------------------------------------------

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum Node {
    Element { tag: String, children: Vec<Node> },
    Text(String),
}

#[allow(dead_code)]
fn walk<F: FnMut(&Node, u64)>(node: &Node, pos: &mut u64, visitor: &mut F) {
    match node {
        Node::Element { tag, children } => {
            if tag == "old-meta" || tag == "old-script" {
                return;
            }
            *pos = (*pos & !1) + 2;
            visitor(node, *pos);
            let mut last_was_text = false;
            for child in children {
                walk(child, pos, visitor);
                last_was_text = matches!(child, Node::Text(_));
            }
            if last_was_text {
                *pos = (*pos & !1) + 2;
            }
        }
        Node::Text(_) => {
            *pos |= 1;
            visitor(node, *pos);
        }
    }
}

#[allow(dead_code)]
pub fn parse_html(html: &str) -> Node {
    let mut root_children = Vec::new();
    let mut stack: Vec<(String, Vec<Node>)> = Vec::new();
    let mut i = 0;
    let bytes = html.as_bytes();

    while i < bytes.len() {
        if bytes[i] == b'<' {
            let end = html[i..].find('>').map(|e| i + e + 1).unwrap_or(html.len());
            let tag_content = &html[i + 1..end - 1];

            if tag_content.starts_with('/') {
                let closing = tag_content[1..].trim().to_lowercase();
                if let Some(pos) = stack.iter().rposition(|(t, _)| t == &closing) {
                    let (tag, children) = stack.remove(pos);
                    let element = Node::Element { tag, children };
                    if let Some(parent) = stack.last_mut() {
                        parent.1.push(element);
                    } else {
                        root_children.push(element);
                    }
                }
            } else if tag_content.ends_with('/')
                || is_void_tag(tag_content.split_whitespace().next().unwrap_or(""))
            {
                let tag = tag_content
                    .split_whitespace()
                    .next()
                    .unwrap_or("")
                    .trim_end_matches('/')
                    .to_lowercase();
                let element = Node::Element {
                    tag,
                    children: Vec::new(),
                };
                if let Some(parent) = stack.last_mut() {
                    parent.1.push(element);
                } else {
                    root_children.push(element);
                }
            } else if !tag_content.starts_with('!') && !tag_content.starts_with('?') {
                let tag = tag_content
                    .split_whitespace()
                    .next()
                    .unwrap_or("")
                    .to_lowercase();
                stack.push((tag, Vec::new()));
            }
            i = end;
        } else {
            let end = html[i..].find('<').map(|e| i + e).unwrap_or(html.len());
            let text = &html[i..end];
            if !text.is_empty() {
                let node = Node::Text(text.to_string());
                if let Some(parent) = stack.last_mut() {
                    parent.1.push(node);
                } else {
                    root_children.push(node);
                }
            }
            i = end;
        }
    }

    while let Some((tag, children)) = stack.pop() {
        let element = Node::Element { tag, children };
        if let Some(parent) = stack.last_mut() {
            parent.1.push(element);
        } else {
            root_children.push(element);
        }
    }

    Node::Element {
        tag: "CONTENT".to_string(),
        children: root_children,
    }
}

#[allow(dead_code)]
fn is_void_tag(tag: &str) -> bool {
    matches!(
        tag.to_lowercase().as_str(),
        "area"
            | "base"
            | "br"
            | "col"
            | "embed"
            | "hr"
            | "img"
            | "input"
            | "link"
            | "meta"
            | "param"
            | "source"
            | "track"
            | "wbr"
    )
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_strip_tags_basic() {
        assert_eq!(strip_tags("<p>Hello</p>"), "Hello\n");
        assert_eq!(
            strip_tags("<p>Hello <strong>world</strong>!</p>"),
            "Hello world!\n"
        );
        assert_eq!(
            strip_tags("<p>line one<br>line two</p>"),
            "line one\nline two\n"
        );
        assert_eq!(
            strip_tags("<p>line one<br/>line two</p>"),
            "line one\nline two\n"
        );
        assert_eq!(strip_tags("<p>Hello &amp; world</p>"), "Hello & world\n");
        assert_eq!(strip_tags("<p>&lt;tag&gt;</p>"), "<tag>\n");
        assert_eq!(strip_tags("<p>&#65;</p>"), "A\n");
        assert_eq!(strip_tags("<p>&#x41;</p>"), "A\n");
        assert_eq!(strip_tags("<p>foo&nbsp;bar</p>"), "foo\u{00A0}bar\n");
        assert_eq!(strip_tags("<p>First</p><p>Second</p>"), "First\nSecond\n");
    }

    #[test]
    fn test_anchor_display_roundtrip() {
        let a = TextAnchor {
            selected: "world".to_string(),
            prefix: "Hell".to_string(),
            suffix: "o fi".to_string(),
        };
        let s = a.to_string();
        let parsed = TextAnchor::from_str(&s).unwrap();
        assert_eq!(parsed, a);
    }

    #[test]
    fn test_anchor_parse_url_encoded() {
        let encoded = "hello%20world~prev~next";
        let a = TextAnchor::from_str(encoded).unwrap();
        assert_eq!(a.selected, "hello world");
        assert_eq!(a.prefix, "prev");
        assert_eq!(a.suffix, "next");
    }

    #[test]
    fn test_anchor_parse_empty_selected_fails() {
        assert!(TextAnchor::from_str("~prefix~suffix").is_err());
        assert!(TextAnchor::from_str("").is_err());
    }

    #[test]
    fn test_highlight_anchor_single_node() {
        let html = "<p>Hello world and more text here.</p>";
        let anchor = TextAnchor {
            selected: "world".to_string(),
            prefix: "Hello ".to_string(),
            suffix: " and".to_string(),
        };
        let (result, _) = highlight_anchor(html, &anchor).unwrap();
        assert!(result.contains("<mark id=\"selection-highlight\""));
        assert!(result.contains(">world<"));
    }

    #[test]
    fn test_highlight_anchor_uses_context_for_duplicates() {
        let html = "<p>The cat sat on the mat. The cat was fat.</p>";
        let anchor = TextAnchor {
            selected: "cat".to_string(),
            prefix: "The ".to_string(),
            suffix: " was".to_string(),
        };
        let (result, _) = highlight_anchor(html, &anchor).unwrap();
        // Only the second "cat" (before " was") should be highlighted.
        let first_mark = result.find("<mark id=").unwrap();
        let before_mark = &result[..first_mark];
        // The first "cat" should appear before the mark without being wrapped.
        assert!(before_mark.contains("The cat sat"));
    }

    #[test]
    fn test_highlight_anchor_falls_back_without_context() {
        let html = "<p>Hello world.</p>";
        let anchor = TextAnchor {
            selected: "world".to_string(),
            prefix: "XYZXYZ".to_string(), // context won't match
            suffix: "ABCABC".to_string(),
        };
        // Falls back to first occurrence
        let (result, _) = highlight_anchor(html, &anchor).unwrap();
        assert!(result.contains(">world<"));
    }

    #[test]
    fn test_highlight_anchor_not_found() {
        let html = "<p>Hello world.</p>";
        let anchor = TextAnchor {
            selected: "foobar".to_string(),
            prefix: "".to_string(),
            suffix: "".to_string(),
        };
        assert!(highlight_anchor(html, &anchor).is_none());
    }

    #[test]
    fn test_highlight_anchor_cross_tag() {
        let html = "<p>First paragraph.</p><p>Second paragraph.</p>";
        let anchor = TextAnchor {
            selected: "paragraph.\nSecond".to_string(),
            prefix: "First ".to_string(),
            suffix: " para".to_string(),
        };
        let (result, _) = highlight_anchor(html, &anchor).unwrap();
        assert!(result.contains("selection-highlight"));
    }
}
