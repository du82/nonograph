pub fn render_markdown(content: &str) -> String {
    let mut result = content.to_string();

    // Process in order to avoid conflicts - longer patterns first
    // Italics: **text** -> <em>text</em>
    result = safe_replace(&result, "**", "**", "<em>", "</em>");

    // Bold: *text* -> <strong>text</strong>
    result = safe_replace(&result, "*", "*", "<strong>", "</strong>");

    // Underscore: _text_ -> <u>text</u>
    result = safe_replace(&result, "_", "_", "<u>", "</u>");

    // Strikethrough: ~text~ -> <del>text</del>
    result = safe_replace(&result, "~", "~", "<del>", "</del>");

    // Superscript: ^text^ -> <sup>text</sup>
    result = safe_replace(&result, "^", "^", "<sup>", "</sup>");

    // Inline code: `text` -> <code>text</code>
    result = safe_replace(&result, "`", "`", "<code>", "</code>");

    // Secret text: #text# -> <span class="secret">text</span>
    result = safe_replace(&result, "#", "#", "<span class=\"secret\">", "</span>");

    // Links with text: (text)[url] -> <a href="url">text</a>
    result = process_links(&result);

    // Auto-embed images and videos
    result = process_media_urls(&result);

    // Replace line breaks with HTML line breaks
    result = result.replace('\n', "<br>\n");

    // Sanitize the HTML to prevent XSS
    ammonia::clean(&result)
}

fn safe_replace(
    text: &str,
    start_pattern: &str,
    end_pattern: &str,
    open_tag: &str,
    close_tag: &str,
) -> String {
    let mut result = String::new();
    let chars: Vec<char> = text.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        let remaining: String = chars[i..].iter().collect();

        if let Some(start_pos) = remaining.find(start_pattern) {
            let actual_start = i + start_pos;
            let search_from = actual_start + start_pattern.chars().count();

            // Add text before the pattern
            result.push_str(&chars[i..actual_start].iter().collect::<String>());

            if search_from < chars.len() {
                let remaining_from_search: String = chars[search_from..].iter().collect();
                if let Some(end_pos) = remaining_from_search.find(end_pattern) {
                    let actual_end = search_from + end_pos;
                    let content: String = chars[search_from..actual_end].iter().collect();

                    // Don't process if content is empty or contains newlines
                    if !content.is_empty() && !content.contains('\n') {
                        result.push_str(&format!("{}{}{}", open_tag, content, close_tag));
                        i = actual_end + end_pattern.chars().count();
                        continue;
                    }
                }
            }

            // If we get here, add the start pattern and continue
            result.push_str(start_pattern);
            i = actual_start + start_pattern.chars().count();
        } else {
            // No more patterns found, add the rest
            result.push_str(&chars[i..].iter().collect::<String>());
            break;
        }
    }

    result
}

fn process_links(text: &str) -> String {
    let mut result = String::new();
    let chars: Vec<char> = text.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        // Look for (text)[url] pattern
        if chars[i] == '(' {
            let mut paren_end = None;
            for j in (i + 1)..chars.len() {
                if chars[j] == ')' {
                    paren_end = Some(j);
                    break;
                }
            }

            if let Some(paren_end_idx) = paren_end {
                if paren_end_idx + 1 < chars.len() && chars[paren_end_idx + 1] == '[' {
                    let mut bracket_end = None;
                    for j in (paren_end_idx + 2)..chars.len() {
                        if chars[j] == ']' {
                            bracket_end = Some(j);
                            break;
                        }
                    }

                    if let Some(bracket_end_idx) = bracket_end {
                        let link_text: String = chars[(i + 1)..paren_end_idx].iter().collect();
                        let link_url: String =
                            chars[(paren_end_idx + 2)..bracket_end_idx].iter().collect();

                        if !link_text.is_empty() && !link_url.is_empty() && link_url.len() <= 4096 {
                            result.push_str(&format!("<a href=\"{}\">{}</a>", link_url, link_text));
                            i = bracket_end_idx + 1;
                            continue;
                        }
                    }
                }
            }
        }

        // Look for [url] pattern (not part of (text)[url])
        if chars[i] == '[' && (i == 0 || chars[i - 1] != ')') {
            let mut bracket_end = None;
            for j in (i + 1)..chars.len() {
                if chars[j] == ']' {
                    bracket_end = Some(j);
                    break;
                }
            }

            if let Some(bracket_end_idx) = bracket_end {
                let link_url: String = chars[(i + 1)..bracket_end_idx].iter().collect();

                if link_url.starts_with("http")
                    && link_url.len() <= 4096
                    && !link_url.contains('\n')
                {
                    result.push_str(&format!("<a href=\"{}\">{}</a>", link_url, link_url));
                    i = bracket_end_idx + 1;
                    continue;
                }
            }
        }

        // No pattern matched, add current character
        result.push(chars[i]);
        i += 1;
    }

    result
}

