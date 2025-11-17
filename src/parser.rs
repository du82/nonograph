fn process_images(text: &str) -> String {
    process_images_with_config(text, &crate::config::Config::default())
}

fn is_safe_url(url: &str) -> bool {
    if !url.contains("://") {
        return true;
    }

    // Block dangerous protocols
    if url.starts_with("javascript:")
        || url.starts_with("data:")
        || url.starts_with("file:")
        || url.starts_with("ftp:")
    {
        return false;
    }

    // Only allow http and https for absolute URLs
    if !url.starts_with("http://") && !url.starts_with("https://") {
        return false;
    }

    // Parse URL to extract host
    if let Some(host_start) = url.find("://").map(|i| i + 3) {
        let remaining = &url[host_start..];
        let host = if let Some(slash_pos) = remaining.find('/') {
            &remaining[..slash_pos]
        } else {
            remaining
        };

        // Remove port if present
        let host = if let Some(colon_pos) = host.find(':') {
            &host[..colon_pos]
        } else {
            host
        };

        // Block localhost variations
        if host == "localhost" || host == "127.0.0.1" || host.starts_with("127.") {
            return false;
        }

        // Block private IP ranges
        if host.starts_with("192.168.") || host.starts_with("10.") || host.starts_with("172.") {
            // Check if it's in 172.16.0.0/12 range
            if host.starts_with("172.") {
                if let Some(third_octet_start) = host[4..].find('.') {
                    if let Ok(second_octet) = host[4..4 + third_octet_start].parse::<u8>() {
                        if second_octet >= 16 && second_octet <= 31 {
                            return false;
                        }
                    }
                }
            } else {
                return false;
            }
        }

        // Block link-local addresses (169.254.0.0/16)
        if host.starts_with("169.254.") {
            return false;
        }

        // Block other internal addresses
        if host == "0.0.0.0" || host.starts_with("0.") {
            return false;
        }
    }

    true
}

pub fn render_markdown(content: &str) -> String {
    let (protected_content, fenced_blocks) = extract_fenced_code_blocks(content);
    let (mut working_content, code_blocks) = extract_code_blocks(&protected_content);

    // Process comments before other formatting to remove them from HTML output
    working_content = process_comments(&working_content);

    // Process footnotes before text formatting to avoid conflicts with ^ and []
    working_content = process_footnotes(&working_content);

    working_content = safe_replace(&working_content, "**", "**", "<strong>", "</strong>");
    working_content = safe_replace(&working_content, "*", "*", "<em>", "</em>");
    working_content = safe_replace(&working_content, "_", "_", "<u>", "</u>");
    working_content = safe_replace(&working_content, "~", "~", "<del>", "</del>");
    working_content = safe_replace(&working_content, "^", "^", "<sup>", "</sup>");
    working_content = safe_replace(&working_content, "==", "==", "<mark>", "</mark>");
    working_content = safe_replace(
        &working_content,
        "#",
        "#",
        "<span class=\"secret\">",
        "</span>",
    );

    working_content = process_images(&working_content);
    working_content = process_links(&working_content);
    working_content = process_tables(&working_content);
    working_content = process_dividers(&working_content);
    working_content = format_paragraphs_with_headers(&working_content);
    working_content = restore_fenced_code_blocks(&working_content, &fenced_blocks);
    working_content = restore_code_blocks(&working_content, &code_blocks);
    working_content = restore_footnotes(&working_content);

    sanitize_html(working_content)
}

fn process_images_with_config(text: &str, config: &crate::config::Config) -> String {
    let mut result = String::with_capacity(text.len() + 1024);
    let chars: Vec<char> = text.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        if chars.len() >= 2 && i < chars.len() - 1 && chars[i] == '!' && chars[i + 1] == '[' {
            // Find closing bracket
            let mut bracket_end = None;
            let mut j = i + 2;
            while j < chars.len() && chars[j] != '\n' {
                if chars[j] == ']' {
                    bracket_end = Some(j);
                    break;
                }
                j += 1;
            }

            if let Some(bracket_end_idx) = bracket_end {
                // Check for ![alt](url) pattern
                if bracket_end_idx + 1 < chars.len() && chars[bracket_end_idx + 1] == '(' {
                    let mut paren_end = None;
                    let mut k = bracket_end_idx + 2;
                    while k < chars.len() && chars[k] != '\n' {
                        if chars[k] == ')' {
                            paren_end = Some(k);
                            break;
                        }
                        k += 1;
                    }

                    if let Some(paren_end_idx) = paren_end {
                        let alt_text: String = chars[(i + 2)..bracket_end_idx].iter().collect();
                        let image_url: String =
                            chars[(bracket_end_idx + 2)..paren_end_idx].iter().collect();

                        if !image_url.is_empty()
                            && image_url.len() <= config.security.max_url_length
                            && is_safe_url(&image_url)
                        {
                            let is_video = is_video_url(&image_url);

                            // Check if alt text is present for caption
                            if !alt_text.trim().is_empty() {
                                result.push_str("<div class=\"media-with-caption\">");
                                if is_video {
                                    result.push_str("<video controls style=\"width: 100%;\">");
                                    result.push_str("<source src=\"");
                                    result.push_str(&html_escape(&image_url));
                                    result.push_str("\" type=\"");
                                    result.push_str(&get_video_mime_type(&image_url));
                                    result.push_str("\">");
                                    result.push_str("Your browser does not support the video tag.");
                                    result.push_str("</video>");
                                } else {
                                    result.push_str("<img src=\"");
                                    result.push_str(&html_escape(&image_url));
                                    result.push_str("\" alt=\"");
                                    result.push_str(&html_escape(&alt_text));
                                    result.push_str("\">");
                                }
                                result.push_str("<div class=\"media-caption\">");
                                result.push_str(&html_escape(&alt_text));
                                result.push_str("</div>");
                                result.push_str("</div>");
                            } else {
                                if is_video {
                                    result.push_str("<video controls style=\"width: 100%;\">");
                                    result.push_str("<source src=\"");
                                    result.push_str(&html_escape(&image_url));
                                    result.push_str("\" type=\"");
                                    result.push_str(&get_video_mime_type(&image_url));
                                    result.push_str("\">");
                                    result.push_str("Your browser does not support the video tag.");
                                    result.push_str("</video>");
                                } else {
                                    result.push_str("<img src=\"");
                                    result.push_str(&html_escape(&image_url));
                                    result.push_str("\" alt=\"");
                                    result.push_str(&html_escape(&alt_text));
                                    result.push_str("\">");
                                }
                            }

                            i = paren_end_idx + 1;
                            continue;
                        }
                    }
                }
            }
        }

        // No pattern matched, add current character
        result.push(chars[i]);
        i += 1;
    }

    result
}

fn is_video_url(url: &str) -> bool {
    let video_extensions = ["mp4", "webm", "ogg", "mov", "avi", "mkv"];
    let lower_url = url.to_lowercase();
    video_extensions
        .iter()
        .any(|ext| lower_url.ends_with(&format!(".{}", ext)))
}

fn get_video_mime_type(url: &str) -> &'static str {
    let lower_url = url.to_lowercase();
    if lower_url.ends_with(".mp4") {
        "video/mp4"
    } else if lower_url.ends_with(".webm") {
        "video/webm"
    } else if lower_url.ends_with(".ogg") {
        "video/ogg"
    } else if lower_url.ends_with(".mov") {
        "video/quicktime"
    } else if lower_url.ends_with(".avi") {
        "video/x-msvideo"
    } else if lower_url.ends_with(".mkv") {
        "video/x-matroska"
    } else {
        "video/mp4" // fallback
    }
}

fn process_links(text: &str) -> String {
    process_links_with_config(text, &crate::config::Config::default())
}

fn safe_replace(
    text: &str,
    start_pattern: &str,
    end_pattern: &str,
    open_tag: &str,
    close_tag: &str,
) -> String {
    let mut result = String::with_capacity(text.len() + 1024);
    let mut remaining = text;

    while let Some(start_pos) = remaining.find(start_pattern) {
        result.push_str(&remaining[..start_pos]);

        let after_start = &remaining[start_pos + start_pattern.len()..];
        if let Some(end_pos) = after_start.find(end_pattern) {
            let content = &after_start[..end_pos];
            if !content.is_empty() && !content.contains('\n') {
                result.push_str(open_tag);
                result.push_str(content);
                result.push_str(close_tag);
                remaining = &after_start[end_pos + end_pattern.len()..];
            } else {
                result.push_str(start_pattern);
                remaining = &remaining[start_pos + start_pattern.len()..];
            }
        } else {
            result.push_str(start_pattern);
            remaining = &remaining[start_pos + start_pattern.len()..];
        }
    }

    result.push_str(remaining);
    result
}

fn sanitize_html(html: String) -> String {
    let mut builder = ammonia::Builder::default();
    builder
        .add_tags(&[
            "video",
            "source",
            "pre",
            "p",
            "table",
            "thead",
            "tbody",
            "tr",
            "th",
            "td",
            "em",
            "strong",
            "u",
            "del",
            "sup",
            "mark",
            "span",
            "code",
            "a",
            "img",
            "br",
            "hr",
            "h1",
            "h2",
            "h3",
            "h4",
            "blockquote",
            "div",
            "ol",
            "li",
        ])
        .add_tag_attributes("video", &["controls", "style"])
        .add_tag_attributes("source", &["src", "type"])
        .add_tag_attributes("img", &["src", "alt", "style"])
        .add_tag_attributes("code", &["class", "data-line-count"])
        .add_tag_attributes("span", &["class"])
        .add_tag_attributes("th", &["style"])
        .add_tag_attributes("td", &["style"])
        .add_tag_attributes("a", &["href", "target", "id", "class"])
        .add_tag_attributes("div", &["class"])
        .add_tag_attributes("hr", &["class"])
        .add_tag_attributes("li", &["id"])
        .add_tag_attributes("sup", &["id"])
        .add_tag_attributes("h1", &["id"])
        .add_tag_attributes("h2", &["id"])
        .add_tag_attributes("h3", &["id"])
        .add_tag_attributes("h4", &["id"])
        .link_rel(Some("noopener noreferrer"));

    builder.clean(&html).to_string()
}

pub fn sanitize_text(text: &str) -> String {
    let builder = ammonia::Builder::empty();
    builder.clean(text).to_string()
}

// Thanks for the code. You know who you are.
pub fn html_attr_escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
        .replace('\n', " ") // Replace newlines with space, not entity
        .replace('\r', "") // Remove carriage returns
}

fn sanitize_language(lang: &str) -> String {
    let sanitized = lang
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_' || *c == '+' || *c == '#')
        .collect::<String>()
        .to_lowercase();
    if sanitized.chars().count() > 15 {
        sanitized.chars().take(15).collect()
    } else {
        sanitized
    }
}

fn process_single_header(text: &str, header_count: &mut usize) -> Option<String> {
    let trimmed = text.trim();
    if trimmed.starts_with("#### ") {
        let header_text = &trimmed[5..];
        *header_count += 1;
        Some(format!(
            "<h4 id=\"h{}\">{}<a href=\"#h{}\" class=\"header-anchor\">#</a></h4>",
            *header_count, header_text, *header_count
        ))
    } else if trimmed.starts_with("### ") {
        let header_text = &trimmed[4..];
        *header_count += 1;
        Some(format!(
            "<h3 id=\"h{}\">{}<a href=\"#h{}\" class=\"header-anchor\">#</a></h3>",
            *header_count, header_text, *header_count
        ))
    } else if trimmed.starts_with("## ") {
        let header_text = &trimmed[3..];
        *header_count += 1;
        Some(format!(
            "<h2 id=\"h{}\">{}<a href=\"#h{}\" class=\"header-anchor\">#</a></h2>",
            *header_count, header_text, *header_count
        ))
    } else if trimmed.starts_with("# ") {
        let header_text = &trimmed[2..];
        *header_count += 1;
        Some(format!(
            "<h1 id=\"h{}\">{}<a href=\"#h{}\" class=\"header-anchor\">#</a></h1>",
            *header_count, header_text, *header_count
        ))
    } else {
        None
    }
}

fn process_single_blockquote(text: &str) -> String {
    let mut blockquote_content = String::new();

    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("> ") {
            let quote_text = &trimmed[2..];
            if !blockquote_content.is_empty() {
                blockquote_content.push_str("<br>");
            }
            blockquote_content.push_str(quote_text);
        }
    }

    format!("<blockquote>{}</blockquote>", blockquote_content)
}

