use chrono::{DateTime, Utc};
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;

#[derive(Debug, Deserialize)]
struct TelegraphResponse {
    ok: bool,
    result: Option<TelegraphPage>,
    error: Option<String>,
}

#[derive(Debug, Deserialize)]
struct TelegraphPage {
    path: String,
    url: String,
    title: String,
    #[allow(dead_code)]
    description: String,
    author_name: Option<String>,
    author_url: Option<String>,
    #[allow(dead_code)]
    image_url: Option<String>,
    content: Option<Vec<Node>>,
    views: u32,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(untagged)]
enum Node {
    Text(String),
    Element(NodeElement),
}

#[derive(Debug, Deserialize, Clone)]
struct NodeElement {
    tag: String,
    attrs: Option<HashMap<String, String>>,
    children: Option<Vec<Node>>,
}

pub struct TelegraphArchiver;

impl TelegraphArchiver {
    pub fn new() -> Self {
        Self
    }

    pub async fn archive_url(&self, url: &str) -> Result<String, Box<dyn std::error::Error>> {
        // Extract path from Telegraph URL
        let path = self.extract_path_from_url(url)?;

        // Fetch content from Telegraph API
        let page = self.fetch_telegraph_page(&path).await?;

        // Convert to Nonograph markdown
        let markdown = self.convert_to_markdown(&page)?;

        // Generate filename and save
        let filename = self.generate_filename(&page);
        let file_path = format!("content/{}", filename);

        // Save to content directory
        fs::write(&file_path, markdown)?;

        // Return Nonograph URL
        let nonograph_id = filename.trim_end_matches(".md");
        Ok(format!("/{}", nonograph_id))
    }

    fn extract_path_from_url(&self, url: &str) -> Result<String, Box<dyn std::error::Error>> {
        let parsed_url = url::Url::parse(url)?;

        // Verify it's a Telegraph URL
        if parsed_url.host_str() != Some("telegra.ph") {
            return Err("URL is not a Telegraph page".into());
        }

        // Extract path (remove leading slash)
        let path = parsed_url.path().trim_start_matches('/');
        if path.is_empty() {
            return Err("Invalid Telegraph URL - no path found".into());
        }

        Ok(path.to_string())
    }

    async fn fetch_telegraph_page(
        &self,
        path: &str,
    ) -> Result<TelegraphPage, Box<dyn std::error::Error>> {
        let api_url = format!(
            "https://api.telegra.ph/getPage/{}?return_content=true",
            path
        );

        let response = reqwest::get(&api_url).await?;
        let telegraph_response: TelegraphResponse = response.json().await?;

        if !telegraph_response.ok {
            return Err(format!("Telegraph API error: {:?}", telegraph_response.error).into());
        }

        telegraph_response
            .result
            .ok_or("No result in Telegraph response".into())
    }

    fn convert_to_markdown(
        &self,
        page: &TelegraphPage,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let mut markdown = String::new();

        // Use current date as archival date
        let now: DateTime<Utc> = Utc::now();
        let archival_date = format!("{}", now.format("%B %d, %Y"));

        // Add date and author in proper format
        if let Some(author) = &page.author_name {
            markdown.push_str(&format!("{} | {}\n\n", archival_date, author));
        } else {
            markdown.push_str(&format!("{}\n\n", archival_date));
        }

        // Add title as H1
        markdown.push_str(&format!("# {}\n", page.title));

        // Convert content
        if let Some(content) = &page.content {
            for node in content {
                self.convert_node_to_markdown(node, &mut markdown, 0)?;
            }
        }

        // Clean up excessive newlines
        let cleaned = self.clean_excessive_newlines(&markdown);
        Ok(cleaned)
    }

    fn convert_node_to_markdown(
        &self,
        node: &Node,
        output: &mut String,
        depth: usize,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.convert_node_to_markdown_with_context(node, output, depth, None)
    }

