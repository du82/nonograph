#[derive(Debug, Clone, PartialEq)]
pub enum Node {
    Element { tag: String, children: Vec<Node> },
    Text(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Cursor {
    pub pos: u64,
    pub offset: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SelectionHash {
    pub begin: Cursor,
    pub end: Cursor,
}

impl std::fmt::Display for SelectionHash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "selection-{}.{}-{}.{}",
            self.begin.pos, self.begin.offset, self.end.pos, self.end.offset
        )
    }
}

impl std::str::FromStr for SelectionHash {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.strip_prefix("selection-").ok_or(())?;
        let (begin_part, end_part) = s.split_once('-').ok_or(())?;
        let (bp, bo) = begin_part.split_once('.').ok_or(())?;
        let (ep, eo) = end_part.split_once('.').ok_or(())?;
        Ok(SelectionHash {
            begin: Cursor {
                pos: bp.parse().map_err(|_| ())?,
                offset: bo.parse().map_err(|_| ())?,
            },
            end: Cursor {
                pos: ep.parse().map_err(|_| ())?,
                offset: eo.parse().map_err(|_| ())?,
            },
        })
    }
}

pub fn walk_for_validation<F: FnMut(&Node, u64)>(node: &Node, pos: &mut u64, visitor: &mut F) {
    walk(node, pos, visitor);
}

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
pub fn compute_hash(
    root: &Node,
    begin_pos: u64,
    begin_offset: usize,
    end_pos: u64,
    end_offset: usize,
) -> Option<SelectionHash> {
    if begin_pos == 0 || end_pos == 0 {
        return None;
    }
    Some(SelectionHash {
        begin: Cursor {
            pos: begin_pos,
            offset: begin_offset,
        },
        end: Cursor {
            pos: end_pos,
            offset: end_offset,
        },
    })
}

pub fn positions(root: &Node) -> Vec<(u64, String)> {
    let mut result = Vec::new();
    let mut pos = 0u64;
    walk(root, &mut pos, &mut |node, p| match node {
        Node::Text(t) => result.push((p, t.clone())),
        Node::Element { tag, .. } => result.push((p, format!("<{}>", tag))),
    });
    result
}

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

pub fn hash_for_text_range(
    html: &str,
    start_char: usize,
    end_char: usize,
) -> Option<SelectionHash> {
    let root = parse_html(html);
    let mut char_count = 0usize;
    let mut begin: Option<Cursor> = None;
    let mut end: Option<Cursor> = None;
    let mut pos = 0u64;

    walk(&root, &mut pos, &mut |node, p| {
        if let Node::Text(t) = node {
            let len = t.chars().count();
            let node_start = char_count;
            let node_end = char_count + len;

            if begin.is_none() && start_char >= node_start && start_char < node_end {
                begin = Some(Cursor {
                    pos: p,
                    offset: start_char - node_start,
                });
            }
            if end.is_none() && end_char > node_start && end_char <= node_end {
                end = Some(Cursor {
                    pos: p,
                    offset: end_char - node_start,
                });
            }

            char_count = node_end;
        }
    });

    match (begin, end) {
        (Some(b), Some(e)) if b.pos > 0 && e.pos > 0 => Some(SelectionHash { begin: b, end: e }),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    fn simple_tree() -> Node {
        Node::Element {
            tag: "CONTENT".to_string(),
            children: vec![
                Node::Element {
                    tag: "p".to_string(),
                    children: vec![Node::Text("Hello world".to_string())],
                },
                Node::Element {
                    tag: "p".to_string(),
                    children: vec![Node::Text("Second paragraph".to_string())],
                },
            ],
        }
    }

    #[test]
    fn test_positions_are_nonzero() {
        let tree = simple_tree();
        let pos = positions(&tree);
        assert!(pos.iter().all(|(p, _)| *p > 0));
    }

    #[test]
    fn test_element_positions_are_even() {
        let tree = simple_tree();
        let pos = positions(&tree);
        for (p, label) in &pos {
            if label.starts_with('<') {
                assert_eq!(p % 2, 0, "element pos {} should be even", p);
            }
        }
    }

    #[test]
    fn test_text_positions_are_odd() {
        let tree = simple_tree();
        let pos = positions(&tree);
        for (p, label) in &pos {
            if !label.starts_with('<') {
                assert_eq!(p % 2, 1, "text pos {} should be odd", p);
            }
        }
    }

    #[test]
    fn test_display_roundtrip() {
        let hash = SelectionHash {
            begin: Cursor { pos: 3, offset: 2 },
            end: Cursor { pos: 7, offset: 5 },
        };
        let s = hash.to_string();
        assert_eq!(s, "selection-3.2-7.5");
        let parsed = SelectionHash::from_str(&s).unwrap();
        assert_eq!(parsed, hash);
    }

    #[test]
    fn test_parse_invalid() {
        assert!(SelectionHash::from_str("notaselection").is_err());
        assert!(SelectionHash::from_str("selection-abc.0-1.0").is_err());
    }

    #[test]
    fn test_hash_for_text_range() {
        let html = "<p>Hello world</p><p>Second paragraph</p>";
        let hash = hash_for_text_range(html, 0, 5).unwrap();
        assert_eq!(hash.begin.offset, 0);
        assert_eq!(hash.end.offset, 5);
        assert_eq!(hash.begin.pos % 2, 1);
    }

    #[test]
    fn test_hash_for_cross_node_range() {
        let html = "<p>Hello</p><p>World</p>";
        let hash = hash_for_text_range(html, 0, 10).unwrap();
        assert_ne!(hash.begin.pos, hash.end.pos);
    }

    #[test]
    fn test_old_meta_skipped() {
        let tree = Node::Element {
            tag: "CONTENT".to_string(),
            children: vec![
                Node::Element {
                    tag: "old-meta".to_string(),
                    children: vec![Node::Text("should not appear".to_string())],
                },
                Node::Element {
                    tag: "p".to_string(),
                    children: vec![Node::Text("visible".to_string())],
                },
            ],
        };
        let pos = positions(&tree);
        assert!(!pos.iter().any(|(_, label)| label == "should not appear"));
        assert!(pos.iter().any(|(_, label)| label == "visible"));
    }

    #[test]
    fn test_positions_stable() {
        let tree = simple_tree();
        let a = positions(&tree);
        let b = positions(&tree);
        assert_eq!(a, b);
    }

    #[test]
    fn test_parse_html_structure() {
        let html = "<p>Hello</p><p>World</p>";
        let root = parse_html(html);
        if let Node::Element { children, .. } = root {
            assert_eq!(children.len(), 2);
        } else {
            panic!("expected element");
        }
    }

    #[test]
    fn test_void_tags_have_no_children() {
        let html = "<p>Before<br>After</p>";
        let root = parse_html(html);
        if let Node::Element { children, .. } = root {
            if let Node::Element {
                children: p_children,
                ..
            } = &children[0]
            {
                let br = p_children
                    .iter()
                    .find(|n| matches!(n, Node::Element { tag, .. } if tag == "br"));
                if let Some(Node::Element {
                    children: br_children,
                    ..
                }) = br
                {
                    assert!(br_children.is_empty());
                }
            }
        }
    }
}