fn extract_fenced_code_blocks(text: &str) -> (String, Vec<(String, String, u32)>) {
    let mut result = String::new();
    let mut fenced_blocks = Vec::new();

    // If there are no fenced code blocks, return original text
    if !text.contains("```") {
        return (text.to_string(), fenced_blocks);
    }

    let lines: Vec<&str> = text.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i];

        // Check if this line starts a fenced code block
        if line.starts_with("```") {
            // Count the fence length
            let fence_length = line.chars().take_while(|&c| c == '`').count();
            let language = sanitize_language(line[fence_length..].trim());
            let mut code_content = String::new();
            let mut found_end = false;

            // Look for the closing fence with same or greater length
            for j in (i + 1)..lines.len() {
                let closing_line = lines[j];
                if closing_line.starts_with("```") {
                    let closing_fence_length =
                        closing_line.chars().take_while(|&c| c == '`').count();
                    // Closing fence must be at least as long as opening fence
                    if closing_fence_length >= fence_length {
                        // Found closing fence
                        let placeholder = format!("{{{{FENCEDBLOCK{}}}}}", fenced_blocks.len());
                        // Count lines in code content (minimum 1, maximum 999)
                        let line_count = if code_content.is_empty() {
                            1
                        } else {
                            let count = code_content.lines().count() as u32;
                            count.clamp(1, 999)
                        };
                        fenced_blocks.push((language, code_content, line_count));
                        result.push_str(&placeholder);
                        if i < lines.len() - 1 || text.ends_with('\n') {
                            result.push('\n');
                        }
                        i = j + 1;
                        found_end = true;
                        break;
                    }
                }
                // Add line to code content (including lines with shorter fences)
                if !code_content.is_empty() {
                    code_content.push('\n');
                }
                code_content.push_str(lines[j]);
            }

            // If no closing fence found, treat as regular text
            if !found_end {
                result.push_str(line);
                if i < lines.len() - 1 {
                    result.push('\n');
                }
                i += 1;
            }
        } else {
            // Regular line
            result.push_str(line);
            if i < lines.len() - 1 {
                result.push('\n');
            }
            i += 1;
        }
    }

    (result, fenced_blocks)
}

fn map_language(lang: &str) -> &str {
    match lang.to_lowercase().as_str() {
        // Primary languages
        "javascript" | "js" => "javascript",
        "python" | "py" => "python",
        "java" => "java",
        "typescript" | "ts" => "typescript",
        "html" => "html",
        "css" => "css",
        "bash" | "sh" | "shell" => "bash",
        "sql" => "sql",
        "c" => "c",
        "cpp" | "c++" => "cpp",
        "csharp" | "c#" | "cs" => "csharp",
        "php" => "php",
        "ruby" | "rb" => "ruby",
        "go" | "golang" => "go",
        "rust" | "rs" => "rust",
        "swift" => "swift",
        "kotlin" | "kt" => "kotlin",
        "r" => "r",
        "matlab" => "matlab",
        "scala" => "scala",
        "perl" => "perl",
        "powershell" | "ps1" => "powershell",

        // Data formats
        "json" => "json",
        "xml" => "xml",
        "yaml" | "yml" => "yaml",
        "markdown" | "md" => "markdown",
        "toml" => "toml",
        "ini" => "ini",
        "properties" => "properties",

        // Web technologies
        "jsx" => "jsx",
        "tsx" => "tsx",
        "vue" => "vue",
        "scss" => "scss",
        "sass" => "sass",
        "less" => "less",
        "graphql" | "gql" => "graphql",
        "svelte" => "svelte",
        "handlebars" | "hbs" => "handlebars",
        "pug" | "jade" => "pug",
        "ejs" => "ejs",
        "nunjucks" | "njk" => "nunjucks",

        // Systems and config
        "dockerfile" | "docker" => "dockerfile",
        "makefile" | "make" => "makefile",
        "cmake" => "cmake",
        "nginx" => "nginx",
        "apache" => "apache",

        // Functional and other languages
        "lua" => "lua",
        "dart" => "dart",
        "elixir" | "ex" => "elixir",
        "haskell" | "hs" => "haskell",
        "clojure" | "clj" => "clojure",
        "objective-c" | "objc" => "objective-c",
        "coffeescript" | "coffee" => "coffeescript",
        "groovy" => "groovy",
        "racket" | "rkt" => "racket",
        "scheme" | "scm" => "scheme",
        "lisp" => "lisp",
        "erlang" | "erl" => "erlang",
        "fsharp" | "f#" | "fs" => "fsharp",
        "ocaml" | "ml" => "ocaml",
        "julia" | "jl" => "julia",
        "nim" => "nim",
        "crystal" | "cr" => "crystal",
        "d" => "d",
        "zig" => "zig",
        "vlang" => "v",
        "solidity" | "sol" => "solidity",

        // Hardware description
        "vhdl" => "vhdl",
        "verilog" => "verilog",
        "assembly" | "asm" => "assembly",

        // Legacy and specialized
        "fortran" | "f90" | "f95" => "fortran",
        "cobol" | "cob" => "cobol",
        "pascal" | "pas" => "pascal",
        "ada" => "ada",
        "prolog" | "pl" => "prolog",
        "smalltalk" | "st" => "smalltalk",
        "tcl" => "tcl",
        "awk" => "awk",
        "sed" => "sed",

        // Editors
        "vim" | "vimscript" => "vim",
        "emacs-lisp" | "elisp" => "emacs-lisp",

        // Alternative languages
        "elm" => "elm",
        "purescript" | "purs" => "purescript",
        "reasonml" | "reason" | "re" => "reasonml",
        "apex" => "apex",
        "arduino" | "ino" => "arduino",
        "processing" | "pde" => "processing",
        "openscad" | "scad" => "openscad",

        // Document formats
        "latex" | "tex" => "latex",
        "bibtex" | "bib" => "bibtex",
        "rmarkdown" | "rmd" => "rmarkdown",
        "restructuredtext" | "rst" => "restructuredtext",
        "asciidoc" | "adoc" => "asciidoc",
        "textile" => "textile",
        "org" => "org",

        // Version control and patches
        "diff" => "diff",
        "patch" => "patch",

        // Generic
        "plaintext" | "text" | "txt" => "plaintext",

        // Return original if no mapping found
        _ => lang,
    }
}

fn restore_fenced_code_blocks(text: &str, fenced_blocks: &[(String, String, u32)]) -> String {
    let mut result = text.to_string();

    for (index, (language, code_content, line_count)) in fenced_blocks.iter().enumerate() {
        let placeholder = format!("{{{{FENCEDBLOCK{}}}}}", index);
        let mapped_lang = map_language(language);
        let class_attr = if mapped_lang.is_empty() {
            String::new()
        } else {
            format!(" class=\"language-{}\"", mapped_lang)
        };
        // HTML escape the code content to prevent sanitizer issues
        let escaped_content = html_escape(code_content);
        let replacement = format!(
            "<pre><code{} data-line-count=\"{}\">{}</code></pre>",
            class_attr, line_count, escaped_content
        );

        result = result.replace(&placeholder, &replacement);
    }

    result
}

fn format_paragraphs_with_headers(text: &str) -> String {
    let mut result = String::with_capacity(text.len() + (text.len() / 10));
    let mut header_count = 0;

    // Preprocess text to ensure headers are properly separated
    let preprocessed = preprocess_headers_for_paragraphs(text);
    let parts: Vec<&str> = preprocessed.split("\n\n").collect();

    for (i, part) in parts.iter().enumerate() {
        let trimmed = part.trim();

        if trimmed.is_empty() {
            continue;
        }

        // Check for headers first
        if let Some(header) = process_single_header(trimmed, &mut header_count) {
            result.push_str(&header);
        }
        // Check for blockquotes
        else if trimmed.lines().any(|line| line.trim().starts_with("> ")) {
            result.push_str(&process_single_blockquote(trimmed));
        }
        // Check for mixed block content
        else if (trimmed.contains("{{FENCEDBLOCK")
            || trimmed.contains("{{CODEBLOCK")
            || trimmed.contains("<table>"))
            && !trimmed.starts_with("{{FENCEDBLOCK")
            && !trimmed.starts_with("{{CODEBLOCK")
            && !trimmed.starts_with("<table>")
        {
            let lines: Vec<&str> = part.lines().collect();
            let mut current_paragraph = String::new();

            for line in lines {
                let line_trimmed = line.trim();

                if line_trimmed.starts_with("{{FENCEDBLOCK")
                    || line_trimmed.starts_with("{{CODEBLOCK")
                    || line_trimmed.starts_with("<table>")
                {
                    if !current_paragraph.is_empty() {
                        result.push_str(&format!("<p>{}</p>\n", current_paragraph.trim()));
                        current_paragraph.clear();
                    }
                    result.push_str(line_trimmed);
                    result.push('\n');
                } else if !line_trimmed.is_empty() {
                    if !current_paragraph.is_empty() {
                        current_paragraph.push_str("<br>");
                    }
                    current_paragraph.push_str(line_trimmed);
                }
            }

            if !current_paragraph.is_empty() {
                result.push_str(&format!("<p>{}</p>", current_paragraph.trim()));
            }
        } else if trimmed.starts_with("{{FENCEDBLOCK")
            || trimmed.starts_with("{{CODEBLOCK")
            || trimmed.starts_with("<img ")
            || trimmed.starts_with("<video ")
            || trimmed.starts_with("<table>")
        {
            result.push_str(trimmed);
        } else {
            let lines: Vec<&str> = part.lines().collect();
            let mut paragraph_content = String::new();

            for (j, line) in lines.iter().enumerate() {
                let trimmed_line = line.trim();
                if !trimmed_line.is_empty() {
                    paragraph_content.push_str(trimmed_line);
                    if j < lines.len() - 1
                        && lines
                            .get(j + 1)
                            .map_or(false, |next| !next.trim().is_empty())
                    {
                        paragraph_content.push_str("<br>");
                    }
                }
            }

            if !paragraph_content.is_empty() {
                result.push_str(&format!("<p>{}</p>", paragraph_content));
            }
        }

        if i < parts.len() - 1 && !result.is_empty() && !result.ends_with('\n') {
            result.push('\n');
        }
    }

    // Clean up empty paragraphs and excessive spacing
    result = result.replace("<p></p>", "");
    result = result.replace("\n\n\n", "\n\n");

    // Remove excessive br tags before tables - more aggressive cleanup
    let mut iterations = 0;
    while result.contains("<br><table>") && iterations < 50 {
        result = result.replace("<br><br>", "<br>");
        result = result.replace("<br><table>", "<table>");
        result = result.replace("<br>\n<table>", "\n<table>");
        iterations += 1;
    }

    result
}

fn process_dividers(content: &str) -> String {
    let mut result = String::new();

    for line in content.lines() {
        let trimmed = line.trim();

        if trimmed == "***" && line.chars().all(|c| c == '*' || c.is_whitespace()) {
            // Three stars divider - centered asterisks
            result.push_str("<div class=\"divider-stars\"><div class=\"asterisk\"><div class=\"center\"></div></div><div class=\"asterisk\"><div class=\"center\"></div></div><div class=\"asterisk\"><div class=\"center\"></div></div></div>");
        } else if trimmed == "-*-"
            && line
                .chars()
                .all(|c| c == '-' || c == '*' || c.is_whitespace())
        {
            // Single asterisk divider - centered single asterisk
            result.push_str("<div class=\"divider-asterisk\"><div class=\"center\"></div></div>");
        } else if trimmed == "---" && line.chars().all(|c| c == '-' || c.is_whitespace()) {
            // Horizontal thin divider
            result.push_str("<hr class=\"divider-thin\">");
        } else if trimmed == "===" && line.chars().all(|c| c == '=' || c.is_whitespace()) {
            // Horizontal double-line divider
            result.push_str("<hr class=\"divider-double\">");
        } else {
            result.push_str(line);
        }
        result.push('\n');
    }

    result
}

fn preprocess_headers_for_paragraphs(text: &str) -> String {
    let mut result = String::new();
    let lines: Vec<&str> = text.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i];
        let trimmed = line.trim();

        // Check if this line is a header
        if trimmed.starts_with("# ")
            || trimmed.starts_with("## ")
            || trimmed.starts_with("### ")
            || trimmed.starts_with("#### ")
        {
            // Add the header line
            result.push_str(line);
            result.push('\n');

            // Always add an extra newline after headers to ensure proper separation
            // This forces headers to be in their own paragraph blocks
            result.push('\n');
        } else {
            result.push_str(line);
            result.push('\n');
        }

        i += 1;
    }

    result
}

