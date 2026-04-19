use chrono::{DateTime, Utc};
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;

#[derive(Debug, Deserialize)]
pub struct TelegraphResponse {
    pub ok: bool,
    pub result: Option<TelegraphPage>,
    pub error: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct TelegraphPage {
    pub path: String,
    #[allow(dead_code)]
    pub url: String,
    pub title: String,
    #[allow(dead_code)]
    pub description: String,
    pub author_name: Option<String>,
    #[allow(dead_code)]
    pub author_url: Option<String>,
    #[allow(dead_code)]
    pub image_url: Option<String>,
    pub content: Option<Vec<Node>>,
    #[allow(dead_code)]
    pub views: u32,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(untagged)]
pub(crate) enum Node {
    Text(String),
    Element(NodeElement),
}

#[derive(Debug, Deserialize, Clone)]
pub(crate) struct NodeElement {
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

    #[allow(dead_code)]
    pub fn archive_from_json(&self, json: &str) -> Result<String, Box<dyn std::error::Error>> {
        self.archive_from_json_with_log(json, &mut |_| {})
    }

    /// Same as `archive_from_json` but calls `log` with a progress message at each step.
    pub fn archive_from_json_with_log(
        &self,
        json: &str,
        log: &mut dyn FnMut(&str),
    ) -> Result<String, Box<dyn std::error::Error>> {
        log("Parsing Telegraph API response");
        let telegraph_response: TelegraphResponse = serde_json::from_str(json)
            .map_err(|e| format!("Failed to parse Telegraph JSON: {}", e))?;

        if !telegraph_response.ok {
            let err = telegraph_response
                .error
                .as_deref()
                .unwrap_or("unknown error");
            return Err(format!("Telegraph API returned an error: {}", err).into());
        }

        log("Extracting page data");
        let page = telegraph_response
            .result
            .ok_or("No result field in Telegraph response")?;

        log(&format!("Found page: \"{}\"", page.title));

        if let Some(ref author) = page.author_name {
            log(&format!("Author: {}", author));
        }

        if let Some(ref content) = page.content {
            log(&format!("Content nodes to convert: {}", content.len()));
        } else {
            log("No content nodes found in page");
        }

        let filename = self.generate_filename(&page);
        let file_path = format!("content/{}", filename);
        let nonograph_id = filename.trim_end_matches(".md");
        let url = format!("/{}", nonograph_id);

        if std::path::Path::new(&file_path).exists() {
            log(&format!("File already exists: {}", file_path));
            log(&format!(
                "Skipping conversion and write. Already available at {}",
                url
            ));
            return Ok(url);
        }

        log("Converting Telegraph nodes to Markdown");
        let markdown = self.convert_to_markdown(&page)?;

        let line_count = markdown.lines().count();
        let byte_count = markdown.len();
        log(&format!(
            "Markdown conversion complete: {} lines, {} bytes",
            line_count, byte_count
        ));

        log(&format!("Writing to {}", file_path));

        fs::write(&file_path, markdown)
            .map_err(|e| format!("Failed to write file \"{}\": {}", file_path, e))?;

        log(&format!("Saved. Available at {}", url));
        Ok(url)
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
                                output.push_str(&format!("![{}]({})\n\n", caption, full_url));
                                if !caption.is_empty() {
                                    output.push_str(&format!("*{}*\n\n", caption));
                                }
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
                        // First pass: collect caption text from figcaption,
                        // recursing into any inline elements inside it.
                        let mut caption = String::new();
                        if let Some(children) = &element.children {
                            for child in children {
                                if let Node::Element(elem) = child {
                                    if elem.tag == "figcaption" {
                                        Self::collect_text(elem, &mut caption);
                                    }
                                }
                            }
                        }

                        let caption_ref = if caption.is_empty() {
                            None
                        } else {
                            Some(caption.as_str())
                        };

                        // Second pass: render every child that is not a
                        // figcaption, passing the caption as alt-text context.
                        if let Some(children) = &element.children {
                            for child in children {
                                match child {
                                    Node::Element(elem) if elem.tag == "figcaption" => {}
                                    _ => {
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
                    }
                    "figcaption" => {
                        // Figcaptions are handled by the figure element above.
                        // If a bare figcaption appears outside a figure, render
                        // it as italic text so it is not silently dropped.
                        let mut caption = String::new();
                        Self::collect_text(element, &mut caption);
                        if !caption.is_empty() {
                            output.push_str(&format!("*{}*\n\n", caption));
                        }
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

    /// Recursively collect all plain text content from a NodeElement into `out`.
    fn collect_text(elem: &NodeElement, out: &mut String) {
        if let Some(children) = &elem.children {
            for child in children {
                match child {
                    Node::Text(t) => out.push_str(t),
                    Node::Element(e) => Self::collect_text(e, out),
                }
            }
        }
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
        // Use the path from Telegraph as base, but make it filesystem-safe
        let mut filename = page.path.clone();

        // Replace any unsafe characters
        filename = filename.replace('/', "-");
        filename = filename.replace('\\', "-");
        filename = filename.replace(':', "-");
        filename = filename.replace('?', "-");
        filename = filename.replace('*', "-");
        filename = filename.replace('"', "-");
        filename = filename.replace('<', "-");
        filename = filename.replace('>', "-");
        filename = filename.replace('|', "-");

        // Ensure it ends with .md
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
}
