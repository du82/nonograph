use chrono::{Datelike, NaiveDate, Utc};
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

        // Prefer the HTML `<time>` date (has the year), then the path's MM-DD, then the archive date.
        let published_date = self.fetch_published_date(&page.url).await;

        // Convert to Nonograph markdown
        let markdown = self.convert_to_markdown(&page, published_date)?;

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

    /// Recover the publish date from a Telegraph path's `-MM-DD` suffix, inferring the year from `reference` (stepping back a year if the date would be in the future); `None` if no valid suffix.
    fn parse_published_date(path: &str, reference: NaiveDate) -> Option<NaiveDate> {
        // Scan from the end for a `MM-DD` pair, skipping a trailing dedup suffix like `-2`.
        let segments: Vec<&str> = path.split('-').collect();

        for end in (2..=segments.len()).rev() {
            let month_part = segments[end - 2];
            let day_part = segments[end - 1];

            // Both parts must be exactly two digits to count as `MM-DD`.
            if month_part.len() != 2 || day_part.len() != 2 {
                continue;
            }
            let (Ok(month), Ok(day)) = (month_part.parse::<u32>(), day_part.parse::<u32>()) else {
                continue;
            };

            let year = reference.year();
            if let Some(date) = NaiveDate::from_ymd_opt(year, month, day) {
                // A future date means the article was published late last year.
                if date > reference {
                    return NaiveDate::from_ymd_opt(year - 1, month, day).or(Some(date));
                }
                return Some(date);
            }
        }

        None
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

    /// Extract the publish date from the HTML page's `<time>` element (the only source with the year); `None` on any network or parse failure.
    async fn fetch_published_date(&self, url: &str) -> Option<NaiveDate> {
        let html = reqwest::get(url).await.ok()?.text().await.ok()?;
        Self::extract_datetime_from_html(&html)
    }

    /// Pull the leading `YYYY-MM-DD` out of a page's `<time datetime="...">` ISO 8601 attribute.
    fn extract_datetime_from_html(html: &str) -> Option<NaiveDate> {
        let needle = "datetime=\"";
        let start = html.find(needle)? + needle.len();
        let rest = &html[start..];
        let end = rest.find('"')?;
        let datetime = &rest[..end];

        // Keep only the date portion before the `T` separator.
        let date_part = datetime.split('T').next()?;
        NaiveDate::parse_from_str(date_part, "%Y-%m-%d").ok()
    }

    fn convert_to_markdown(
        &self,
        page: &TelegraphPage,
        published_date: Option<NaiveDate>,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let mut markdown = String::new();

        // Resolve date by reliability: HTML `<time>`, then path MM-DD, then archive date.
        let today = Utc::now().date_naive();
        let published = published_date
            .or_else(|| Self::parse_published_date(&page.path, today))
            .unwrap_or(today);
        let date_str = published.format("%Y-%m-%d").to_string();

        // Emit YAML frontmatter matching the canonical format in save.rs.
        markdown.push_str("---\n");
        markdown.push_str(&format!("title: {}\n", page.title));
        markdown.push_str(&format!("date: {}\n", date_str));
        if let Some(author) = &page.author_name {
            markdown.push_str(&format!("author: {}\n", author));
        }
        markdown.push_str(&format!(
            "generator: {} v{}\n",
            env!("CARGO_PKG_NAME"),
            env!("CARGO_PKG_VERSION")
        ));
        // Record the URL this page was archived from.
        markdown.push_str(&format!("origin: {}\n", page.url));
        markdown.push_str("---\n\n");

        // The title lives in the frontmatter; don't duplicate it as an H1 in the body.
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
                        // Unordered list items.
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
                        // Extract the figcaption first.
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

                        // Process the remaining children with the caption context.
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
                        // Handled by the figure element above; skip to avoid duplication.
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
                        // Convert aside to blockquote.
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
                        // Unhandled tags: just process children.
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
        // Use the Telegraph path as a filesystem-safe base.
        let mut filename = page.path.clone();

        // Replace unsafe characters.
        filename = filename.replace('/', "-");
        filename = filename.replace('\\', "-");
        filename = filename.replace(':', "-");
        filename = filename.replace('?', "-");
        filename = filename.replace('*', "-");
        filename = filename.replace('"', "-");
        filename = filename.replace('<', "-");
        filename = filename.replace('>', "-");
        filename = filename.replace('|', "-");

        // Ensure it ends with .md.
        if !filename.ends_with(".md") {
            filename.push_str(".md");
        }

        filename
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

    #[test]
    fn test_generate_filename() {
        let archiver = TelegraphArchiver::new();
        let page = TelegraphPage {
            path: "Sample-Page-12-15".to_string(),
            url: "https://telegra.ph/Sample-Page-12-15".to_string(),
            title: "Sample Page".to_string(),
            description: "A sample page".to_string(),
            author_name: None,
            author_url: None,
            image_url: None,
            content: None,
            views: 100,
        };

        let filename = archiver.generate_filename(&page);
        assert_eq!(filename, "Sample-Page-12-15.md");
    }

    #[test]
    fn test_convert_to_markdown_emits_yaml_frontmatter() {
        let archiver = TelegraphArchiver::new();
        let page = TelegraphPage {
            path: "Sample-Page-12-15".to_string(),
            url: "https://telegra.ph/Sample-Page-12-15".to_string(),
            title: "Sample Page".to_string(),
            description: "A sample page".to_string(),
            author_name: Some("Jane Doe".to_string()),
            author_url: None,
            image_url: None,
            content: None,
            views: 100,
        };

        // An explicit published date is used verbatim, including its year.
        let published = NaiveDate::from_ymd_opt(2017, 12, 15);
        let markdown = archiver.convert_to_markdown(&page, published).unwrap();

        // YAML frontmatter block with the expected fields.
        assert!(markdown.starts_with("---\n"));
        assert!(markdown.contains("\n---\n"));
        assert!(markdown.contains("title: Sample Page\n"));
        assert!(markdown.contains("author: Jane Doe\n"));
        assert!(markdown.contains(&format!(
            "generator: {} v{}\n",
            env!("CARGO_PKG_NAME"),
            env!("CARGO_PKG_VERSION")
        )));
        // The origin records the URL the page was archived from.
        assert!(markdown.contains("origin: https://telegra.ph/Sample-Page-12-15\n"));

        // The explicit published date wins, with its exact year.
        assert!(markdown.contains("date: 2017-12-15\n"));

        // The title must not be duplicated as an H1 in the body.
        assert!(!markdown.contains("# Sample Page"));
    }

    #[test]
    fn test_parse_published_date() {
        // Reference date fixed so the year inference is deterministic.
        let reference = NaiveDate::from_ymd_opt(2026, 6, 15).unwrap();

        // A path with a `-MM-DD` suffix earlier in the year uses the current year.
        assert_eq!(
            TelegraphArchiver::parse_published_date("Sample-Page-03-10", reference),
            Some(NaiveDate::from_ymd_opt(2026, 3, 10).unwrap())
        );

        // A date that would fall after the archive date is treated as last year.
        assert_eq!(
            TelegraphArchiver::parse_published_date("Year-End-12-31", reference),
            Some(NaiveDate::from_ymd_opt(2025, 12, 31).unwrap())
        );

        // The archive date itself is valid (not in the future).
        assert_eq!(
            TelegraphArchiver::parse_published_date("Same-Day-06-15", reference),
            Some(NaiveDate::from_ymd_opt(2026, 6, 15).unwrap())
        );

        // A trailing numeric dedup suffix (`-2`) is skipped to find `MM-DD`.
        assert_eq!(
            TelegraphArchiver::parse_published_date("Dup-Page-03-10-2", reference),
            Some(NaiveDate::from_ymd_opt(2026, 3, 10).unwrap())
        );

        // Paths without a valid two-digit MM-DD suffix return None.
        assert_eq!(
            TelegraphArchiver::parse_published_date("No-Date-Here", reference),
            None
        );
        assert_eq!(
            TelegraphArchiver::parse_published_date("Bad-Month-13-10", reference),
            None
        );
        assert_eq!(
            TelegraphArchiver::parse_published_date("Single-Digit-3-5", reference),
            None
        );
    }

    #[test]
    fn test_convert_to_markdown_omits_author_when_absent() {
        let archiver = TelegraphArchiver::new();
        let page = TelegraphPage {
            path: "No-Author-12-15".to_string(),
            url: "https://telegra.ph/No-Author-12-15".to_string(),
            title: "No Author".to_string(),
            description: String::new(),
            author_name: None,
            author_url: None,
            image_url: None,
            content: None,
            views: 0,
        };

        // No explicit date -> fall back to the path's `-12-15` suffix.
        let markdown = archiver.convert_to_markdown(&page, None).unwrap();

        assert!(!markdown.contains("author:"));
        assert!(markdown.contains("origin: https://telegra.ph/No-Author-12-15\n"));
        // Date derived from the path; year depends on the archive date.
        assert!(markdown.contains("-12-15\n"));
    }

    #[test]
    fn test_extract_datetime_from_html() {
        // Real-world markup: the publish date (with year) lives in the `<time>` attribute.
        let html = r#"<address><time datetime="2017-10-29T12:51:56+0000">October 29, 2017</time></address>"#;
        assert_eq!(
            TelegraphArchiver::extract_datetime_from_html(html),
            Some(NaiveDate::from_ymd_opt(2017, 10, 29).unwrap())
        );

        // No `<time>` element -> None, so callers fall back to the path heuristic.
        assert_eq!(
            TelegraphArchiver::extract_datetime_from_html("<p>no date here</p>"),
            None
        );

        // Malformed datetime value -> None.
        assert_eq!(
            TelegraphArchiver::extract_datetime_from_html("<time datetime=\"not-a-date\">x</time>"),
            None
        );
    }
}