fn process_tables(text: &str) -> String {
    let mut result = String::new();
    let lines: Vec<&str> = text.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i].trim();

        // Check if this line looks like a table header (contains |)
        if line.contains('|') && line.len() > 0 {
            // Look for separator line
            if i + 1 < lines.len() {
                let separator = lines[i + 1].trim();
                if is_table_separator(separator) {
                    // Found a table - parse it
                    let (table_html, lines_consumed) = parse_table(&lines[i..]);
                    result.push_str(&table_html);
                    i += lines_consumed;
                    continue;
                }
            }
        }

        // Not a table line, add as is
        result.push_str(lines[i]);
        if i < lines.len() - 1 {
            result.push('\n');
        }
        i += 1;
    }

    result
}

fn is_table_separator(line: &str) -> bool {
    let line = line.trim();
    if line.is_empty() || !line.contains('|') {
        return false;
    }

    // Check if line contains only |, -, :, and spaces
    line.chars().all(|c| matches!(c, '|' | '-' | ':' | ' '))
}

fn parse_table(lines: &[&str]) -> (String, usize) {
    if lines.len() < 2 {
        return (String::new(), 0);
    }

    let header_line = lines[0].trim();
    let separator_line = lines[1].trim();

    // Parse header
    let headers = parse_table_row(header_line);
    let alignments = parse_table_alignments(separator_line);

    let mut table_html = String::from("<table>\n<thead>\n<tr>");
    for (i, header) in headers.iter().enumerate() {
        let style = match alignments.get(i).unwrap_or(&TableAlignment::Left) {
            TableAlignment::Left => "",
            TableAlignment::Center => " style=\"text-align: center\"",
            TableAlignment::Right => " style=\"text-align: right\"",
        };
        table_html.push_str(&format!("<th{}>{}</th>", style, header.trim()));
    }
    table_html.push_str("</tr>\n</thead>\n<tbody>\n");

    // Parse body rows
    let mut rows_processed = 2; // header + separator
    for line_idx in 2..lines.len() {
        let line = lines[line_idx].trim();
        if line.is_empty() || !line.contains('|') {
            break;
        }

        let cells = parse_table_row(line);
        table_html.push_str("<tr>");
        for (i, cell) in cells.iter().enumerate() {
            let style = match alignments.get(i).unwrap_or(&TableAlignment::Left) {
                TableAlignment::Left => "",
                TableAlignment::Center => " style=\"text-align: center\"",
                TableAlignment::Right => " style=\"text-align: right\"",
            };
            table_html.push_str(&format!("<td{}>{}</td>", style, cell.trim()));
        }
        table_html.push_str("</tr>\n");
        rows_processed += 1;
    }

    table_html.push_str("</tbody>\n</table>\n");
    (table_html, rows_processed)
}

fn parse_table_row(line: &str) -> Vec<String> {
    let line = line.trim();
    let line = if line.starts_with('|') {
        &line[1..]
    } else {
        line
    };
    let line = if line.ends_with('|') {
        &line[..line.len() - 1]
    } else {
        line
    };

    line.split('|').map(|s| s.trim().to_string()).collect()
}

#[derive(Debug, Clone)]
enum TableAlignment {
    Left,
    Center,
    Right,
}

fn parse_table_alignments(separator: &str) -> Vec<TableAlignment> {
    let cells = parse_table_row(separator);
    cells
        .iter()
        .map(|cell| {
            let cell = cell.trim();
            if cell.starts_with(':') && cell.ends_with(':') {
                TableAlignment::Center
            } else if cell.ends_with(':') {
                TableAlignment::Right
            } else {
                TableAlignment::Left
            }
        })
        .collect()
}

pub fn html_escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}

fn extract_code_blocks(text: &str) -> (String, Vec<String>) {
    let mut result = String::new();
    let mut code_blocks = Vec::new();
    let chars: Vec<char> = text.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        if chars[i] == '`' {
            // Look for closing backtick
            let start = i + 1;
            let mut end = None;

            for j in start..chars.len() {
                if chars[j] == '`' {
                    end = Some(j);
                    break;
                }
            }

            if let Some(end_pos) = end {
                let code_content: String = chars[start..end_pos].iter().collect();

                if !code_content.is_empty() && !code_content.contains('\n') {
                    let placeholder = format!("{{{{CODEBLOCK{}}}}}", code_blocks.len());
                    code_blocks.push(code_content);
                    result.push_str(&placeholder);
                    i = end_pos + 1;
                    continue;
                }
            }
        }

        result.push(chars[i]);
        i += 1;
    }

    (result, code_blocks)
}

fn restore_code_blocks(text: &str, code_blocks: &[String]) -> String {
    let mut result = text.to_string();

    for (index, code_content) in code_blocks.iter().enumerate() {
        let placeholder = format!("{{{{CODEBLOCK{}}}}}", index);
        let escaped_content = html_escape(code_content);
        let replacement = format!("<code>{}</code>", escaped_content);
        result = result.replace(&placeholder, &replacement);
    }

    result
}

fn process_comments(content: &str) -> String {
    let lines: Vec<&str> = content.lines().collect();
    let mut result = Vec::new();

    for line in lines {
        // Check if line starts with "// " (comment syntax)
        if line.trim_start().starts_with("// ") {
            // Skip comment lines - they won't appear in HTML output
            continue;
        } else {
            result.push(line);
        }
    }

    result.join("\n")
}

fn process_footnotes(content: &str) -> String {
    let mut result = String::new();
    let mut footnote_definitions = std::collections::HashMap::new();
    let mut footnote_counter = 0u32;
    let mut inline_footnote_counter = 0u32;

    // First pass: extract footnote definitions [^id]: text
    let lines: Vec<&str> = content.lines().collect();
    let mut content_lines = Vec::new();

    for line in &lines {
        let trimmed = line.trim();
        if trimmed.starts_with("[^") && trimmed.contains("]:") {
            if let Some(colon_pos) = trimmed.find("]:") {
                let id_part = &trimmed[2..colon_pos];
                let definition = trimmed[colon_pos + 2..].trim();
                footnote_definitions.insert(id_part.to_string(), definition.to_string());
            }
        } else {
            content_lines.push(*line);
        }
    }

    let content_text = content_lines.join("\n");
    let chars: Vec<char> = content_text.chars().collect();
    let mut i = 0;
    let mut footnote_references = Vec::new();
    let mut inline_footnotes = Vec::new();

    // Second pass: process footnote references and inline footnotes
    while i < chars.len() {
        if chars.len() >= 3 && i < chars.len() - 2 && chars[i] == '^' && chars[i + 1] == '[' {
            // Inline footnote: ^[text]
            let mut bracket_end = None;
            let mut j = i + 2;
            let mut bracket_depth = 1;

            while j < chars.len() && bracket_depth > 0 {
                if chars[j] == '[' {
                    bracket_depth += 1;
                } else if chars[j] == ']' {
                    bracket_depth -= 1;
                    if bracket_depth == 0 {
                        bracket_end = Some(j);
                        break;
                    }
                }
                j += 1;
            }

            if let Some(end_pos) = bracket_end {
                inline_footnote_counter += 1;
                let footnote_text: String = chars[(i + 2)..end_pos].iter().collect();
                let footnote_id = format!("ifn{}", inline_footnote_counter);

                inline_footnotes.push((footnote_id.clone(), footnote_text));

                // Use placeholder to avoid processing by other markdown processors
                result.push_str(&format!("XFOOTNOTEINLINEX{}XENDX", inline_footnote_counter));

                i = end_pos + 1;
                continue;
            }
        } else if chars.len() >= 4 && i < chars.len() - 3 && chars[i] == '[' && chars[i + 1] == '^'
        {
            // Reference footnote: [^id]
            let mut bracket_end = None;
            let mut j = i + 2;

            while j < chars.len() && chars[j] != '\n' {
                if chars[j] == ']' {
                    bracket_end = Some(j);
                    break;
                }
                j += 1;
            }

            if let Some(end_pos) = bracket_end {
                let footnote_id: String = chars[(i + 2)..end_pos].iter().collect();

                if footnote_definitions.contains_key(&footnote_id) {
                    footnote_counter += 1;
                    footnote_references.push((footnote_id.clone(), footnote_counter));

                    // Use placeholder to avoid processing by other markdown processors
                    result.push_str(&format!("XFOOTNOTEREFX{}XENDX", footnote_counter));

                    i = end_pos + 1;
                    continue;
                }
            }
        }

        result.push(chars[i]);
        i += 1;
    }

    // Replace placeholders with actual HTML
    for i in 1..=footnote_counter {
        let placeholder = format!("XFOOTNOTEREFX{}XENDX", i);
        let replacement = format!(
            "<sup><a href=\"XHASHXFN{}\" id=\"fnref{}\">{}</a></sup>",
            i, i, i
        );
        result = result.replace(&placeholder, &replacement);
    }

    for i in 1..=inline_footnote_counter {
        let placeholder = format!("XFOOTNOTEINLINEX{}XENDX", i);
        let replacement = format!(
            "<sup><a href=\"XHASHXifn{}\" id=\"ifn{}ref\">{}</a></sup>",
            i, i, i
        );
        result = result.replace(&placeholder, &replacement);
    }

    // Add footnotes section at the end if there are any footnotes
    if !footnote_references.is_empty() || !inline_footnotes.is_empty() {
        result.push_str("\n\nXFOOTNOTESECTIONSTARTX");

        // Add reference footnotes
        for (footnote_id, number) in footnote_references {
            if let Some(definition) = footnote_definitions.get(&footnote_id) {
                result.push_str(&format!(
                    "<li id=\"fn{}\">{} <a href=\"XHASHXfnref{}\" class=\"footnote-backref\">↩</a></li>",
                    number, definition, number
                ));
            }
        }

        // Add inline footnotes
        for (footnote_id, footnote_text) in inline_footnotes.iter() {
            result.push_str(&format!(
                "<li id=\"{}\">{} <a href=\"XHASHX{}ref\" class=\"footnote-backref\">↩</a></li>",
                footnote_id, footnote_text, footnote_id
            ));
        }

        result.push_str("XFOOTNOTESECTIONENDX");
    }

    result
}

fn restore_footnotes(text: &str) -> String {
    let mut result = text.to_string();

    // Replace footnote section placeholders
    result = result.replace(
        "XFOOTNOTESECTIONSTARTX",
        "<div class=\"footnotes\">\n<ol>\n",
    );
    result = result.replace("XFOOTNOTESECTIONENDX", "</ol>\n</div>");

    // Replace hash placeholders
    result = result.replace("XHASHX", "#");

    result
}

