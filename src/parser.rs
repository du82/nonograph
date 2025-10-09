fn process_images(text: &str) -> String {
    process_images_with_config(text, &crate::config::Config::default())
}

pub fn render_markdown(content: &str) -> String {
    let (protected_content, fenced_blocks) = extract_fenced_code_blocks(content);
    let (mut working_content, code_blocks) = extract_code_blocks(&protected_content);

    // Process footnotes before text formatting to avoid conflicts with ^ and []
    working_content = process_footnotes(&working_content);

    working_content = safe_replace(&working_content, "*", "*", "<em>", "</em>");
    working_content = safe_replace(&working_content, "**", "**", "<strong>", "</strong>");
    working_content = safe_replace(&working_content, "_", "_", "<u>", "</u>");
    working_content = safe_replace(&working_content, "~", "~", "<del>", "</del>");
    working_content = safe_replace(&working_content, "^", "^", "<sup>", "</sup>");
    working_content = safe_replace(
        &working_content,
        "#",
        "#",
        "<span class=\"secret\">",
        "</span>",
    );

    working_content = process_images(&working_content);
    working_content = process_links(&working_content);
    working_content = process_media_urls(&working_content);
    working_content = process_tables(&working_content);
    working_content = format_paragraphs(&working_content);
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
        if i < chars.len() - 1 && chars[i] == '!' && chars[i + 1] == '[' {
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
                        {
                            result.push_str("<img src=\"");
                            result.push_str(&html_escape(&image_url));
                            result.push_str("\" alt=\"");
                            result.push_str(&html_escape(&alt_text));
                            result.push_str("\">");
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
        .add_tag_attributes("code", &["class"])
        .add_tag_attributes("span", &["class"])
        .add_tag_attributes("th", &["style"])
        .add_tag_attributes("td", &["style"])
        .add_tag_attributes("a", &["href", "target", "id", "class"])
        .add_tag_attributes("div", &["class"])
        .add_tag_attributes("li", &["id"])
        .add_tag_attributes("sup", &["id"])
        .link_rel(Some("noopener noreferrer"));

    builder.clean(&html).to_string()
}

fn process_single_header(text: &str) -> Option<String> {
    let trimmed = text.trim();
    if trimmed.starts_with("#### ") {
        let header_text = &trimmed[5..];
        Some(format!("<h4>{}</h4>", header_text))
    } else if trimmed.starts_with("### ") {
        let header_text = &trimmed[4..];
        Some(format!("<h3>{}</h3>", header_text))
    } else if trimmed.starts_with("## ") {
        let header_text = &trimmed[3..];
        Some(format!("<h2>{}</h2>", header_text))
    } else if trimmed.starts_with("# ") {
        let header_text = &trimmed[2..];
        Some(format!("<h1>{}</h1>", header_text))
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

fn extract_fenced_code_blocks(text: &str) -> (String, Vec<(String, String)>) {
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
            let language = line[fence_length..].trim().to_string();
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
                        fenced_blocks.push((language, code_content));
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

fn restore_fenced_code_blocks(text: &str, fenced_blocks: &[(String, String)]) -> String {
    let mut result = text.to_string();

    for (index, (language, code_content)) in fenced_blocks.iter().enumerate() {
        let placeholder = format!("{{{{FENCEDBLOCK{}}}}}", index);
        let mapped_lang = map_language(language);
        let class_attr = if mapped_lang.is_empty() {
            String::new()
        } else {
            format!(" class=\"language-{}\"", mapped_lang)
        };
        // HTML escape the code content to prevent sanitizer issues
        let escaped_content = html_escape(code_content);
        let replacement = format!("<pre><code{}>{}</code></pre>", class_attr, escaped_content);
        result = result.replace(&placeholder, &replacement);
    }

    result
}

fn format_paragraphs(text: &str) -> String {
    let mut result = String::with_capacity(text.len() + (text.len() / 10));

    let parts: Vec<&str> = text.split("\n\n").collect();

    for (i, part) in parts.iter().enumerate() {
        let trimmed = part.trim();

        if trimmed.is_empty() {
            continue;
        }

        // Check for headers first
        if let Some(header) = process_single_header(trimmed) {
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

fn process_footnotes(text: &str) -> String {
    let mut result = String::new();
    let mut footnote_definitions = std::collections::HashMap::new();
    let mut footnote_counter = 0u32;
    let mut inline_footnote_counter = 0u32;

    // First pass: extract footnote definitions [^id]: text
    let lines: Vec<&str> = text.lines().collect();
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
        if i < chars.len() - 2 && chars[i] == '^' && chars[i + 1] == '[' {
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
        } else if i < chars.len() - 3 && chars[i] == '[' && chars[i + 1] == '^' {
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
            "<sup><a href=\"XHASHXIFN{}\" id=\"ifn{}ref\">{}</a></sup>",
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
                    "<li id=\"fn{}\">{} <a href=\"XHASHXfnref{}\" class=\"footnote-backref\">↩</a></li>\n",
                    number, definition, number
                ));
            }
        }

        // Add inline footnotes
        for (footnote_id, footnote_text) in inline_footnotes.iter() {
            result.push_str(&format!(
                "<li id=\"{}\">{} <a href=\"XHASHX{}ref\" class=\"footnote-backref\">↩</a></li>\n",
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
                    format!("<video controls style=\"width: 100%;\"><source src=\"{}\" type=\"video/mp4\"></video>", url)
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
        assert!(result3.contains("<strong>bold</strong>"));
        assert!(result3.contains("<code>this is code with *asterisks*</code>"));
        // The asterisks inside code should NOT become <strong> tags
        assert!(!result3.contains("<code>this is code with <strong>asterisks</strong></code>"));

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
            result.contains("<pre><code class=\"language-json\">{\"key\": \"value\"}</code></pre>")
        );

        // Test Python code block
        let text_py = "```py\nprint('hello world')\n```";
        let result_py = render_markdown(text_py);
        assert!(result_py
            .contains("<pre><code class=\"language-python\">print('hello world')</code></pre>"));

        // Test JavaScript code block
        let text_js = "```js\nconsole.log('hello');\n```";
        let result_js = render_markdown(text_js);
        assert!(result_js.contains(
            "<pre><code class=\"language-javascript\">console.log('hello');</code></pre>"
        ));

        // Test code block without language
        let text_no_lang = "```\nsome code\n```";
        let result_no_lang = render_markdown(text_no_lang);
        assert!(result_no_lang.contains("<pre><code>some code</code></pre>"));
        assert!(!result_no_lang.contains("class=\"language-"));

        // Test multiline code block
        let text_multi = "```rust\nfn main() {\n    println!(\"Hello, world!\");\n}\n```";
        let result_multi = render_markdown(text_multi);

        assert!(result_multi.contains("<pre><code class=\"language-rust\">fn main() {\n    println!(\"Hello, world!\");\n}</code></pre>"));
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
    fn test_mixed_code_blocks() {
        // Test mixing fenced and inline code blocks
        let text = "Here's some `inline code` and a fenced block:\n```json\n{\"test\": true}\n```\nMore text with `more inline`.";
        let result = render_markdown(text);

        assert!(result.contains("<code>inline code</code>"));
        assert!(result.contains("<code>more inline</code>"));
        assert!(result.contains("<pre><code class=\"language-json\">{\"test\": true}</code></pre>"));
    }

    #[test]
    fn test_fenced_vs_regular_markdown_processing() {
        // Test that shows the clear difference between processed and unprocessed markdown
        let mixed_content = r#"Regular text with *bold* and **italic** formatting.

```js
// This code has *bold* and **italic** but should NOT be processed
const message = "*not bold* and **not italic**";
console.log("[not a link](http://example.com)");
```

More regular text with _underline_ and ~strikethrough~."#;

        let result = render_markdown(mixed_content);

        // Regular text should be processed into HTML
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
        assert!(result.contains("<pre><code class=\"language-json\">{\"test\": true}</code></pre>"));
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
        assert!(result.contains("<pre><code class=\"language-json\">"));
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

        // Test automatic URL embedding (legacy behavior)
        let text = "https://example.com/image.jpg";
        let result = render_markdown(text);
        assert!(result.contains("<img src=\"https://example.com/image.jpg\""));

        // Test video embedding
        let video_text = "https://example.com/video.mp4";
        let video_result = render_markdown(video_text);

        assert!(video_result.contains("<video controls=\"\" style=\"width: 100%;\""));
        assert!(video_result
            .contains("<source src=\"https://example.com/video.mp4\" type=\"video/mp4\""));
        assert!(!video_result.contains("Your browser does not support"));

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
        assert!(mixed_result.contains("Here is an image: <img src=\"https://example.com/cool.jpg\" alt=\"Cool pic\"> - isn't it nice?"));

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
        assert!(inline_result.contains("<sup><a href=\"#IFN1\""));
        assert!(inline_result.contains("<sup><a href=\"#IFN2\""));
        assert!(inline_result.contains("This is inline"));
        assert!(inline_result.contains("Second inline"));
        assert!(inline_result.contains("href=\"#ifn1ref\""));
        assert!(inline_result.contains("href=\"#ifn2ref\""));

        // Test mixed footnotes
        let mixed_text = "Reference[^1] and inline^[Inline text].\n\n[^1]: Reference text.";
        let mixed_result = render_markdown(mixed_text);

        assert!(mixed_result.contains("href=\"#FN1\""));
        assert!(mixed_result.contains("href=\"#IFN1\""));
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
        assert!(result.contains("href=\"#IFN1\""));
        assert!(result.contains("href=\"#IFN2\""));

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
        assert!(result.contains("<h1>"));
        assert!(result.contains("<em>bold</em>")); // ** is italic, * is bold in this parser
        assert!(result.contains("<strong>italic</strong>"));
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
        let text_formatted = "This has *bold* and **italic** text.";
        let result_formatted = render_markdown(text_formatted);
        assert!(result_formatted
            .contains("<p>This has <strong>bold</strong> and <em>italic</em> text.</p>"));
    }

    #[test]
    fn test_comprehensive_paragraph_structure() {
        let complex_content = r#"This is the first paragraph with *bold* text.

This is the second paragraph with a `code snippet` inline.

```python
def hello():
    print("This is a code block")
```

This paragraph comes after the code block.

Here's an image: https://example.com/image.jpg

Final paragraph with **emphasis** and _underline_."#;

        let result = render_markdown(complex_content);

        // Should have proper paragraph structure
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
        assert!(result.contains("<pre><code class=\"language-python\">def hello():\n    print(\"This is a code block\")</code></pre>"));

        // Image should be standalone
        assert!(result.contains("<img src=\"https://example.com/image.jpg\""));

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
        assert!(empty_result.contains("<pre><code class=\"language-python\">def test():\n\n    print('with empty line')\n\n    return True</code></pre>"));

        // Test mixed content
        let mixed = "Text before\n\n```js\nconsole.log('test');\n```\n\nText after";
        let mixed_result = render_markdown(mixed);
        assert!(mixed_result.contains("<p>Text before</p>"));
        assert!(mixed_result.contains("<p>Text after</p>"));
        assert!(mixed_result.contains(
            "<pre><code class=\"language-javascript\">console.log('test');</code></pre>"
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
            "<p>Line with <strong>bold</strong><br>Another line with <em>italic</em></p>"
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
This is line 2 with *bold*
This is line 3

New paragraph here
Another line in paragraph

```python
def hello():
    print("world")
```

Final paragraph
With line breaks"#;

        let result = render_markdown(mixed_content);

        // First paragraph should have line breaks preserved
        assert!(result.contains(
            "<p>This is line 1<br>This is line 2 with <strong>bold</strong><br>This is line 3</p>"
        ));

        // Second paragraph should have line breaks
        assert!(result.contains("<p>New paragraph here<br>Another line in paragraph</p>"));

        // Code block should be separate
        assert!(result.contains(
            "<pre><code class=\"language-python\">def hello():\n    print(\"world\")</code></pre>"
        ));

        // Final paragraph should have line breaks
        assert!(result.contains("<p>Final paragraph<br>With line breaks</p>"));
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

        assert!(result.contains("<th><em>Bold</em></th>"));
        assert!(result.contains("<th><strong>Italic</strong></th>"));
        assert!(result.contains("<td><strong>test</strong></td>"));
        assert!(result.contains("<td><em>bold</em></td>"));
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
        let text = "# Header 1\n\n## Header 2\n\n### Header 3\n\n#### Header 4";
        let result = render_markdown(text);

        assert!(result.contains("<h1>Header 1</h1>"));
        assert!(result.contains("<h2>Header 2</h2>"));
        assert!(result.contains("<h3>Header 3</h3>"));
        assert!(result.contains("<h4>Header 4</h4>"));
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

        assert!(result.contains("<h1>Title</h1>"));
        assert!(result.contains("<blockquote>A quote</blockquote>"));
        assert!(result.contains("<h2>Subtitle</h2>"));
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
}