    fn convert_node_to_markdown_with_context(
        &self,
        node: &Node,
        output: &mut String,
        depth: usize,
        image_caption: Option<&str>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        match node {
            Node::Text(text) => {
                output.push_str(text);
            }
            Node::Element(element) => {
                match element.tag.as_str() {
                    "p" => {
                        if let Some(children) = &element.children {
                            for child in children {
                                self.convert_node_to_markdown_with_context(
                                    child,
                                    output,
                                    depth,
                                    image_caption,
                                )?;
                            }
                        }
                        output.push_str("\n\n");
                    }
                    "br" => {
                        output.push_str("\n");
                    }
                    "strong" | "b" => {
                        output.push_str("**");
                        if let Some(children) = &element.children {
                            for child in children {
                                self.convert_node_to_markdown_with_context(
                                    child,
                                    output,
                                    depth,
                                    image_caption,
                                )?;
                            }
                        }
                        output.push_str("**");
                    }
                    "em" | "i" => {
                        output.push_str("*");
                        if let Some(children) = &element.children {
                            for child in children {
                                self.convert_node_to_markdown_with_context(
                                    child,
                                    output,
                                    depth,
                                    image_caption,
                                )?;
                            }
                        }
                        output.push_str("*");
                    }
                    "u" => {
                        output.push_str("_");
                        if let Some(children) = &element.children {
                            for child in children {
                                self.convert_node_to_markdown_with_context(
                                    child,
                                    output,
                                    depth,
                                    image_caption,
                                )?;
                            }
                        }
                        output.push_str("_");
                    }
                    "s" => {
                        output.push_str("~");
                        if let Some(children) = &element.children {
                            for child in children {
                                self.convert_node_to_markdown_with_context(
                                    child,
                                    output,
                                    depth,
                                    image_caption,
                                )?;
                            }
                        }
                        output.push_str("~");
                    }
                    "code" => {
                        output.push_str("`");
                        if let Some(children) = &element.children {
                            for child in children {
                                self.convert_node_to_markdown_with_context(
                                    child,
                                    output,
                                    depth,
                                    image_caption,
                                )?;
                            }
                        }
                        output.push_str("`");
                    }
                    "pre" => {
                        output.push_str("```\n");
                        if let Some(children) = &element.children {
                            for child in children {
                                self.convert_node_to_markdown_with_context(
                                    child,
                                    output,
                                    depth,
                                    image_caption,
                                )?;
                            }
                        }
                        output.push_str("\n```\n\n");
                    }
                    "a" => {
                        if let Some(attrs) = &element.attrs {
                            if let Some(href) = attrs.get("href") {
                                output.push_str("[");
                                if let Some(children) = &element.children {
                                    for child in children {
                                        self.convert_node_to_markdown_with_context(
                                            child,
                                            output,
                                            depth,
                                            image_caption,
                                        )?;
                                    }
                                } else {
                                    output.push_str(href);
                                }
                                output.push_str(&format!("]({})", href));
                            }
                        }
                    }
                    "img" => {
                        if let Some(attrs) = &element.attrs {
                            if let Some(src) = attrs.get("src") {
                                let full_url = if src.starts_with("/file/") {
                                    format!("https://telegra.ph{}", src)
                                } else {
                                    src.clone()
                                };
                                let caption = image_caption.unwrap_or("");
                                output.push_str(&format!("![{}]({})", caption, full_url));
                            }
                        }
                    }
                    "h3" => {
                        output.push_str("### ");
                        if let Some(children) = &element.children {
                            for child in children {
                                self.convert_node_to_markdown_with_context(
                                    child,
                                    output,
                                    depth,
                                    image_caption,
                                )?;
                            }
                        }
                        output.push_str("\n\n");
                    }
                    "h4" => {
                        output.push_str("#### ");
                        if let Some(children) = &element.children {
                            for child in children {
                                self.convert_node_to_markdown_with_context(
                                    child,
                                    output,
                                    depth,
                                    image_caption,
                                )?;
                            }
                        }
                        output.push_str("\n\n");
                    }
                    "blockquote" => {
                        output.push_str("> ");
                        if let Some(children) = &element.children {
                            for child in children {
                                self.convert_node_to_markdown_with_context(
                                    child,
                                    output,
                                    depth,
                                    image_caption,
                                )?;
                            }
                        }
                        output.push_str("\n\n");
                    }
                    "ul" => {
                        if let Some(children) = &element.children {
                            for child in children {
                                self.convert_node_to_markdown_with_context(
                                    child,
                                    output,
                                    depth,
                                    image_caption,
                                )?;
                            }
                        }
                        output.push_str("\n");
                    }
                    "ol" => {
                        if let Some(children) = &element.children {
                            for (i, child) in children.iter().enumerate() {
                                if let Node::Element(li) = child {
                                    if li.tag == "li" {
                                        output.push_str(&format!("{}. ", i + 1));
                                        if let Some(li_children) = &li.children {
                                            for li_child in li_children {
                                                self.convert_node_to_markdown_with_context(
                                                    li_child,
                                                    output,
                                                    depth,
                                                    image_caption,
                                                )?;
                                            }
                                        }
                                        output.push_str("\n");
                                    }
                                }
                            }
                        }
                        output.push_str("\n");
                    }
                    "li" => {
                        // Handle unordered list items
                        if depth == 0 {
                            output.push_str("- ");
                        }
                        if let Some(children) = &element.children {
                            for child in children {
                                self.convert_node_to_markdown_with_context(
                                    child,
                                    output,
                                    depth + 1,
                                    image_caption,
                                )?;
                            }
                        }
                        if depth == 0 {
                            output.push_str("\n");
                        }
                    }
                    "hr" => {
                        output.push_str("---\n\n");
                    }
                    "figure" => {
                        // Extract caption from figcaption first
                        let mut caption = String::new();
                        if let Some(children) = &element.children {
                            for child in children {
                                if let Node::Element(elem) = child {
                                    if elem.tag == "figcaption" {
                                        if let Some(caption_children) = &elem.children {
                                            for caption_child in caption_children {
                                                if let Node::Text(text) = caption_child {
                                                    caption.push_str(text);
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        // Now process all children with the caption context
                        if let Some(children) = &element.children {
                            for child in children {
                                if let Node::Element(elem) = child {
                                    if elem.tag != "figcaption" {
                                        let caption_ref = if caption.is_empty() {
                                            None
                                        } else {
                                            Some(caption.as_str())
                                        };
                                        self.convert_node_to_markdown_with_context(
                                            child,
                                            output,
                                            depth,
                                            caption_ref,
                                        )?;
                                    }
                                }
                            }
                        }
                        output.push_str("\n");
                    }
                    "figcaption" => {
                        // Figcaptions are now handled by the figure element above
                        // Skip processing them directly to avoid duplication
                    }
                    "iframe" | "video" => {
                        if let Some(attrs) = &element.attrs {
                            if let Some(src) = attrs.get("src") {
                                let full_url = if src.starts_with("/file/") {
                                    format!("https://telegra.ph{}", src)
                                } else {
                                    src.clone()
                                };
                                let caption = image_caption.unwrap_or("Video");
                                output.push_str(&format!("![{}]({})\n\n", caption, full_url));
                            }
                        }
                    }
                    "aside" => {
                        // Convert aside to blockquote
                        output.push_str("> ");
                        if let Some(children) = &element.children {
                            for child in children {
                                self.convert_node_to_markdown_with_context(
                                    child,
                                    output,
                                    depth,
                                    image_caption,
                                )?;
                            }
                        }
                        output.push_str("\n\n");
                    }
                    _ => {
                        // For unhandled tags, just process children
                        if let Some(children) = &element.children {
                            for child in children {
                                self.convert_node_to_markdown_with_context(
                                    child,
                                    output,
                                    depth,
                                    image_caption,
                                )?;
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }

    fn clean_excessive_newlines(&self, content: &str) -> String {
        // Replace 3+ consecutive newlines with just 2
        let mut result = content.to_string();
        while result.contains("\n\n\n") {
            result = result.replace("\n\n\n", "\n\n");
        }
        result
    }

    fn generate_filename(&self, page: &TelegraphPage) -> String {
        // `page.path` comes from a remote response, so it can be anything the
        // Telegraph API returns -- including ".", "..", "" or a value that is
        // nothing but separators. This name is used to *write* a file, and the
        // matching *read* goes through `is_valid_post_id`, which accepts only
        // ASCII alphanumerics, '-' and '_'. Previously any other character was
        // kept (only a shell-safety set was replaced), so a path such as
        // "a/../../etc/passwd" produced "a-..-..-etc-passwd.md": written fine,
        // and then never served, because the reader rejects the id naming it.
        //
        // Apply the reader's character set here instead of a separate one.
        let mapped: String = page
            .path
            .chars()
            .map(|c| if c.is_ascii_alphanumeric() || c == '-' || c == '_' { c } else { '-' })
            .collect();

        // Collapse runs and trim the ends, so a separator-only path yields an
        // empty stem rather than a "-"-only one.
        let mut stem = String::with_capacity(mapped.len());
        let mut last_dash = false;
        for c in mapped.chars() {
            if c == '-' {
                if !last_dash {
                    stem.push(c);
                }
                last_dash = true;
            } else {
                stem.push(c);
                last_dash = false;
            }
        }
        stem = stem.trim_matches('-').to_string();

        // An empty stem is the one case with no recoverable name. "." and ".."
        // also land here, which is acceptable: nothing derived from them is a
        // meaningful slug, and each must still be servable.
        if stem.is_empty() {
            stem = "untitled".to_string();
        }

        // Bound the stem the way `is_valid_post_id` bounds the id it is read
        // back with, reserving room for the extension appended below
        // (`MAX_POST_ID_LEN` is 255).
        const MAX_STEM_LEN: usize = 250;
        if stem.len() > MAX_STEM_LEN {
            stem.truncate(MAX_STEM_LEN);
            stem = stem.trim_end_matches('-').to_string();
            if stem.is_empty() {
                stem = "untitled".to_string();
            }
        }

        if stem.ends_with(".md") {
            stem
        } else {
            format!("{}.md", stem)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_path_from_url() {
        let archiver = TelegraphArchiver::new();

        let url = "https://telegra.ph/Sample-Page-12-15";
        let path = archiver.extract_path_from_url(url).unwrap();
        assert_eq!(path, "Sample-Page-12-15");

        let invalid_url = "https://example.com/page";
        assert!(archiver.extract_path_from_url(invalid_url).is_err());
    }

    fn page_with_path(path: &str) -> TelegraphPage {
        TelegraphPage {
            path: path.to_string(),
            url: format!("https://telegra.ph/{}", path),
            title: "Sample Page".to_string(),
            description: "A sample page".to_string(),
            author_name: None,
            author_url: None,
            image_url: None,
            content: None,
            views: 100,
        }
    }

    #[test]
    fn test_generate_filename() {
        let archiver = TelegraphArchiver::new();
        let filename = archiver.generate_filename(&page_with_path("Sample-Page-12-15"));
        assert_eq!(filename, "Sample-Page-12-15.md");
    }

    /// The filename is what a *write* creates and what a *read* has to name.
    /// `is_valid_post_id` gates the read, so a name it rejects is a file that
    /// is written and then never served.
    #[test]
    fn test_generated_filenames_are_servable() {
        let archiver = TelegraphArchiver::new();
        for path in [
            "Sample-Page-12-15",
            "about",
            "some_post_1",
            ".",
            "..",
            "...",
            "",
            "/",
            "///",
            ".env",
            ".hidden.md",
            ".a.b",
            "a/../../etc/passwd",
            "..\\..\\win",
            "C:\\windows\\system32",
            "Q?x*\"y<z>|w",
            "trailing space ",
            "  leading",
            "unicode-日本語-ページ",
        ] {
            let filename = archiver.generate_filename(&page_with_path(path));
            let stem = filename.strip_suffix(".md").expect("filename must end in .md");
            assert!(
                crate::is_valid_post_id(stem),
                "path {path:?} produced {filename:?}, whose id {stem:?} is not readable"
            );
            assert!(
                !filename.starts_with('.'),
                "path {path:?} produced the hidden filename {filename:?}"
            );
        }
    }

    /// A path made only of separators or dots names nothing. It must still
    /// produce a servable file rather than the previous `.md` dotfile.
    #[test]
    fn test_degenerate_paths_fall_back_to_untitled() {
        let archiver = TelegraphArchiver::new();
        for path in [".", "..", "...", "", "/", "///", ".../..."] {
            assert_eq!(
                archiver.generate_filename(&page_with_path(path)),
                "untitled.md",
                "path {path:?} should fall back to a servable name"
            );
        }
    }

    /// Regression: these all produced a written-but-unreadable file before,
    /// because only a shell-safety character set was replaced.
    ///
    /// Note the deliberate consequence of adopting the reader's ASCII set: a
    /// path with non-ASCII segments loses them (`unicode-日本語-ページ` becomes
    /// `unicode.md`). That is the price of a name the reader will accept, and
    /// it replaces a name the reader could never accept at all.
    #[test]
    fn test_separators_and_dots_do_not_survive_into_the_filename() {
        let archiver = TelegraphArchiver::new();
        let cases = [
            ("a/../../etc/passwd", "a-etc-passwd.md"),
            (".env", "env.md"),
            (".hidden.md", "hidden-md.md"),
            (".a.b", "a-b.md"),
            ("..\\..\\win", "win.md"),
            ("Q?x*\"y<z>|w", "Q-x-y-z-w.md"),
            ("trailing space ", "trailing-space.md"),
            ("  leading", "leading.md"),
            ("unicode-日本語-ページ", "unicode.md"),
        ];
        for (path, expected) in cases {
            let actual = archiver.generate_filename(&page_with_path(path));
            assert_eq!(actual, expected, "path {path:?}");
            let stem = actual.strip_suffix(".md").expect("must end in .md");
            assert!(
                crate::is_valid_post_id(stem),
                "path {path:?} produced an unreadable id {stem:?}"
            );
        }
    }

    /// `is_valid_post_id` rejects ids longer than 255 bytes, so the stem has to
    /// stay bounded with room for the extension.
    #[test]
    fn test_long_path_is_bounded_to_a_readable_id() {
        let archiver = TelegraphArchiver::new();
        let filename = archiver.generate_filename(&page_with_path(&"a".repeat(400)));
        let stem = filename.strip_suffix(".md").expect("must end in .md");
        assert!(crate::is_valid_post_id(stem), "{filename:?} is not readable");
    }

    /// Distinct ordinary paths must keep distinct names.
    #[test]
    fn test_distinct_ordinary_paths_do_not_collide() {
        let archiver = TelegraphArchiver::new();
        let a = archiver.generate_filename(&page_with_path("first-page"));
        let b = archiver.generate_filename(&page_with_path("second-page"));
        assert_ne!(a, b);
    }
}
