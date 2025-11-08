pub fn strip_javascript(html: &str) -> String {
    let mut result = String::with_capacity(html.len());
    let mut chars = html.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '<' {
            let mut tag_chars = Vec::new();
            tag_chars.push(ch);

            let mut temp_chars = chars.clone();
            let mut is_script_tag = false;
            let mut is_end_tag = false;

            if let Some('/') = temp_chars.peek() {
                tag_chars.push(temp_chars.next().unwrap());
                is_end_tag = true;
            }

            let mut tag_name = String::new();
            while let Some(&next_ch) = temp_chars.peek() {
                if next_ch.is_whitespace() || next_ch == '>' || next_ch == '/' {
                    break;
                }
                tag_name.push(temp_chars.next().unwrap());
                tag_chars.push(tag_name.chars().last().unwrap());
            }

            if tag_name.to_lowercase() == "script" {
                is_script_tag = true;
            }

            if is_script_tag {
                if is_end_tag {
                    while let Some(ch) = chars.next() {
                        if ch == '>' {
                            break;
                        }
                    }
                } else {
                    while let Some(ch) = chars.next() {
                        if ch == '>' {
                            break;
                        }
                    }

                    let mut in_script = true;
                    while in_script && chars.peek().is_some() {
                        if let Some('<') = chars.peek() {
                            let mut temp_chars = chars.clone();
                            temp_chars.next();

                            if let Some('/') = temp_chars.peek() {
                                temp_chars.next();

                                let mut closing_tag = String::new();
                                while let Some(&next_ch) = temp_chars.peek() {
                                    if next_ch.is_whitespace() || next_ch == '>' {
                                        break;
                                    }
                                    closing_tag.push(temp_chars.next().unwrap());
                                }

                                if closing_tag.to_lowercase() == "script" {
                                    chars.next();
                                    chars.next();
                                    for _ in 0..6 {
                                        chars.next();
                                    }
                                    while let Some(ch) = chars.next() {
                                        if ch == '>' {
                                            break;
                                        }
                                    }
                                    in_script = false;
                                }
                            }
                        }

                        if in_script {
                            chars.next();
                        }
                    }
                }
            } else {
                result.push(ch);
            }
        } else {
            result.push(ch);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_simple_script() {
        let html =
            r#"<html><head><script>alert('test');</script></head><body>Content</body></html>"#;
        let result = strip_javascript(html);
        assert!(!result.contains("<script"));
        assert!(!result.contains("alert"));
        assert!(result.contains("Content"));
    }

    #[test]
    fn test_strip_script_with_attributes() {
        let html = r#"<div><script type="text/javascript" src="file.js">console.log("hi");</script><p>Keep this</p></div>"#;
        let result = strip_javascript(html);
        assert!(!result.contains("<script"));
        assert!(!result.contains("console.log"));
        assert!(result.contains("Keep this"));
    }

    #[test]
    fn test_strip_multiple_scripts() {
        let html = r#"<html>
        <head><script>var x = 1;</script></head>
        <body>
            <p>Content</p>
            <script>alert('hello');</script>
            <div>More content</div>
        </body>
        </html>"#;
        let result = strip_javascript(html);
        assert!(!result.contains("<script"));
        assert!(!result.contains("var x"));
        assert!(!result.contains("alert"));
        assert!(result.contains("Content"));
        assert!(result.contains("More content"));
    }

    #[test]
    fn test_case_insensitive_script_tags() {
        let html = r#"<SCRIPT>alert('test');</SCRIPT><Script>console.log();</Script>"#;
        let result = strip_javascript(html);
        assert!(!result.contains("SCRIPT"));
        assert!(!result.contains("Script"));
        assert!(!result.contains("alert"));
        assert!(!result.contains("console.log"));
    }

    #[test]
    fn test_no_script_tags() {
        let html =
            r#"<html><head><title>Test</title></head><body><p>No scripts here</p></body></html>"#;
        let result = strip_javascript(html);
        assert_eq!(html, result);
    }

    #[test]
    fn test_script_in_text_content() {
        let html =
            r#"<p>This mentions script tags but isn't one</p><script>alert('remove me');</script>"#;
        let result = strip_javascript(html);
        assert!(result.contains("This mentions script tags"));
        assert!(!result.contains("alert"));
        assert!(!result.contains("<script"));
    }

    #[test]
    fn test_malformed_script_tags() {
        let html = r#"<script>unclosed script<div>content</div>"#;
        let result = strip_javascript(html);
        // Should remove everything after <script> since there's no proper closing
        assert!(!result.contains("<script"));
        assert!(!result.contains("unclosed script"));
        assert!(!result.contains("content"));
    }
}