fn process_media_urls(text: &str) -> String {
    let mut result = text.to_string();

    // Image extensions
    let image_extensions = ["jpg", "jpeg", "png", "gif", "webp"];
    let video_extensions = ["mp4", "webm", "ogg"];

    for ext in &image_extensions {
        result = simple_url_replace(&result, ext, true);
    }

    for ext in &video_extensions {
        result = simple_url_replace(&result, ext, false);
    }

    result
}

fn simple_url_replace(text: &str, extension: &str, is_image: bool) -> String {
    let mut result = text.to_string();
    let pattern = format!(".{}", extension);

    let chars: Vec<char> = result.chars().collect();
    let mut i = 0;
    let mut new_chars = Vec::new();

    while i < chars.len() {
        if i + 4 <= chars.len() && chars[i..i + 4].iter().collect::<String>() == "http" {
            // Find end of URL
            let url_start = i;
            let mut url_end = i;

            while url_end < chars.len() && !chars[url_end].is_whitespace() {
                url_end += 1;
            }

            let url: String = chars[url_start..url_end].iter().collect();
            if url.ends_with(&pattern) {
                let replacement = if is_image {
                    format!("<img src=\"{}\" alt=\"Image\">", url)
                } else {
                    format!("<video controls><source src=\"{}\" type=\"video/mp4\">Your browser does not support the video tag.</video>", url)
                };
                new_chars.extend(replacement.chars());
                i = url_end;
            } else {
                new_chars.push(chars[i]);
                i += 1;
            }
        } else {
            new_chars.push(chars[i]);
            i += 1;
        }
    }

    result = new_chars.into_iter().collect();

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_formatting() {
        assert_eq!(
            render_markdown("*bold* text").contains("<strong>bold</strong>"),
            true
        );
        assert_eq!(
            render_markdown("**italic** text").contains("<em>italic</em>"),
            true
        );
        assert_eq!(
            render_markdown("_underline_ text").contains("<u>underline</u>"),
            true
        );
    }

    #[test]
    fn test_unicode_handling() {
        let japanese = "渋い美しさ *bold* text";
        let result = render_markdown(japanese);
        assert!(result.contains("渋い美しさ"));
        assert!(result.contains("<strong>bold</strong>"));
    }

    #[test]
    fn test_links() {
        let text = "(Google)[https://google.com]";
        let result = render_markdown(text);
        assert!(result.contains("<a href=\"https://google.com\""));
        assert!(result.contains(">Google</a>"));
    }

    #[test]
    fn test_simple_links() {
        let text = "[https://example.com]";
        let result = render_markdown(text);
        assert!(result.contains("<a href=\"https://example.com\""));
        assert!(result.contains(">https://example.com</a>"));
    }

    #[test]
    fn test_secret_text() {
        let text = "#secret message#";
        let result = render_markdown(text);
        assert!(result.contains("<span class=\"secret\">secret message</span>"));
    }

    #[test]
    fn test_code() {
        let text = "`code block`";
        let result = render_markdown(text);
        assert!(result.contains("<code>code block</code>"));
    }

    #[test]
    fn test_media_embedding() {
        let text = "https://example.com/image.jpg";
        let result = render_markdown(text);
        assert!(result.contains("<img src=\"https://example.com/image.jpg\""));
    }

    #[test]
    fn test_line_breaks() {
        let text = "Line one\nLine two";
        let result = render_markdown(text);
        assert!(result.contains("Line one<br>\nLine two"));
    }
}