fn process_links_with_config(text: &str, config: &crate::config::Config) -> String {
    let mut result = String::with_capacity(text.len() + 1024);
    let chars: Vec<char> = text.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        if chars[i] == '[' {
            // Find closing bracket
            let mut bracket_end = None;
            let mut j = i + 1;
            while j < chars.len() && chars[j] != '\n' {
                if chars[j] == ']' {
                    bracket_end = Some(j);
                    break;
                }
                j += 1;
            }

            if let Some(bracket_end_idx) = bracket_end {
                // Check for [text](url) pattern
                if bracket_end_idx + 1 < chars.len() && chars[bracket_end_idx + 1] == '(' {
                    let mut paren_end = None;
                    let mut k = bracket_end_idx + 2;
                    while k < chars.len() && chars[k] != '\n' {
                        if chars[k] == ')' {
                            paren_end = Some(k);
                            break;
                        }
                        k += 1;
                    }

                    if let Some(paren_end_idx) = paren_end {
                        let link_text: String = chars[(i + 1)..bracket_end_idx].iter().collect();
                        let link_url: String =
                            chars[(bracket_end_idx + 2)..paren_end_idx].iter().collect();

                        if !link_text.is_empty()
                            && !link_url.is_empty()
                            && link_url.len() <= config.security.max_url_length
                        {
                            result.push_str("<a href=\"");
                            result.push_str(&link_url);
                            if config.security.external_link_security {
                                result.push_str("\" target=\"_blank\">");
                            } else {
                                result.push_str("\">");
                            }
                            result.push_str(&link_text);
                            result.push_str("</a>");
                            i = paren_end_idx + 1;
                            continue;
                        }
                    }
                }

                // Check for [url] pattern (bare URL in brackets)
                let link_url: String = chars[(i + 1)..bracket_end_idx].iter().collect();
                if link_url.len() <= config.security.max_url_length && link_url.starts_with("http")
                {
                    result.push_str("<a href=\"");
                    result.push_str(&link_url);
                    if config.security.external_link_security {
                        result.push_str("\" target=\"_blank\">");
                    } else {
                        result.push_str("\">");
                    }
                    result.push_str(&link_url);
                    result.push_str("</a>");
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_formatting() {
        assert_eq!(
            render_markdown("*italic* text").contains("<em>italic</em>"),
            true
        );
        assert_eq!(
            render_markdown("**bold** text").contains("<strong>bold</strong>"),
            true
        );
        assert_eq!(
            render_markdown("_underline_ text").contains("<u>underline</u>"),
            true
        );
        assert_eq!(
            render_markdown("==highlight== text").contains("<mark>highlight</mark>"),
            true
        );
    }

    #[test]
    fn test_highlighting_with_mixed_formatting() {
        // Test highlighting mixed with other formatting
        let mixed = "This has ==highlighted== text with **bold** and *italic* formatting.";
        let result = render_markdown(mixed);
        assert!(result.contains("<mark>highlighted</mark>"));
        assert!(result.contains("<strong>bold</strong>"));
        assert!(result.contains("<em>italic</em>"));

        // Test nested highlighting scenarios
        let complex = "==This is ==nested== highlighting== and normal text.";
        let complex_result = render_markdown(complex);
        assert!(complex_result.contains("<mark>"));
    }

    #[test]
    fn test_unicode_handling() {
        let japanese = "渋い美しさ *bold* text";
        let result = render_markdown(japanese);
        assert!(result.contains("渋い美しさ"));
        assert!(result.contains("<em>bold</em>"));
    }

    #[test]
    fn test_links() {
        let text = "[Google](https://google.com)";
        let result = render_markdown(text);

        assert!(result.contains("<a href=\"https://google.com\""));
        assert!(result.contains("target=\"_blank\""));
        assert!(result.contains("rel=\"noopener noreferrer\""));
        assert!(result.contains(">Google</a>"));
    }

    #[test]
    fn test_simple_links() {
        let text = "[https://example.com]";
        let result = render_markdown(text);

        assert!(result.contains("<a href=\"https://example.com\""));
        assert!(result.contains("target=\"_blank\""));
        assert!(result.contains("rel=\"noopener noreferrer\""));
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
    fn test_code_literal_rendering() {
        // Test that markdown inside code blocks is not processed
        let text = "`(link text)[https://example.com]`";
        let result = render_markdown(text);

        assert!(result.contains("<code>(link text)[https://example.com]</code>"));
        assert!(!result.contains("<a href"));

        // Test with various markdown elements inside code
        let text2 = "`*bold* **italic** _underline_ ~strike~`";
        let result2 = render_markdown(text2);

        assert!(result2.contains("<code>*bold* **italic** _underline_ ~strike~</code>"));
        assert!(!result2.contains("<strong>"));
        assert!(!result2.contains("<em>"));
        assert!(!result2.contains("<u>"));
        assert!(!result2.contains("<del>"));

        // Test code block mixed with regular markdown
        let text3 = "This is *bold* and `this is code with *asterisks*` and more *bold*.";
        let result3 = render_markdown(text3);
        assert!(result3.contains("<em>bold</em>"));
        assert!(result3.contains("<code>this is code with *asterisks*</code>"));
        // The asterisks inside code should NOT become <strong> tags
        assert!(!result3.contains("<code>this is code with <em>asterisks</em></code>"));

        // Test adjacent backticks with content
        let text4 = "Test `first` and `second` code blocks";
        let result4 = render_markdown(text4);
        assert!(result4.contains("<code>first</code>"));
        assert!(result4.contains("<code>second</code>"));
    }

    #[test]
    fn test_fenced_code_blocks() {
        // Test basic fenced code block
        let text = "```json\n{\"key\": \"value\"}\n```";
        let result = render_markdown(text);
        assert!(
            result.contains("<pre><code class=\"language-json\" data-line-count=\"1\">{\"key\": \"value\"}</code></pre>")
        );

        // Test Python code block
        let text_py = "```py\nprint('hello world')\n```";
        let result_py = render_markdown(text_py);
        assert!(result_py
            .contains("<pre><code class=\"language-python\" data-line-count=\"1\">print('hello world')</code></pre>"));

        // Test JavaScript code block
        let text_js = "```js\nconsole.log('hello');\n```";
        let result_js = render_markdown(text_js);
        assert!(result_js.contains(
            "<pre><code class=\"language-javascript\" data-line-count=\"1\">console.log('hello');</code></pre>"
        ));

        // Test code block without language
        let text_no_lang = "```\nsome code\n```";
        let result_no_lang = render_markdown(text_no_lang);
        assert!(result_no_lang.contains("<pre><code data-line-count=\"1\">some code</code></pre>"));
        assert!(!result_no_lang.contains("class=\"language-"));

        // Test multiline code block
        let text_multi = "```rust\nfn main() {\n    println!(\"Hello, world!\");\n}\n```";
        let result_multi = render_markdown(text_multi);

        assert!(result_multi.contains("<pre><code class=\"language-rust\" data-line-count=\"3\">fn main() {\n    println!(\"Hello, world!\");\n}</code></pre>"));
    }

    #[test]
    fn test_fenced_code_blocks_no_markdown_processing() {
        // Test that markdown inside fenced code blocks is not processed
        let text =
            "```js\nconst message = \"*bold* **italic** _underline_\";\nconsole.log(message);\n```";
        let result = render_markdown(text);

        // Should contain raw markdown characters, not HTML tags
        assert!(result.contains("*bold* **italic** _underline_"));
        assert!(!result.contains("<strong>"));
        assert!(!result.contains("<em>"));
        assert!(!result.contains("<u>"));

        // Test with various markdown elements
        let complex = "```python\n# This is *not* **processed**\ndef test():\n    print(\"[not a link](http://example.com)\")\n    return ~strikethrough~ and ^superscript^\n```";
        let complex_result = render_markdown(complex);

        assert!(complex_result.contains("*not* **processed**"));
        assert!(complex_result.contains("[not a link](http://example.com)"));
        assert!(complex_result.contains("~strikethrough~"));
        assert!(complex_result.contains("^superscript^"));
        assert!(!complex_result.contains("<a href"));
        assert!(!complex_result.contains("<del>"));
        assert!(!complex_result.contains("<sup>"));
    }

    #[test]
    fn test_sanitize_text() {
        let malicious_text = "<script>alert('xss')</script>Hello World";
        let sanitized = sanitize_text(&malicious_text);
        assert_eq!(sanitized, "Hello World");

        let various_tags = "<b>Bold</b><i>Italic</i><script>alert('xss')</script>Clean Text";
        let sanitized_tags = sanitize_text(&various_tags);
        assert_eq!(sanitized_tags, "BoldItalicClean Text");

        let clean_text = "Just normal text";
        let sanitized_clean = sanitize_text(&clean_text);
        assert_eq!(sanitized_clean, "Just normal text");

        // Test long text (no truncation)
        let long_text = "This is a very long text that should be truncated";
        let sanitized = sanitize_text(&long_text);
        assert_eq!(
            sanitized,
            "This is a very long text that should be truncated"
        );
    }

    #[test]
    fn test_sanitize_language() {
        assert_eq!(sanitize_language("javascript"), "javascript");
        assert_eq!(sanitize_language("c++"), "c++");
        assert_eq!(sanitize_language("c#"), "c#");
        assert_eq!(sanitize_language("f#"), "f#");
        assert_eq!(sanitize_language("objective-c"), "objective-c");
        assert_eq!(sanitize_language("emacs-lisp"), "emacs-lisp");

        assert_eq!(
            sanitize_language("<script>alert('xss')</script>"),
            "scriptalertxsss"
        );
        assert_eq!(sanitize_language("python; rm -rf /"), "pythonrm-rf");
        assert_eq!(
            sanitize_language("js</style><script>evil()</script>"),
            "jsstylescriptev"
        );
        assert_eq!(
            sanitize_language("bash && curl evil.com"),
            "bashcurlevilcom"
        );
        assert_eq!(sanitize_language("JAVASCRIPT"), "javascript");

        assert_eq!(sanitize_language(""), "");
        assert_eq!(sanitize_language("   python   "), "python");
        assert_eq!(sanitize_language("python3.9"), "python39");

        // Test truncation
        assert_eq!(
            sanitize_language("verylonglanguagename12345"),
            "verylonglanguag"
        );
        assert_eq!(
            sanitize_language("superlonglanguagename"),
            "superlonglangua"
        );
    }

    #[test]
    fn test_malicious_language_code_blocks() {
        let malicious_code = r#"```<script>alert('xss')</script>
console.log('test');
```"#;
        let result = render_markdown(malicious_code);
        assert!(
            result.contains("<pre><code class=\"language-scriptalertxsss\" data-line-count=\"1\">")
        );
        assert!(result.contains("console.log('test');"));
        assert!(!result.contains("<script>"));

        let injection_code = r#"```python; rm -rf /
print("hello")
```"#;
        let result2 = render_markdown(injection_code);
        assert!(
            result2.contains("<pre><code class=\"language-pythonrm-rf\" data-line-count=\"1\">")
        );
        assert!(result2.contains("print(\"hello\")"));
        assert!(!result2.contains("rm -rf"));

        let html_injection = r#"```js</style><script>evil()</script>
var x = 1;
```"#;
        let result3 = render_markdown(html_injection);
        assert!(result3
            .contains("<pre><code class=\"language-jsstylescriptev\" data-line-count=\"1\">"));
        assert!(result3.contains("var x = 1;"));
        assert!(!result3.contains("</style>"));
        assert!(!result3.contains("<script>evil()"));
    }

    #[test]
    fn test_legitimate_special_character_languages() {
        let valid_langs = [
            ("c++", "cpp"),
            ("c#", "csharp"),
            ("f#", "fsharp"),
            ("objective-c", "objective-c"),
            ("emacs-lisp", "emacs-lisp"),
        ];

        for (input, mapped_output) in valid_langs {
            let sanitized = sanitize_language(input);
            assert!(!sanitized.contains("<"));
            assert!(!sanitized.contains(">"));

            let code = format!("```{}\ntest code\n```", input);
            let result = render_markdown(&code);
            assert!(result.contains(&format!("class=\"language-{}\"", mapped_output)));
        }
    }

    #[test]
    fn test_comprehensive_language_mapping() {
        // Test popular language mappings
        let mappings = [
            ("py", "python"),
            ("js", "javascript"),
            ("ts", "typescript"),
            ("rs", "rust"),
            ("sh", "bash"),
            ("cpp", "cpp"),
            ("c++", "cpp"),
            ("c#", "csharp"),
            ("cs", "csharp"),
            ("rb", "ruby"),
            ("kt", "kotlin"),
            ("go", "go"),
            ("golang", "go"),
            ("yml", "yaml"),
            ("md", "markdown"),
            ("jsx", "jsx"),
            ("tsx", "tsx"),
            ("vue", "vue"),
            ("scss", "scss"),
            ("sass", "sass"),
            ("dockerfile", "dockerfile"),
            ("docker", "dockerfile"),
            ("makefile", "makefile"),
            ("make", "makefile"),
            ("hs", "haskell"),
            ("clj", "clojure"),
            ("ex", "elixir"),
            ("erl", "erlang"),
            ("ml", "ocaml"),
            ("jl", "julia"),
            ("cr", "crystal"),
            ("sol", "solidity"),
            ("asm", "assembly"),
            ("f90", "fortran"),
            ("cob", "cobol"),
            ("pas", "pascal"),
            ("st", "smalltalk"),
            ("elisp", "emacs-lisp"),
            ("purs", "purescript"),
            ("re", "reasonml"),
            ("ino", "arduino"),
            ("pde", "processing"),
            ("scad", "openscad"),
            ("tex", "latex"),
            ("bib", "bibtex"),
            ("rst", "restructuredtext"),
            ("adoc", "asciidoc"),
            ("txt", "plaintext"),
            ("text", "plaintext"),
            ("vlang", "v"),
        ];

        for (alias, expected) in mappings {
            let text = format!("```{}\ncode here\n```", alias);
            let result = render_markdown(&text);
            assert!(
                result.contains(&format!("class=\"language-{}\"", expected)),
                "Failed mapping: {} should map to {}",
                alias,
                expected
            );
        }
    }

    #[test]
    fn test_code_block_line_count() {
        // Test single line code block
        let single_line = "```rust\nlet x = 5;\n```";
        let result_single = render_markdown(single_line);

        assert!(result_single.contains("data-line-count=\"1\""));
        assert!(result_single.contains(
            "<pre><code class=\"language-rust\" data-line-count=\"1\">let x = 5;</code></pre>"
        ));

        // Test multi-line code block
        let multi_line = "```python\ndef hello():\n    print(\"world\")\n    return True\n```";
        let result_multi = render_markdown(multi_line);
        assert!(result_multi.contains("data-line-count=\"3\""));
        assert!(result_multi.contains("<pre><code class=\"language-python\" data-line-count=\"3\">def hello():\n    print(\"world\")\n    return True</code></pre>"));

        // Test empty code block (should have line count of 1)
        let empty_block = "```\n```";
        let result_empty = render_markdown(empty_block);
        assert!(result_empty.contains("data-line-count=\"1\""));

        // Test code block without language
        let no_lang = "```\nsome code\nmore code\n```";
        let result_no_lang = render_markdown(no_lang);
        assert!(result_no_lang.contains("data-line-count=\"2\""));
        assert!(result_no_lang
            .contains("<pre><code data-line-count=\"2\">some code\nmore code</code></pre>"));

        // Test very large code block (should be clamped to 999)
        let large_code = format!("```rust\n{}\n```", "println!(\"line\");\n".repeat(1001));
        let result_large = render_markdown(&large_code);
        assert!(result_large.contains("data-line-count=\"999\""));
    }

    #[test]
    fn test_mixed_code_blocks() {
        // Test mixing fenced and inline code blocks
        let text = "Here's some `inline code` and a fenced block:\n```json\n{\"test\": true}\n```\nMore text with `more inline`.";
        let result = render_markdown(text);

        assert!(result.contains("<code>inline code</code>"));
        assert!(result.contains("<code>more inline</code>"));
        assert!(result.contains("<pre><code class=\"language-json\" data-line-count=\"1\">{\"test\": true}</code></pre>"));
    }

    #[test]
    fn test_fenced_vs_regular_markdown_processing() {
        // Test that shows the clear difference between processed and unprocessed markdown
        let mixed_content = r#"Regular text with **bold** and *italic* formatting.

```js
// This code has *bold* and **italic** but should NOT be processed
const message = "*not bold* and **not italic**";
console.log("[not a link](http://example.com)");
```

More regular text with _underline_ and ~strikethrough~."#;

        let result = render_markdown(mixed_content);

        // Regular text should be processed
        assert!(result.contains("<strong>bold</strong>"));
        assert!(result.contains("<em>italic</em>"));
        assert!(result.contains("<u>underline</u>"));
        assert!(result.contains("<del>strikethrough</del>"));

        // Code block content should be raw/unprocessed
        assert!(result.contains("*not bold* and **not italic**"));
        assert!(result.contains("[not a link](http://example.com)"));

        // Verify the code block doesn't contain processed HTML
        let code_block_part = result
            .split("<pre><code")
            .nth(1)
            .unwrap()
            .split("</code></pre>")
            .next()
            .unwrap();
        assert!(!code_block_part.contains("<strong>"));
        assert!(!code_block_part.contains("<em>"));
        assert!(!code_block_part.contains("<a href"));
    }

    #[test]
    fn test_code_block_line_breaks() {
        let text = "Here's some text:\n```json\n{\"test\": true}\n```\nMore text after.";
        let result = render_markdown(text);

        assert!(result.contains("<p>Here's some text:</p>"));
        assert!(result.contains("<pre><code class=\"language-json\" data-line-count=\"1\">{\"test\": true}</code></pre>"));
        assert!(result.contains("<p>More text after.</p>"));
        assert!(!result.contains("<br>\n<pre>"));
        assert!(!result.contains("<br><pre>"));
        assert!(!result.contains("</pre><br>\n"));
        assert!(!result.contains("</pre><br>"));

        // Test with multiple code blocks
        let multiple = "First block:\n```js\nconsole.log('test');\n```\nMiddle text.\n```py\nprint('hello')\n```\nEnd text.";
        let multiple_result = render_markdown(multiple);

        // Should not have unwanted <br> tags around any code block
        assert!(!multiple_result.contains("<br>\n<pre>"));
        assert!(!multiple_result.contains("<br><pre>"));
        assert!(!multiple_result.contains("</pre><br>\n"));
        assert!(!multiple_result.contains("</pre><br>"));

        // Test that regular paragraph breaks still work properly
        let with_paragraphs = "First paragraph.\n\nSecond paragraph.\n\n```js\nconsole.log('test');\n```\n\nThird paragraph.";
        let paragraph_result = render_markdown(with_paragraphs);

        // Should have proper paragraph structure
        assert!(paragraph_result.contains("<p>First paragraph.</p>"));
        assert!(paragraph_result.contains("<p>Second paragraph.</p>"));
        assert!(paragraph_result.contains("<p>Third paragraph.</p>"));
    }

    #[test]
    fn test_fenced_code_block_structure() {
        let text = "```json\n{\"test\": true}\n```";
        let result = render_markdown(text);

        // Verify the structure includes pre > code with language class
        assert!(result.contains("<pre><code class=\"language-json\" data-line-count=\"1\">"));
        assert!(result.contains("{\"test\": true}"));
        assert!(result.contains("</code></pre>"));

        // Test that the JSON content is properly preserved
        assert!(!result.contains("&quot;")); // Should not be double-encoded
    }

    #[test]
    fn test_media_embedding() {
        // Test CommonMark image syntax
        let image_text = "![Alt text](https://example.com/image.jpg)";
        let image_result = render_markdown(image_text);
        assert!(
            image_result.contains("<img src=\"https://example.com/image.jpg\" alt=\"Alt text\">")
        );

        // Test video syntax with caption
        let video_text = "![Video caption](https://example.com/video.mp4)";
        let video_result = render_markdown(video_text);
        assert!(video_result.contains("<div class=\"media-with-caption\">"));
        assert!(video_result.contains("<video controls"));
        assert!(video_result.contains("<source src=\"https://example.com/video.mp4\""));
        assert!(video_result.contains("<div class=\"media-caption\">Video caption</div>"));

        // Test video without caption
        let video_no_caption = "![](https://example.com/video.webm)";
        let video_no_caption_result = render_markdown(video_no_caption);
        assert!(video_no_caption_result.contains("<video controls"));
        assert!(video_no_caption_result.contains("<source src=\"https://example.com/video.webm\""));
        assert!(!video_no_caption_result.contains("<div class=\"media-caption\">"));
        assert!(!video_no_caption_result.contains("<div class=\"media-with-caption\">"));

        // Test image with empty alt text
        let empty_alt = "![](https://example.com/test.png)";
        let empty_alt_result = render_markdown(empty_alt);
        assert!(empty_alt_result.contains("<img src=\"https://example.com/test.png\" alt=\"\">"));
    }

    #[test]
    fn test_commonmark_image_syntax() {
        // Test basic image syntax
        let basic = "![Alt text](https://example.com/image.jpg)";
        let basic_result = render_markdown(basic);
        assert!(
            basic_result.contains("<img src=\"https://example.com/image.jpg\" alt=\"Alt text\">")
        );

        // Test image with special characters in alt text
        let special_alt = "![My \"special\" image & test](https://example.com/test.png)";
        let special_result = render_markdown(special_alt);
        assert!(special_result.contains("alt=\"My &quot;special&quot; image &amp; test\""));

        // Test image with special characters in URL (sanitizer handles escaping)
        let special_url = "![Test](https://example.com/test<>&.png)";
        let url_result = render_markdown(special_url);
        assert!(url_result.contains("<img src="));
        assert!(url_result.contains("alt=\"Test\""));

        // Test multiple images in one text
        let multiple =
            "![First](https://example.com/1.jpg) and ![Second](https://example.com/2.png)";
        let multiple_result = render_markdown(multiple);
        assert!(multiple_result.contains("<img src=\"https://example.com/1.jpg\" alt=\"First\">"));
        assert!(multiple_result.contains("<img src=\"https://example.com/2.png\" alt=\"Second\">"));

        // Test image mixed with text
        let mixed = "Here is an image: ![Cool pic](https://example.com/cool.jpg) - isn't it nice?";
        let mixed_result = render_markdown(mixed);
        assert!(mixed_result.contains("<div class=\"media-with-caption\">"));
        assert!(
            mixed_result.contains("<img src=\"https://example.com/cool.jpg\" alt=\"Cool pic\">")
        );
        assert!(mixed_result.contains("<div class=\"media-caption\">Cool pic</div>"));

        // Test that incomplete syntax is not processed
        let incomplete1 = "![Alt text](no-closing-paren";
        let incomplete1_result = render_markdown(incomplete1);
        assert!(!incomplete1_result.contains("<img"));

        let incomplete2 = "![Alt text without url]";
        let incomplete2_result = render_markdown(incomplete2);
        assert!(!incomplete2_result.contains("<img"));

        // Test that images are processed before links (so ![text](url) doesn't become a link)
        let not_link = "![This should be an image](https://example.com/image.jpg)";
        let not_link_result = render_markdown(not_link);
        assert!(not_link_result.contains("<img"));
        assert!(!not_link_result.contains("<a href"));
    }

    #[test]
    fn test_image_captions() {
        // Test image with alt text shows caption
        let with_alt = "![This is a caption](https://example.com/image.jpg)";
        let with_alt_result = render_markdown(with_alt);
        assert!(with_alt_result.contains("<div class=\"media-with-caption\">"));
        assert!(with_alt_result
            .contains("<img src=\"https://example.com/image.jpg\" alt=\"This is a caption\">"));
        assert!(with_alt_result.contains("<div class=\"media-caption\">This is a caption</div>"));

        // Test image without alt text shows no caption
        let no_alt = "![](https://example.com/image.jpg)";
        let no_alt_result = render_markdown(no_alt);
        assert!(no_alt_result.contains("<img src=\"https://example.com/image.jpg\" alt=\"\">"));
        assert!(!no_alt_result.contains("<div class=\"media-caption\">"));
        assert!(!no_alt_result.contains("<div class=\"media-with-caption\">"));

        // Test image with only whitespace alt text shows no caption
        let whitespace_alt = "![   ](https://example.com/image.jpg)";
        let whitespace_result = render_markdown(whitespace_alt);
        assert!(
            whitespace_result.contains("<img src=\"https://example.com/image.jpg\" alt=\"   \">")
        );
        assert!(!whitespace_result.contains("<div class=\"media-caption\">"));
        assert!(!whitespace_result.contains("<div class=\"media-with-caption\">"));

        // Test image with special characters in alt text
        let special_alt = "![My \"special\" image & test](https://example.com/image.jpg)";
        let special_result = render_markdown(special_alt);
        assert!(special_result.contains("<div class=\"media-with-caption\">"));
        assert!(special_result.contains("alt=\"My &quot;special&quot; image &amp; test\""));
        assert!(special_result
            .contains("<div class=\"media-caption\">My \"special\" image &amp; test</div>"));

        // Test multiple images with different alt text scenarios
        let multiple = "![First caption](img1.jpg) ![](img2.jpg) ![Third caption](img3.jpg)";
        let multiple_result = render_markdown(multiple);
        assert!(multiple_result.contains("<div class=\"media-caption\">First caption</div>"));
        assert!(multiple_result.contains("<div class=\"media-caption\">Third caption</div>"));
        // Count caption occurrences - should be exactly 2
        assert_eq!(
            multiple_result
                .matches("<div class=\"media-caption\">")
                .count(),
            2
        );
        // Count wrapper occurrences - should be exactly 2 (only images with alt text)
        assert_eq!(
            multiple_result
                .matches("<div class=\"media-with-caption\">")
                .count(),
            2
        );
    }

    #[test]
    fn test_image_captions_demo() {
        // Demonstrate the image caption feature
        let demo_text = r#"
# Image Caption Demo

Here's an image with alt text that becomes a caption:
![A beautiful sunset over the mountains](https://example.com/sunset.jpg)

And here's an image without alt text (no caption):
![](https://example.com/no-caption.jpg)

Multiple images:
![First image](img1.jpg) ![Second image](img2.jpg)
"#;

        let result = render_markdown(demo_text);

        // Should have captions for images with alt text
        assert!(result.contains("<div class=\"media-with-caption\">"));
        assert!(result
            .contains("<div class=\"media-caption\">A beautiful sunset over the mountains</div>"));
        assert!(result.contains("<div class=\"media-caption\">First image</div>"));
        assert!(result.contains("<div class=\"media-caption\">Second image</div>"));

        // Should have regular img for image without alt text
        assert!(result.contains("<img src=\"https://example.com/no-caption.jpg\" alt=\"\">"));

        // Verify we have the right number of captions (3 images with alt text)
        assert_eq!(result.matches("<div class=\"media-caption\">").count(), 3);
        assert_eq!(
            result.matches("<div class=\"media-with-caption\">").count(),
            3
        );
    }

    #[test]
    fn test_html_escape_function() {
        // Test the html_escape function directly
        let test_input = r#"<script>alert('xss')</script>"onclick""#;
        let escaped = html_escape(test_input);
        println!("Input: {}", test_input);
        println!("Escaped: {}", escaped);

        assert!(!escaped.contains("<script>"));
        assert!(escaped.contains("&lt;script&gt;"));
        assert!(escaped.contains("&quot;onclick&quot;"));
    }

    #[test]
    fn test_image_alt_text_sanitization() {
        // Test that malicious alt text is properly sanitized in both alt attribute and caption
        let malicious_alt = r#"![<script>alert('xss')</script>](https://example.com/image.jpg)"#;
        let result = render_markdown(malicious_alt);

        // The caption should be properly escaped
        assert!(result.contains("<div class=\"media-with-caption\">"));
        assert!(result.contains(
            "<div class=\"media-caption\">&lt;script&gt;alert('xss')&lt;/script&gt;</div>"
        ));

        // Test with various dangerous characters
        let complex_alt = r#"![<img src=x onerror=alert(1)> & "quotes"](test.jpg)"#;
        let complex_result = render_markdown(complex_alt);

        // The caption content should be escaped (main security concern)
        assert!(complex_result.contains("&lt;img"));
        assert!(complex_result.contains("&amp;"));
        assert!(complex_result.contains("&quot;quotes&quot;"));
    }

    #[test]
    fn test_image_captions_final_demo() {
        // Final demonstration of the image caption feature
        let input = r#"
Check out this image with a caption:
![A beautiful landscape with mountains and trees](https://example.com/landscape.jpg)

And this one without alt text (no caption):
![](https://example.com/no-alt.jpg)

Multiple images:
![First image](img1.jpg) and ![Second image](img2.jpg)
"#;

        let result = render_markdown(input);

        // Image with alt text gets a caption
        assert!(result.contains("<div class=\"media-with-caption\">"));
        assert!(result.contains("<img src=\"https://example.com/landscape.jpg\" alt=\"A beautiful landscape with mountains and trees\">"));
        assert!(result.contains(
            "<div class=\"media-caption\">A beautiful landscape with mountains and trees</div>"
        ));

        // Image without alt text gets no caption
        assert!(result.contains("<img src=\"https://example.com/no-alt.jpg\" alt=\"\">"));
        assert!(!result.contains("<div class=\"media-caption\"></div>"));

        // Multiple images with captions
        assert!(result.contains("<div class=\"media-caption\">First image</div>"));
        assert!(result.contains("<div class=\"media-caption\">Second image</div>"));

        // Verify caption count
        assert_eq!(result.matches("<div class=\"media-caption\">").count(), 3);
        assert_eq!(
            result.matches("<div class=\"media-with-caption\">").count(),
            3
        );
    }

    #[test]
    fn test_image_caption_html_structure() {
        // Test the exact HTML structure produced
        let input = "![Test caption](image.jpg)";
        let result = render_markdown(input);

        // Should produce the wrapper div with both image and caption inside
        assert!(result.contains("<div class=\"media-with-caption\">"));
        assert!(result.contains("<img src=\"image.jpg\" alt=\"Test caption\">"));
        assert!(result.contains("<div class=\"media-caption\">Test caption</div>"));
        assert!(result.contains("</div></div>")); // Both closing divs

        // Test image without caption
        let no_caption = "![](image.jpg)";
        let no_caption_result = render_markdown(no_caption);
        assert!(!no_caption_result.contains("media-with-caption"));
        assert!(no_caption_result.contains("<img src=\"image.jpg\" alt=\"\">"));
    }

    #[test]
    fn test_legacy_url_embedding_removed() {
        // Test that raw URLs no longer get auto-converted to media elements
        let raw_image_url = "Check this out: https://example.com/image.jpg";
        let result = render_markdown(raw_image_url);

        // Should NOT contain img tag (legacy behavior removed)
        assert!(!result.contains("<img"));

        // Should contain the raw URL as text
        assert!(result.contains("https://example.com/image.jpg"));

        // Test video URL
        let raw_video_url = "Watch this: https://example.com/video.mp4";
        let video_result = render_markdown(raw_video_url);

        // Should NOT contain video tag (legacy behavior removed)
        assert!(!video_result.contains("<video"));

        // Should contain the raw URL as text
        assert!(video_result.contains("https://example.com/video.mp4"));
    }

    #[test]
    fn test_video_caption_functionality() {
        // Test various video formats with captions
        let formats = ["mp4", "webm", "ogg", "mov"];

        for format in &formats {
            let video_text = format!(
                "![My {} video](https://example.com/video.{})",
                format, format
            );
            let result = render_markdown(&video_text);

            assert!(result.contains("<div class=\"media-with-caption\">"));
            assert!(result.contains("<video controls"));
            assert!(result.contains(&format!("src=\"https://example.com/video.{}", format)));
            assert!(result.contains(&format!(
                "<div class=\"media-caption\">My {} video</div>",
                format
            )));
        }

        // Test video with special characters in caption
        let special_caption = r#"![My "special" video & test](https://example.com/video.mp4)"#;
        let special_result = render_markdown(special_caption);
        assert!(special_result
            .contains("<div class=\"media-caption\">My \"special\" video &amp; test</div>"));
    }

    #[test]
    fn test_mixed_images_and_videos_with_captions() {
        // Test mixing images and videos with various caption scenarios
        let mixed_content = r#"
Here's an image with a caption:
![Beautiful landscape](https://example.com/image.jpg)

And a video with a caption:
![Awesome video](https://example.com/video.mp4)

Image without caption:
![](https://example.com/no-caption.png)

Video without caption:
![](https://example.com/silent.webm)

Multiple media in one paragraph:
![First image](img1.jpg) and ![First video](vid1.mp4)
"#;

        let result = render_markdown(mixed_content);

        // Check image with caption
        assert!(result.contains("<div class=\"media-with-caption\">"));
        assert!(result
            .contains("<img src=\"https://example.com/image.jpg\" alt=\"Beautiful landscape\">"));
        assert!(result.contains("<div class=\"media-caption\">Beautiful landscape</div>"));

        // Check video with caption
        assert!(result.contains("<video controls"));
        assert!(result.contains("<source src=\"https://example.com/video.mp4\""));
        assert!(result.contains("<div class=\"media-caption\">Awesome video</div>"));

        // Check image without caption (no wrapper)
        assert!(result.contains("<img src=\"https://example.com/no-caption.png\" alt=\"\">"));

        // Check video without caption (no wrapper)
        assert!(result.contains("<source src=\"https://example.com/silent.webm\""));

        // Verify correct number of wrappers (only for media with captions)
        assert_eq!(
            result.matches("<div class=\"media-with-caption\">").count(),
            4
        );
        assert_eq!(result.matches("<div class=\"media-caption\">").count(), 4);
    }

    #[test]
    fn test_consistent_media_naming() {
        // Test that demonstrates the consistent media-* naming convention
        let mixed_media = r#"
Here's an image with caption:
![Beautiful photo](https://example.com/photo.jpg)

And a video with caption:
![Awesome clip](https://example.com/video.mp4)
"#;

        let result = render_markdown(mixed_media);

        // Both images and videos use consistent naming
        assert_eq!(
            result.matches("<div class=\"media-with-caption\">").count(),
            2
        );
        assert_eq!(result.matches("<div class=\"media-caption\">").count(), 2);

        // Check specific captions
        assert!(result.contains("<div class=\"media-caption\">Beautiful photo</div>"));
        assert!(result.contains("<div class=\"media-caption\">Awesome clip</div>"));

        // Check that both wrapper and caption classes are semantic and consistent
        assert!(result.contains("media-with-caption"));
        assert!(result.contains("media-caption"));
        assert!(!result.contains("image-caption"));
        assert!(!result.contains("img-with-caption"));
    }

    #[test]
    fn test_header_edge_cases() {
        // Test headers followed by other block elements
        let edge_cases = r#"# Header Before Code
```rust
let x = 5;
```

## Header Before List
- Item 1
- Item 2

### Header Before Blockquote
> This is a quote

#### Header Before Table
| Col 1 | Col 2 |
|-------|-------|
| A     | B     |

# Header at End"#;

        let result = render_markdown(edge_cases);

        // Check that headers are processed correctly
        assert!(result.contains(
            "<h1 id=\"h1\">Header Before Code<a href=\"#h1\" class=\"header-anchor\" rel=\"noopener noreferrer\">#</a></h1>"
        ));
        assert!(result.contains(
            "<h2 id=\"h2\">Header Before List<a href=\"#h2\" class=\"header-anchor\" rel=\"noopener noreferrer\">#</a></h2>"
        ));
        assert!(result.contains(
            "<h3 id=\"h3\">Header Before Blockquote<a href=\"#h3\" class=\"header-anchor\" rel=\"noopener noreferrer\">#</a></h3>"
        ));
        assert!(result.contains(
            "<h4 id=\"h4\">Header Before Table<a href=\"#h4\" class=\"header-anchor\" rel=\"noopener noreferrer\">#</a></h4>"
        ));
        assert!(result.contains(
            "<h1 id=\"h5\">Header at End<a href=\"#h5\" class=\"header-anchor\" rel=\"noopener noreferrer\">#</a></h1>"
        ));

        // Check that the following elements are still processed correctly
        assert!(result.contains("<pre><code"));
        assert!(result.contains("let x = 5;"));
        assert!(result.contains("<blockquote"));
        assert!(result.contains("<table"));

        // Verify headers are not inside other elements
        assert!(!result.contains("<p><h"));
        assert!(!result.contains("<blockquote><h"));
        assert!(!result.contains("<code><h"));
    }

    #[test]
    fn test_footnotes() {
        // Test reference footnotes
        let reference_text = "This has a footnote[^1] and another[^2].\n\n[^1]: First footnote text.\n[^2]: Second footnote text.";
        let reference_result = render_markdown(reference_text);

        // Check that footnote references are created (link processor adds attributes)
        assert!(reference_result.contains("<sup><a href=\"#FN1\""));
        assert!(reference_result.contains("<sup><a href=\"#FN2\""));
        assert!(reference_result.contains(">1</a></sup>"));
        assert!(reference_result.contains(">2</a></sup>"));
        assert!(reference_result.contains("<div class=\"footnotes\">"));
        assert!(reference_result.contains("First footnote text."));
        assert!(reference_result.contains("Second footnote text."));
        assert!(reference_result.contains("href=\"#fnref1\""));
        assert!(reference_result.contains("href=\"#fnref2\""));

        // Test inline footnotes
        let inline_text =
            "This has an inline footnote^[This is inline] and another^[Second inline].";
        let inline_result = render_markdown(inline_text);

        // Check that inline footnotes are created
        assert!(inline_result.contains("<sup><a href=\"#ifn1\""));
        assert!(inline_result.contains("<sup><a href=\"#ifn2\""));
        assert!(inline_result.contains("This is inline"));
        assert!(inline_result.contains("Second inline"));
        assert!(inline_result.contains("href=\"#ifn1ref\""));
        assert!(inline_result.contains("href=\"#ifn2ref\""));

        // Test mixed footnotes
        let mixed_text = "Reference[^1] and inline^[Inline text].\n\n[^1]: Reference text.";
        let mixed_result = render_markdown(mixed_text);

        assert!(mixed_result.contains("href=\"#FN1\""));
        assert!(mixed_result.contains("href=\"#ifn1\""));
        assert!(mixed_result.contains("Reference text."));
        assert!(mixed_result.contains("Inline text"));

        // Test footnote without definition (should not be processed)
        let undefined_text = "This has undefined[^missing] footnote.";
        let undefined_result = render_markdown(undefined_text);

        assert!(!undefined_result.contains("<sup>"));
        assert!(undefined_result.contains("[^missing]"));
    }

    #[test]
    fn test_footnote_integration() {
        // Test comprehensive footnote functionality with mixed content
        let complex_text = r#"# Document with Footnotes

This is a **bold** text with a reference footnote[^1] and some *italic* text.

Here's an inline footnote^[This is inline content with **formatting**] in the middle.

Another paragraph with multiple footnotes[^ref] and inline^[Another inline note].

> This blockquote also has a footnote[^quote].

```rust
// Code blocks should not process footnotes[^code]
let x = 42;
```

[^1]: First reference footnote with *formatting*.
[^ref]: Reference footnote with a [link](https://example.com).
[^quote]: Footnote from blockquote."#;

        let result = render_markdown(complex_text);

        // Check that reference footnotes work
        assert!(result.contains("href=\"#FN1\""));
        assert!(result.contains("href=\"#FN2\""));
        assert!(result.contains("href=\"#FN3\""));

        // Check that inline footnotes work
        assert!(result.contains("href=\"#ifn1\""));
        assert!(result.contains("href=\"#ifn2\""));

        // Check footnotes section exists
        assert!(result.contains("<div class=\"footnotes\">"));
        assert!(result.contains("First reference footnote"));
        assert!(result.contains("Reference footnote with a"));
        assert!(result.contains("Footnote from blockquote"));
        assert!(result.contains("This is inline content"));
        assert!(result.contains("Another inline note"));

        // Check that code blocks don't process footnotes
        assert!(result.contains("footnotes[^code]"));
        assert!(!result.contains("href=\"#FN4\""));

        // Check that other markdown still works (with correct formatting)
        assert!(result.contains("<h1 id=\"h1\">"));
        assert!(result.contains(
            "<a href=\"#h1\" class=\"header-anchor\" rel=\"noopener noreferrer\">#</a></h1>"
        ));
        assert!(result.contains("<strong>bold</strong>")); // ** is bold, * is italic in this parser
        assert!(result.contains("<em>italic</em>"));
        assert!(result.contains("<blockquote>"));
        assert!(result.contains("href=\"https://example.com\""));
    }

    #[test]
    fn test_paragraph_formatting() {
        // Test single paragraph
        let text = "This is a single paragraph.";
        let result = render_markdown(text);
        assert!(result.contains("<p>This is a single paragraph.</p>"));

        // Test multiple paragraphs
        let text_multi = "First paragraph.\n\nSecond paragraph.";
        let result_multi = render_markdown(text_multi);
        assert!(result_multi.contains("<p>First paragraph.</p>"));
        assert!(result_multi.contains("<p>Second paragraph.</p>"));

        // Test paragraph with inline formatting
        let text_formatted = "This has *italic* and **bold** text.";
        let result_formatted = render_markdown(text_formatted);
        assert!(result_formatted
            .contains("<p>This has <em>italic</em> and <strong>bold</strong> text.</p>"));
    }

    #[test]
    fn test_comprehensive_paragraph_structure() {
        let complex_content = r#"This is the first paragraph with **bold** text.

This is the second paragraph with a `code snippet` inline.

```python
def hello():
    print("This is a code block")
```

This paragraph comes after the code block.

Here's an image: ![Test image](https://example.com/image.jpg)

Final paragraph with *emphasis* and _underline_."#;

        let result = render_markdown(complex_content);

        // Verify paragraph structure
        assert!(
            result.contains("<p>This is the first paragraph with <strong>bold</strong> text.</p>")
        );
        assert!(result.contains(
            "<p>This is the second paragraph with a <code>code snippet</code> inline.</p>"
        ));
        assert!(result.contains("<p>This paragraph comes after the code block.</p>"));
        assert!(
            result.contains("<p>Final paragraph with <em>emphasis</em> and <u>underline</u>.</p>")
        );

        // Code block should be standalone
        assert!(result.contains("<pre><code class=\"language-python\" data-line-count=\"2\">def hello():\n    print(\"This is a code block\")</code></pre>"));

        // Image should be standalone with caption wrapper
        assert!(result.contains("<div class=\"media-with-caption\">"));
        assert!(result.contains("<img src=\"https://example.com/image.jpg\" alt=\"Test image\">"));

        // Should not have any <br> tags (everything should be in proper paragraphs or blocks)
        assert!(!result.contains("<br>"));
    }

    #[test]
    fn test_complex_code_block_scenarios() {
        // Test code block with special characters
        let special_chars = "```json\n{\n  \"message\": \"Hello *world* with **markdown**\",\n  \"tags\": [\"<script>\", \"&amp;\"]\n}\n```";
        let special_result = render_markdown(special_chars);

        assert!(special_result.contains("*world*"));
        assert!(special_result.contains("**markdown**"));
        assert!(special_result.contains("&lt;script&gt;"));
        assert!(special_result.contains("&amp;amp;"));

        // Verify the complete JSON structure is preserved
        assert!(special_result.contains("\"message\": \"Hello *world* with **markdown**\""));
        assert!(special_result.contains("\"tags\": [\"&lt;script&gt;\", \"&amp;amp;\"]"));

        // Test code block with empty lines (should preserve structure)
        let empty_lines =
            "```python\ndef test():\n\n    print('with empty line')\n\n    return True\n```";
        let empty_result = render_markdown(empty_lines);
        assert!(empty_result.contains("<pre><code class=\"language-python\" data-line-count=\"5\">def test():\n\n    print('with empty line')\n\n    return True</code></pre>"));

        // Test mixed content
        let mixed = "Text before\n\n```js\nconsole.log('test');\n```\n\nText after";
        let mixed_result = render_markdown(mixed);
        assert!(mixed_result.contains("<p>Text before</p>"));
        assert!(mixed_result.contains("<p>Text after</p>"));
        assert!(mixed_result.contains(
            "<pre><code class=\"language-javascript\" data-line-count=\"1\">console.log('test');</code></pre>"
        ));
    }

    #[test]
    fn test_line_break_preservation() {
        // Test that single line breaks are preserved as <br> tags
        let text = "First line\nSecond line\nThird line";
        let result = render_markdown(text);
        assert!(result.contains("<p>First line<br>Second line<br>Third line</p>"));

        // Test line breaks mixed with formatting
        let formatted = "Line with *bold*\nAnother line with **italic**";
        let formatted_result = render_markdown(formatted);
        assert!(formatted_result.contains(
            "<p>Line with <em>bold</em><br>Another line with <strong>italic</strong></p>"
        ));

        // Test that double line breaks still create separate paragraphs
        let paragraphs = "First paragraph\n\nSecond paragraph";
        let paragraph_result = render_markdown(paragraphs);
        assert!(paragraph_result.contains("<p>First paragraph</p>"));
        assert!(paragraph_result.contains("<p>Second paragraph</p>"));

        // Test empty lines are ignored within paragraphs
        let with_empty = "Line 1\n\nLine 2\n\nLine 3";
        let empty_result = render_markdown(with_empty);
        assert!(empty_result.contains("<p>Line 1</p>"));
        assert!(empty_result.contains("<p>Line 2</p>"));
        assert!(empty_result.contains("<p>Line 3</p>"));
    }

    #[test]
    fn test_comprehensive_line_break_behavior() {
        // Test mixed content with line breaks
        let mixed_content = r#"This is line 1
This is line 2 with **bold**
This is line 3

New paragraph here
Another line in paragraph

```python
def hello():
    print("world")
```

Third paragraph with *italic* formatting."#;

        let result = render_markdown(mixed_content);

        // First paragraph should have line breaks preserved
        assert!(result.contains(
            "<p>This is line 1<br>This is line 2 with <strong>bold</strong><br>This is line 3</p>"
        ));

        // Second paragraph should have line breaks
        assert!(result.contains("<p>New paragraph here<br>Another line in paragraph</p>"));

        // Code block should be separate
        assert!(result.contains(
            "<pre><code class=\"language-python\" data-line-count=\"2\">def hello():\n    print(\"world\")</code></pre>"
        ));

        // Third paragraph should contain italic formatting
        assert!(result.contains("<p>Third paragraph with <em>italic</em> formatting.</p>"));
    }

    #[test]
    fn test_markdown_tables() {
        // Basic table
        let table_text = "| Name | Age | City |\n|------|-----|------|\n| John | 30  | NYC  |\n| Jane | 25  | LA   |";
        let result = render_markdown(table_text);

        assert!(result.contains("<table>"));
        assert!(result.contains("<thead>"));
        assert!(result.contains("<tbody>"));
        assert!(result.contains("<th>Name</th>"));
        assert!(result.contains("<td>John</td>"));
        assert!(result.contains("</table>"));

        // Table with alignment
        let aligned_table =
            "| Left | Center | Right |\n|:-----|:------:|------:|\n| L1   |   C1   |    R1 |";
        let aligned_result = render_markdown(aligned_table);

        assert!(aligned_result.contains("text-align: center"));
        assert!(aligned_result.contains("text-align: right"));
        assert!(aligned_result.contains("<td>L1</td>"));
        assert!(aligned_result.contains("<td style=\"text-align: center\">C1</td>"));
        assert!(aligned_result.contains("<td style=\"text-align: right\">R1</td>"));
    }

    #[test]
    fn test_tables_with_formatting() {
        // Table with markdown formatting in cells
        let formatted_table = "| **Bold** | *Italic* | `Code` |\n|----------|----------|--------|\n| *test*   | **bold** | `var`  |";
        let result = render_markdown(formatted_table);

        assert!(result.contains("<th><strong>Bold</strong></th>"));
        assert!(result.contains("<th><em>Italic</em></th>"));
        assert!(result.contains("<td><em>test</em></td>"));
        assert!(result.contains("<td><strong>bold</strong></td>"));
    }

    #[test]
    fn test_table_spacing() {
        let simple_table = "| Name | Age |\n|------|-----|\n| John | 30  |";
        let result = render_markdown(simple_table);

        // Should have clean table without extra paragraphs or breaks
        assert!(result.contains("<table>"));
        assert!(!result.contains("<p></p>"));
        assert!(!result.contains("<br><table>"));

        // Test table in context
        let with_text =
            "Here's a table:\n\n| Name | Age |\n|------|-----|\n| John | 30  |\n\nAfter table.";
        let context_result = render_markdown(with_text);
        assert!(context_result.contains("<p>Here's a table:</p>"));
        assert!(context_result.contains("<p>After table.</p>"));
        assert!(context_result.contains("<table>"));
    }

    #[test]
    fn test_headers() {
        // Test headers with blank lines (traditional)
        let text_with_blanks = "# Header 1\n\n## Header 2\n\n### Header 3\n\n#### Header 4";
        let result = render_markdown(text_with_blanks);

        assert!(result
            .contains("<h1 id=\"h1\">Header 1<a href=\"#h1\" class=\"header-anchor\" rel=\"noopener noreferrer\">#</a></h1>"));
        assert!(result
            .contains("<h2 id=\"h2\">Header 2<a href=\"#h2\" class=\"header-anchor\" rel=\"noopener noreferrer\">#</a></h2>"));
        assert!(result
            .contains("<h3 id=\"h3\">Header 3<a href=\"#h3\" class=\"header-anchor\" rel=\"noopener noreferrer\">#</a></h3>"));
        assert!(result
            .contains("<h4 id=\"h4\">Header 4<a href=\"#h4\" class=\"header-anchor\" rel=\"noopener noreferrer\">#</a></h4>"));

        // Test headers without blank lines
        let text_without_blanks = "# Main Header\nThis paragraph follows immediately.\n\n## Sub Header\nAnother paragraph right after.";
        let result2 = render_markdown(text_without_blanks);

        assert!(result2.contains(
            "<h1 id=\"h1\">Main Header<a href=\"#h1\" class=\"header-anchor\" rel=\"noopener noreferrer\">#</a></h1>"
        ));
        assert!(result2.contains(
            "<h2 id=\"h2\">Sub Header<a href=\"#h2\" class=\"header-anchor\" rel=\"noopener noreferrer\">#</a></h2>"
        ));
        assert!(result2.contains("<p>This paragraph follows immediately.</p>"));
        assert!(result2.contains("<p>Another paragraph right after.</p>"));

        // Verify headers are not wrapped in paragraphs
        assert!(!result2.contains("<p><h1>"));
        assert!(!result2.contains("<p><h2>"));
    }

    #[test]
    fn test_blockquotes() {
        let text = "> This is a quote\n> Continued quote\n\nNormal text";
        let result = render_markdown(text);

        assert!(result.contains("<blockquote>This is a quote<br>Continued quote</blockquote>"));
        assert!(result.contains("<p>Normal text</p>"));
    }

    #[test]
    fn test_mixed_headers_and_quotes() {
        let text = "# Title\n\n> A quote\n\n## Subtitle\n\nNormal text";
        let result = render_markdown(text);

        assert!(result
            .contains("<h1 id=\"h1\">Title<a href=\"#h1\" class=\"header-anchor\" rel=\"noopener noreferrer\">#</a></h1>"));
        assert!(result.contains("<blockquote>A quote</blockquote>"));
        assert!(result
            .contains("<h2 id=\"h2\">Subtitle<a href=\"#h2\" class=\"header-anchor\" rel=\"noopener noreferrer\">#</a></h2>"));
        assert!(result.contains("<p>Normal text</p>"));
    }

    #[test]
    fn test_markup_md_specific_case() {
        println!("=== MARKUP.MD SPECIFIC TEST ===");

        // Test the exact pattern from markup.md
        let input = "```\n> This is a quoted text\n```";
        println!("Input: {}", input);

        let result = render_markdown(input);
        println!("Output: {}", result);

        // This should contain the literal text with proper HTML escaping, not formatting
        assert!(result.contains("&gt; This is a quoted text"));
        assert!(!result.contains("**italic**"));

        println!("=== END MARKUP.MD TEST ===");
    }

    #[test]
    fn test_manual_comment_verification() {
        let test_content = r#"# Test File for Comment Functionality

This is a test file to verify that comments work correctly in Nonograph.

// This is a comment that should not appear in HTML output
// But should be visible in the .md version

Here is some **bold text** after a comment.

// Another comment here
*Italic text* should still work normally.

## Section with Comments

// Comment in a section
This paragraph contains normal text.

```javascript
// This is NOT a Nonograph comment, it's JavaScript code
function hello() {
    console.log("Hello world");
}
```

// But this IS a Nonograph comment outside the code block

Final paragraph with normal text."#;

        let html_output = render_markdown(test_content);

        println!("=== MANUAL TEST OUTPUT ===");
        println!("{}", html_output);

        // Verify comments are removed from HTML
        assert!(!html_output.contains("// This is a comment that should not appear"));
        assert!(!html_output.contains("// But should be visible in the .md version"));
        assert!(!html_output.contains("// Another comment here"));
        assert!(!html_output.contains("// Comment in a section"));
        assert!(!html_output.contains("// But this IS a Nonograph comment"));

        // Verify normal formatting still works
        assert!(html_output.contains("<strong>bold text</strong>"));
        assert!(html_output.contains("<em>Italic text</em>"));
        assert!(html_output.contains("<h1 id=\"h1\">Test File for Comment Functionality<a href=\"#h1\" class=\"header-anchor\" rel=\"noopener noreferrer\">#</a></h1>"));
        assert!(html_output.contains(
            "<h2 id=\"h2\">Section with Comments<a href=\"#h2\" class=\"header-anchor\" rel=\"noopener noreferrer\">#</a></h2>"
        ));

        // Verify code block comments are preserved (they're inside code blocks)
        assert!(html_output.contains("// This is NOT a Nonograph comment"));

        println!("✅ All manual verification tests passed!");
    }

    #[test]
    fn test_comments_edge_cases() {
        // Test comment that doesn't start at beginning of line
        let input1 = "Normal text // not a comment\n// This is a comment";
        let result1 = render_markdown(input1);
        assert!(result1.contains("Normal text // not a comment"));
        assert!(!result1.contains("// This is a comment"));

        // Test comment with only "//" (no space)
        let input2 = "//No space comment\n// Space comment";
        let result2 = render_markdown(input2);
        assert!(result2.contains("//No space comment"));
        assert!(!result2.contains("// Space comment"));

        // Test empty comment
        let input3 = "// \n//\nNormal text";
        let result3 = render_markdown(input3);
        assert!(!result3.contains("// "));
        assert!(result3.contains("//"));
        assert!(result3.contains("Normal text"));

        // Test comment with special characters
        let input4 = "// Comment with *bold* and [link](url)\nNormal text";
        let result4 = render_markdown(input4);
        assert!(!result4.contains("// Comment with"));
        assert!(result4.contains("Normal text"));
    }

    #[test]
    fn test_comments() {
        let input = "This is normal text\n// This is a comment\nMore normal text\n// Another comment\nFinal text";
        let result = render_markdown(input);

        // Comments should not appear in HTML output
        assert!(!result.contains("// This is a comment"));
        assert!(!result.contains("// Another comment"));

        // Normal text should still be there
        assert!(result.contains("This is normal text"));
        assert!(result.contains("More normal text"));
        assert!(result.contains("Final text"));
    }

    #[test]
    fn test_comments_with_indentation() {
        let input = "Normal line\n    // Indented comment\n**Bold text**";
        let result = render_markdown(input);

        // Comment should be removed even if indented
        assert!(!result.contains("// Indented comment"));

        // Other formatting should work
        assert!(result.contains("<strong>Bold text</strong>"));
        assert!(result.contains("Normal line"));
    }

    #[test]
    fn test_comments_mixed_with_other_features() {
        let input =
            "# Header\n\n// This is a comment\n**Bold text**\n\n// Another comment\n> Quote";
        let result = render_markdown(input);

        // Comments should not appear
        assert!(!result.contains("// This is a comment"));
        assert!(!result.contains("// Another comment"));

        // Other features should work normally
        assert!(result
            .contains("<h1 id=\"h1\">Header<a href=\"#h1\" class=\"header-anchor\" rel=\"noopener noreferrer\">#</a></h1>"));
        assert!(result.contains("<strong>Bold text</strong>"));
        assert!(result.contains("<blockquote>Quote</blockquote>"));
    }

    #[test]
    fn test_bold_italic_fix_demonstration() {
        // Demonstrate that the fix works correctly
        let text = "This has *italic* text and **bold** text.";
        let result = render_markdown(text);

        // Should render * as italic and ** as bold (standard Markdown)
        assert!(result.contains("<em>italic</em>"));
        assert!(result.contains("<strong>bold</strong>"));

        // Should not have any leftover asterisks
        assert!(!result.contains("*"));

        // Test that **text** doesn't get processed as two *text* anymore
        let bold_only = "**just bold**";
        let bold_result = render_markdown(bold_only);
        assert!(bold_result.contains("<strong>just bold</strong>"));
        assert!(!bold_result.contains("<em>"));
    }

    #[test]
    fn test_ssrf_protection() {
        // Test that dangerous URLs are blocked
        let dangerous_urls = vec![
            "http://192.168.1.1/admin",
            "http://10.0.0.1/config",
            "http://localhost:8009/admin",
            "http://127.0.0.1/secret",
            "http://169.254.169.254/latest/meta-data/",
            "javascript:alert(1)",
            "data:text/html,<script>alert(1)</script>",
            "file:///etc/passwd",
        ];

        for dangerous_url in dangerous_urls {
            let content = format!("![test]({})", dangerous_url);
            let result = render_markdown(&content);

            // Dangerous URLs should not appear in img src attributes
            assert!(
                !result.contains(&format!("src=\"{}\"", dangerous_url)),
                "Dangerous URL {} was not blocked",
                dangerous_url
            );
        }

        // Test that safe URLs are allowed
        let safe_urls = vec![
            "https://example.com/image.jpg",
            "https://cdn.example.com/photo.png",
            "http://example.com/video.mp4",
            "relative-image.jpg",
            "./local/image.png",
            "../parent/image.gif",
        ];

        for safe_url in safe_urls {
            let content = format!("![test]({})", safe_url);
            let result = render_markdown(&content);

            // Safe URLs should appear in img src attributes
            assert!(
                result.contains(&format!("src=\"{}\"", safe_url)),
                "Safe URL {} was incorrectly blocked",
                safe_url
            );
        }
    }

    #[test]
    fn test_header_anchor_functionality() {
        // Test that headers get proper anchor links with sequential numbering
        let content = "# First Header\n\nSome content\n\n## Second Header\n\nMore content\n\n### Third Header\n\n#### Fourth Header";
        let result = render_markdown(content);

        // Check that each header gets the correct sequential ID
        assert!(result.contains(
            "<h1 id=\"h1\">First Header<a href=\"#h1\" class=\"header-anchor\" rel=\"noopener noreferrer\">#</a></h1>"
        ));
        assert!(result.contains(
            "<h2 id=\"h2\">Second Header<a href=\"#h2\" class=\"header-anchor\" rel=\"noopener noreferrer\">#</a></h2>"
        ));
        assert!(result.contains(
            "<h3 id=\"h3\">Third Header<a href=\"#h3\" class=\"header-anchor\" rel=\"noopener noreferrer\">#</a></h3>"
        ));
        assert!(result.contains(
            "<h4 id=\"h4\">Fourth Header<a href=\"#h4\" class=\"header-anchor\" rel=\"noopener noreferrer\">#</a></h4>"
        ));

        // Test that numbering is consistent across multiple renders of same content
        let result2 = render_markdown(content);
        assert_eq!(result, result2);

        // Test mixed header levels maintain correct numbering
        let mixed_content = "## Starting with H2\n\n# Then H1\n\n#### Then H4\n\n### Then H3";
        let mixed_result = render_markdown(mixed_content);

        assert!(mixed_result.contains(
            "<h2 id=\"h1\">Starting with H2<a href=\"#h1\" class=\"header-anchor\" rel=\"noopener noreferrer\">#</a></h2>"
        ));
        assert!(mixed_result.contains(
            "<h1 id=\"h2\">Then H1<a href=\"#h2\" class=\"header-anchor\" rel=\"noopener noreferrer\">#</a></h1>"
        ));
        assert!(mixed_result.contains(
            "<h4 id=\"h3\">Then H4<a href=\"#h3\" class=\"header-anchor\" rel=\"noopener noreferrer\">#</a></h4>"
        ));
        assert!(mixed_result.contains(
            "<h3 id=\"h4\">Then H3<a href=\"#h4\" class=\"header-anchor\" rel=\"noopener noreferrer\">#</a></h3>"
        ));
    }

    #[test]
    fn test_dividers() {
        // Test three stars divider
        let stars_text = "Some text\n***\nMore text";
        let stars_result = render_markdown(stars_text);
        assert!(stars_result.contains("<div class=\"divider-stars\">"));
        assert!(stars_result.contains("<div class=\"asterisk\"><div class=\"center\"></div></div>"));

        // Test single asterisk divider
        let asterisk_text = "Some text\n-*-\nMore text";
        let asterisk_result = render_markdown(asterisk_text);
        assert!(asterisk_result
            .contains("<div class=\"divider-asterisk\"><div class=\"center\"></div></div>"));

        // Test horizontal thin divider
        let thin_text = "Some text\n---\nMore text";
        let thin_result = render_markdown(thin_text);
        assert!(thin_result.contains("<hr class=\"divider-thin\">"));

        // Test horizontal double-line divider
        let double_text = "Some text\n===\nMore text";
        let double_result = render_markdown(double_text);
        assert!(double_result.contains("<hr class=\"divider-double\">"));

        // Test that dividers work with surrounding whitespace
        let whitespace_text = "   ***   ";
        let whitespace_result = render_markdown(whitespace_text);
        assert!(whitespace_result.contains("<div class=\"divider-stars\">"));
        assert!(whitespace_result
            .contains("<div class=\"asterisk\"><div class=\"center\"></div></div>"));

        // Test that partial matches don't trigger dividers
        let partial_text = "This has *** in the middle of text";
        let partial_result = render_markdown(partial_text);
        assert!(!partial_result.contains("<div class=\"divider-stars\">"));

        // Test dividers mixed with other content on same line don't trigger
        let mixed_content_tests = vec![
            "Here is some *** text after",
            "Before text --- and after",
            "Some === content here",
            "Text -*-  more text",
            "# Header with *** stars",
            "## Another --- header",
            "**Bold *** text**",
            "*Italic -*- text*",
        ];

        for mixed_text in mixed_content_tests {
            let mixed_result = render_markdown(mixed_text);
            assert!(!mixed_result.contains("<div class=\"divider-stars\">"));
            assert!(!mixed_result.contains("<div class=\"divider-asterisk\">"));
            assert!(!mixed_result.contains("<hr class=\"divider-thin\">"));
            assert!(!mixed_result.contains("<hr class=\"divider-double\">"));
        }

        // Test multiple dividers
        let multiple_text = "Text\n***\nMore text\n---\nEven more\n===\nFinal text";
        let multiple_result = render_markdown(multiple_text);
        assert!(multiple_result.contains("<div class=\"divider-stars\">"));
        assert!(
            multiple_result.contains("<div class=\"asterisk\"><div class=\"center\"></div></div>")
        );
        assert!(multiple_result.contains("<hr class=\"divider-thin\">"));
        assert!(multiple_result.contains("<hr class=\"divider-double\">"));
    }
}
