use chrono::{DateTime, Utc};
use ego_tree::NodeRef;
use scraper::{Html, Node as ScraperNode, Selector};
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;

// ─── Telegraph types ──────────────────────────────────────────────────────────

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

// ─── Generic article type (used for arbitrary URLs) ───────────────────────────

struct GenericPage {
    url: String,
    title: String,
    author: Option<String>,
    date: Option<String>,
    content_html: String,
}

// ─── Archiver ─────────────────────────────────────────────────────────────────

pub struct TelegraphArchiver;

impl TelegraphArchiver {
    pub fn new() -> Self {
        Self
    }

    /// Archive any URL.  Telegraph pages use the structured API; everything
    /// else is scraped as HTML and converted to Nonograph markdown.
    pub async fn archive_url(&self, url: &str) -> Result<String, Box<dyn std::error::Error>> {
        let is_telegraph = {
            let parsed = url::Url::parse(url)?;
            parsed.host_str() == Some("telegra.ph")
        };

        if is_telegraph {
            self.archive_telegraph_url(url).await
        } else {
            self.archive_generic_url(url).await
        }
    }

    // ── Telegraph path ────────────────────────────────────────────────────────

    async fn archive_telegraph_url(&self, url: &str) -> Result<String, Box<dyn std::error::Error>> {
        let path = self.extract_path_from_url(url)?;
        let page = self.fetch_telegraph_page(&path).await?;
        let markdown = self.convert_to_markdown(&page)?;
        let filename = self.generate_filename(&page);
        let file_path = format!("content/{}", filename);
        fs::write(&file_path, markdown)?;
        let nonograph_id = filename.trim_end_matches(".md");
        Ok(format!("/{}", nonograph_id))
    }

    fn extract_path_from_url(&self, url: &str) -> Result<String, Box<dyn std::error::Error>> {
        let parsed_url = url::Url::parse(url)?;
        if parsed_url.host_str() != Some("telegra.ph") {
            return Err("URL is not a Telegraph page".into());
        }
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
            .ok_or_else(|| "No result in Telegraph response".into())
    }

    fn convert_to_markdown(
        &self,
        page: &TelegraphPage,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let mut markdown = String::new();
        let now: DateTime<Utc> = Utc::now();
        let archival_date = now.format("%B %d, %Y").to_string();

        if let Some(author) = &page.author_name {
            markdown.push_str(&format!("{} | {}\n\n", archival_date, author));
        } else {
            markdown.push_str(&format!("{}\n\n", archival_date));
        }

        markdown.push_str(&format!("# {}\n", page.title));

        // Source comment (hidden from HTML output)
        markdown.push_str(&format!("// Source: {}\n\n", page.url));

        if let Some(content) = &page.content {
            for node in content {
                self.convert_node_to_markdown(node, &mut markdown, 0)?;
            }
        }

        Ok(self.clean_excessive_newlines(&markdown))
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
                        output.push('\n');
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
                        output.push('*');
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
                        output.push('*');
                    }
                    "u" => {
                        output.push('_');
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
                        output.push('_');
                    }
                    "s" => {
                        output.push('~');
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
                        output.push('~');
                    }
                    "code" => {
                        output.push('`');
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
                        output.push('`');
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
                                output.push('[');
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
                    "h1" => {
                        output.push_str("# ");
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
                    "h2" => {
                        output.push_str("## ");
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
                    "h4" | "h5" | "h6" => {
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
                        output.push('\n');
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
                                        output.push('\n');
                                    }
                                }
                            }
                        }
                        output.push('\n');
                    }
                    "li" => {
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
                            output.push('\n');
                        }
                    }
                    "hr" => {
                        output.push_str("---\n\n");
                    }
                    "figure" => {
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
                        output.push('\n');
                    }
                    "figcaption" => {
                        // handled by parent <figure>
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

    // ── Generic HTML path ─────────────────────────────────────────────────────

    async fn archive_generic_url(&self, url: &str) -> Result<String, Box<dyn std::error::Error>> {
        let page = self.fetch_generic_page(url).await?;
        let markdown = self.convert_generic_to_markdown(&page);
        let filename = self.generate_generic_filename(&page);
        let file_path = format!("content/{}", filename);
        fs::write(&file_path, markdown)?;
        let nonograph_id = filename.trim_end_matches(".md");
        Ok(format!("/{}", nonograph_id))
    }

    async fn fetch_generic_page(
        &self,
        url: &str,
    ) -> Result<GenericPage, Box<dyn std::error::Error>> {
        let client = reqwest::Client::builder()
            .user_agent("Mozilla/5.0 (compatible; nonograph-archiver/1.0)")
            .build()?;

        let response = client.get(url).send().await?;
        if !response.status().is_success() {
            return Err(format!("HTTP {}", response.status()).into());
        }
        let html = response.text().await?;
        self.parse_generic_html(url, &html)
    }

    fn parse_generic_html(
        &self,
        url: &str,
        html: &str,
    ) -> Result<GenericPage, Box<dyn std::error::Error>> {
        let document = Html::parse_document(html);

        // ── Title ─────────────────────────────────────────────────────────────
        // Priority: og:title > <title> > h1
        let title = self
            .select_attr(&document, "meta[property='og:title']", "content")
            .or_else(|| self.select_attr(&document, "meta[name='twitter:title']", "content"))
            .or_else(|| self.select_text(&document, "title"))
            .or_else(|| self.select_text(&document, "h1"))
            .unwrap_or_else(|| "Untitled".to_string());

        let title = title.trim().to_string();

        // ── Author ────────────────────────────────────────────────────────────
        let author = self
            .select_attr(&document, "meta[name='author']", "content")
            .or_else(|| self.select_attr(&document, "meta[property='article:author']", "content"))
            .or_else(|| self.select_text(&document, "[rel='author']"))
            .or_else(|| self.select_text(&document, ".author"))
            .or_else(|| self.select_text(&document, ".byline"))
            .or_else(|| self.select_text(&document, "[itemprop='author']"));

        // ── Date ──────────────────────────────────────────────────────────────
        let date = self
            .select_attr(
                &document,
                "meta[property='article:published_time']",
                "content",
            )
            .or_else(|| self.select_attr(&document, "meta[name='date']", "content"))
            .or_else(|| self.select_attr(&document, "time", "datetime"))
            .or_else(|| self.select_text(&document, "time"))
            .map(|d| self.format_date_string(&d));

        // ── Content ───────────────────────────────────────────────────────────
        // Try common article containers, fall back to <body>
        let content_selectors = [
            "article",
            "[role='main']",
            "main",
            ".post-content",
            ".article-content",
            ".entry-content",
            ".story-body",
            ".postbody",
            "#content",
            ".content",
        ];

        let content_html = content_selectors
            .iter()
            .find_map(|sel| {
                Selector::parse(sel)
                    .ok()
                    .and_then(|s| document.select(&s).next().map(|el| el.html()))
            })
            .or_else(|| {
                Selector::parse("body")
                    .ok()
                    .and_then(|s| document.select(&s).next().map(|el| el.html()))
            })
            .unwrap_or_default();

        Ok(GenericPage {
            url: url.to_string(),
            title,
            author,
            date,
            content_html,
        })
    }

    /// Convert a `GenericPage` to Nonograph-flavoured markdown.
    fn convert_generic_to_markdown(&self, page: &GenericPage) -> String {
        let now: DateTime<Utc> = Utc::now();
        let archival_date = page
            .date
            .clone()
            .unwrap_or_else(|| now.format("%B %d, %Y").to_string());

        let mut md = String::new();

        // Header line
        if let Some(author) = &page.author {
            let author = author.trim();
            if !author.is_empty() {
                md.push_str(&format!("{} | {}\n\n", archival_date, author));
            } else {
                md.push_str(&format!("{}\n\n", archival_date));
            }
        } else {
            md.push_str(&format!("{}\n\n", archival_date));
        }

        // Title
        md.push_str(&format!("# {}\n", page.title));

        // Source comment
        md.push_str(&format!("// Source: {}\n\n", page.url));

        // Body
        let body = self.html_to_markdown(&page.content_html, &page.url);
        md.push_str(&body);

        self.clean_excessive_newlines(&md)
    }

    /// Walk an HTML fragment and emit Nonograph markdown.
    fn html_to_markdown(&self, html: &str, base_url: &str) -> String {
        let fragment = Html::parse_fragment(html);
        let mut out = String::new();
        // scraper gives us an implicit root element; iterate its children
        for node in fragment.root_element().children() {
            self.walk_node(node, &mut out, base_url, 0);
        }
        self.clean_excessive_newlines(&out)
    }

    fn walk_node(
        &self,
        node: NodeRef<ScraperNode>,
        out: &mut String,
        base_url: &str,
        depth: usize,
    ) {
        match node.value() {
            ScraperNode::Text(text) => {
                let t = text.trim();
                if !t.is_empty() {
                    out.push_str(t);
                }
            }
            ScraperNode::Element(el) => {
                let tag = el.name();

                // Skip non-content elements entirely
                match tag {
                    "script" | "style" | "noscript" | "nav" | "header" | "footer" | "aside"
                    | "form" | "button" | "input" | "select" | "textarea" | "label" | "svg"
                    | "canvas" => return,
                    _ => {}
                }

                match tag {
                    // ── Block elements ────────────────────────────────────────
                    "p" | "div" | "section" | "article" | "main" => {
                        let inner = self.walk_children_to_string(node, out, base_url, depth);
                        if !inner.trim().is_empty() {
                            out.push_str(inner.trim());
                            out.push_str("\n\n");
                        }
                    }
                    "h1" => {
                        let inner = self.children_text(node, base_url, depth);
                        if !inner.trim().is_empty() {
                            out.push_str(&format!("# {}\n\n", inner.trim()));
                        }
                    }
                    "h2" => {
                        let inner = self.children_text(node, base_url, depth);
                        if !inner.trim().is_empty() {
                            out.push_str(&format!("## {}\n\n", inner.trim()));
                        }
                    }
                    "h3" => {
                        let inner = self.children_text(node, base_url, depth);
                        if !inner.trim().is_empty() {
                            out.push_str(&format!("### {}\n\n", inner.trim()));
                        }
                    }
                    "h4" | "h5" | "h6" => {
                        let inner = self.children_text(node, base_url, depth);
                        if !inner.trim().is_empty() {
                            out.push_str(&format!("#### {}\n\n", inner.trim()));
                        }
                    }
                    "blockquote" => {
                        let inner = self.children_text(node, base_url, depth);
                        for line in inner.trim().lines() {
                            out.push_str(&format!("> {}\n", line));
                        }
                        out.push('\n');
                    }
                    "pre" => {
                        let inner = self.children_text(node, base_url, depth);
                        // Try to detect language from a class like "language-rust"
                        let lang = el
                            .attr("class")
                            .and_then(|c| {
                                c.split_whitespace().find_map(|cls| {
                                    cls.strip_prefix("language-")
                                        .or_else(|| cls.strip_prefix("lang-"))
                                })
                            })
                            .unwrap_or("");
                        out.push_str(&format!("```{}\n{}\n```\n\n", lang, inner.trim()));
                    }
                    "hr" => {
                        out.push_str("---\n\n");
                    }
                    "br" => {
                        out.push('\n');
                    }
                    "ul" => {
                        for child in node.children() {
                            if let ScraperNode::Element(li_el) = child.value() {
                                if li_el.name() == "li" {
                                    let inner = self.children_text(child, base_url, depth + 1);
                                    out.push_str(&format!("- {}\n", inner.trim()));
                                }
                            }
                        }
                        out.push('\n');
                    }
                    "ol" => {
                        let mut idx = 1usize;
                        for child in node.children() {
                            if let ScraperNode::Element(li_el) = child.value() {
                                if li_el.name() == "li" {
                                    let inner = self.children_text(child, base_url, depth + 1);
                                    out.push_str(&format!("{}. {}\n", idx, inner.trim()));
                                    idx += 1;
                                }
                            }
                        }
                        out.push('\n');
                    }
                    "li" => {
                        // Standalone <li> outside of ul/ol
                        let inner = self.children_text(node, base_url, depth);
                        out.push_str(&format!("- {}\n", inner.trim()));
                    }
                    "figure" => {
                        // Look for img/video + figcaption
                        let mut img_md = String::new();
                        let mut caption = String::new();
                        for child in node.children() {
                            if let ScraperNode::Element(cel) = child.value() {
                                match cel.name() {
                                    "figcaption" => {
                                        caption = self.children_text(child, base_url, depth);
                                    }
                                    "img" => {
                                        if let Some(src) = cel.attr("src") {
                                            let src = self.resolve_url(src, base_url);
                                            img_md = format!("![placeholder]({})", src);
                                        }
                                    }
                                    "video" => {
                                        if let Some(src) = cel.attr("src") {
                                            let src = self.resolve_url(src, base_url);
                                            img_md = format!("![placeholder]({})", src);
                                        }
                                    }
                                    _ => {}
                                }
                            }
                        }
                        if !img_md.is_empty() {
                            // Replace placeholder with actual caption
                            let final_md = img_md.replace("placeholder", caption.trim());
                            out.push_str(&final_md);
                            out.push_str("\n\n");
                        }
                    }

                    // ── Inline elements ───────────────────────────────────────
                    "strong" | "b" => {
                        let inner = self.children_text(node, base_url, depth);
                        if !inner.trim().is_empty() {
                            out.push_str(&format!("**{}**", inner.trim()));
                        }
                    }
                    "em" | "i" => {
                        let inner = self.children_text(node, base_url, depth);
                        if !inner.trim().is_empty() {
                            out.push_str(&format!("*{}*", inner.trim()));
                        }
                    }
                    "u" => {
                        let inner = self.children_text(node, base_url, depth);
                        if !inner.trim().is_empty() {
                            out.push_str(&format!("_{}_", inner.trim()));
                        }
                    }
                    "s" | "del" | "strike" => {
                        let inner = self.children_text(node, base_url, depth);
                        if !inner.trim().is_empty() {
                            out.push_str(&format!("~{}~", inner.trim()));
                        }
                    }
                    "code" => {
                        let inner = self.children_text(node, base_url, depth);
                        if !inner.trim().is_empty() {
                            out.push_str(&format!("`{}`", inner.trim()));
                        }
                    }
                    "mark" => {
                        let inner = self.children_text(node, base_url, depth);
                        if !inner.trim().is_empty() {
                            out.push_str(&format!("=={} ==", inner.trim()));
                        }
                    }
                    "sup" => {
                        let inner = self.children_text(node, base_url, depth);
                        if !inner.trim().is_empty() {
                            out.push_str(&format!("^{}^", inner.trim()));
                        }
                    }
                    "a" => {
                        let href = el
                            .attr("href")
                            .map(|h| self.resolve_url(h, base_url))
                            .unwrap_or_default();
                        let inner = self.children_text(node, base_url, depth);
                        let text = inner.trim();
                        if href.is_empty() {
                            out.push_str(text);
                        } else if text.is_empty() || text == href {
                            out.push_str(&format!("[{}]", href));
                        } else {
                            out.push_str(&format!("[{}]({})", text, href));
                        }
                    }
                    "img" => {
                        let src = el
                            .attr("src")
                            .map(|s| self.resolve_url(s, base_url))
                            .unwrap_or_default();
                        if !src.is_empty() {
                            let alt = el.attr("alt").unwrap_or("");
                            out.push_str(&format!("![{}]({})", alt, src));
                        }
                    }
                    "video" => {
                        let src = el
                            .attr("src")
                            .map(|s| self.resolve_url(s, base_url))
                            .unwrap_or_default();
                        if !src.is_empty() {
                            out.push_str(&format!("![Video]({})\n\n", src));
                        } else {
                            // Try <source> children
                            for child in node.children() {
                                if let ScraperNode::Element(cel) = child.value() {
                                    if cel.name() == "source" {
                                        if let Some(s) = cel.attr("src") {
                                            let s = self.resolve_url(s, base_url);
                                            out.push_str(&format!("![Video]({})\n\n", s));
                                            break;
                                        }
                                    }
                                }
                            }
                        }
                    }
                    "iframe" => {
                        if let Some(src) = el.attr("src") {
                            let src = self.resolve_url(src, base_url);
                            // Only keep YouTube embeds etc. as links
                            if !src.is_empty() {
                                out.push_str(&format!("[{}]({})\n\n", src, src));
                            }
                        }
                    }
                    "table" => {
                        self.table_to_markdown(node, out, base_url, depth);
                    }
                    _ => {
                        // Pass-through: just process children
                        for child in node.children() {
                            self.walk_node(child, out, base_url, depth);
                        }
                    }
                }
            }
            _ => {}
        }
    }

    /// Walk children, accumulating inline output into a new String and also
    /// pushing block-level output directly to `out`.  Returns inline text.
    fn walk_children_to_string(
        &self,
        node: NodeRef<ScraperNode>,
        out: &mut String,
        base_url: &str,
        depth: usize,
    ) -> String {
        let mut inner = String::new();
        for child in node.children() {
            self.walk_node(child, &mut inner, base_url, depth);
        }
        // Flush any block-level content that ended up in inner to out
        // (walk_node already pushes directly; this just collects inline text)
        let _ = out; // used in some code paths above; keep signature symmetric
        inner
    }

    /// Collect all text content under a node (inline only, no block wrappers).
    fn children_text(&self, node: NodeRef<ScraperNode>, base_url: &str, depth: usize) -> String {
        let mut s = String::new();
        for child in node.children() {
            self.walk_node(child, &mut s, base_url, depth);
        }
        s
    }

    /// Rudimentary table → Nonograph markdown table converter.
    fn table_to_markdown(
        &self,
        node: NodeRef<ScraperNode>,
        out: &mut String,
        base_url: &str,
        depth: usize,
    ) {
        let mut rows: Vec<Vec<String>> = Vec::new();
        let mut is_header_row: Vec<bool> = Vec::new();

        for child in node.descendants() {
            if let ScraperNode::Element(el) = child.value() {
                match el.name() {
                    "tr" => {
                        let mut cells: Vec<String> = Vec::new();
                        let mut has_th = false;
                        for td in child.children() {
                            if let ScraperNode::Element(cel) = td.value() {
                                match cel.name() {
                                    "th" => {
                                        has_th = true;
                                        cells.push(
                                            self.children_text(td, base_url, depth)
                                                .trim()
                                                .to_string(),
                                        );
                                    }
                                    "td" => {
                                        cells.push(
                                            self.children_text(td, base_url, depth)
                                                .trim()
                                                .to_string(),
                                        );
                                    }
                                    _ => {}
                                }
                            }
                        }
                        if !cells.is_empty() {
                            is_header_row.push(has_th);
                            rows.push(cells);
                        }
                    }
                    _ => {}
                }
            }
        }

        if rows.is_empty() {
            return;
        }

        // Determine column count from widest row
        let col_count = rows.iter().map(|r| r.len()).max().unwrap_or(0);

        for (i, row) in rows.iter().enumerate() {
            let padded: Vec<String> = (0..col_count)
                .map(|j| row.get(j).cloned().unwrap_or_default())
                .collect();
            out.push_str(&format!("| {} |\n", padded.join(" | ")));
            // Insert separator after the first row (header)
            if i == 0 {
                let sep: Vec<&str> = (0..col_count).map(|_| "---").collect();
                out.push_str(&format!("|{}|\n", sep.join("|")));
            }
        }
        out.push('\n');
    }

    // ── URL helpers ───────────────────────────────────────────────────────────

    /// Resolve a potentially relative URL against the page's base URL.
    fn resolve_url(&self, href: &str, base_url: &str) -> String {
        if href.starts_with("http://") || href.starts_with("https://") || href.starts_with("//") {
            return href.to_string();
        }
        if let Ok(base) = url::Url::parse(base_url) {
            if let Ok(resolved) = base.join(href) {
                return resolved.to_string();
            }
        }
        href.to_string()
    }

    // ── CSS selector helpers ──────────────────────────────────────────────────

    fn select_text(&self, document: &Html, selector: &str) -> Option<String> {
        let sel = Selector::parse(selector).ok()?;
        document
            .select(&sel)
            .next()
            .map(|el| el.text().collect::<String>().trim().to_string())
            .filter(|s| !s.is_empty())
    }

    fn select_attr(&self, document: &Html, selector: &str, attr: &str) -> Option<String> {
        let sel = Selector::parse(selector).ok()?;
        document
            .select(&sel)
            .next()
            .and_then(|el| el.value().attr(attr))
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
    }

    // ── Date normalisation ────────────────────────────────────────────────────

    /// Try to turn an ISO-8601 or other date string into "Month DD, YYYY".
    fn format_date_string(&self, raw: &str) -> String {
        let trimmed = raw.trim();
        // Try full ISO datetime first
        if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(trimmed) {
            return dt.format("%B %d, %Y").to_string();
        }
        // Try date-only YYYY-MM-DD
        if let Ok(d) = chrono::NaiveDate::parse_from_str(trimmed, "%Y-%m-%d") {
            return d.format("%B %d, %Y").to_string();
        }
        // Return as-is if we can't parse it
        trimmed.to_string()
    }

    // ── Filename generation ───────────────────────────────────────────────────

    fn generate_filename(&self, page: &TelegraphPage) -> String {
        let mut filename = page.path.clone();
        for ch in &['/', '\\', ':', '?', '*', '"', '<', '>', '|'] {
            filename = filename.replace(*ch, "-");
        }
        if !filename.ends_with(".md") {
            filename.push_str(".md");
        }
        filename
    }

    /// Derive a filesystem-safe filename from a generic page.
    fn generate_generic_filename(&self, page: &GenericPage) -> String {
        let now: DateTime<Utc> = Utc::now();
        let date_suffix = now.format("%m-%d-%Y").to_string();

        // Slugify the title
        let slug: String = page
            .title
            .to_lowercase()
            .chars()
            .map(|c| if c.is_alphanumeric() { c } else { '-' })
            .collect::<String>()
            // Collapse runs of dashes
            .split('-')
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join("-");

        // Truncate slug so the total filename stays reasonable
        let slug = if slug.len() > 60 {
            slug[..60].trim_end_matches('-').to_string()
        } else {
            slug
        };

        format!("{}-{}.md", slug, date_suffix)
    }

    // ── Shared utilities ──────────────────────────────────────────────────────

    fn clean_excessive_newlines(&self, content: &str) -> String {
        let mut result = content.to_string();
        while result.contains("\n\n\n") {
            result = result.replace("\n\n\n", "\n\n");
        }
        result
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

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
    fn test_is_telegraph_url_routed_correctly() {
        // Just confirm the routing logic — no network needed
        let telegraph = "https://telegra.ph/My-Post-01-01";
        let generic = "https://www.example.com/article/foo";

        let t_parsed = url::Url::parse(telegraph).unwrap();
        let g_parsed = url::Url::parse(generic).unwrap();

        assert_eq!(t_parsed.host_str(), Some("telegra.ph"));
        assert_ne!(g_parsed.host_str(), Some("telegra.ph"));
    }

    #[test]
    fn test_generate_generic_filename_slug() {
        let archiver = TelegraphArchiver::new();
        let page = GenericPage {
            url: "https://example.com/test".to_string(),
            title: "Hello World! This Is A Test".to_string(),
            author: None,
            date: None,
            content_html: String::new(),
        };
        let filename = archiver.generate_generic_filename(&page);
        // Should start with the slugified title
        assert!(filename.starts_with("hello-world-this-is-a-test-"));
        assert!(filename.ends_with(".md"));
    }

    #[test]
    fn test_resolve_url_absolute() {
        let archiver = TelegraphArchiver::new();
        let result = archiver.resolve_url(
            "https://cdn.example.com/img.png",
            "https://example.com/page",
        );
        assert_eq!(result, "https://cdn.example.com/img.png");
    }

    #[test]
    fn test_resolve_url_relative() {
        let archiver = TelegraphArchiver::new();
        let result = archiver.resolve_url("/images/photo.jpg", "https://example.com/article/1");
        assert_eq!(result, "https://example.com/images/photo.jpg");
    }

    #[test]
    fn test_format_date_string_iso() {
        let archiver = TelegraphArchiver::new();
        let result = archiver.format_date_string("2026-04-30T17:59:01+00:00");
        assert_eq!(result, "April 30, 2026");
    }

    #[test]
    fn test_format_date_string_date_only() {
        let archiver = TelegraphArchiver::new();
        let result = archiver.format_date_string("2026-04-30");
        assert_eq!(result, "April 30, 2026");
    }

    #[test]
    fn test_html_to_markdown_basic() {
        let archiver = TelegraphArchiver::new();
        let html = "<p>Hello <strong>world</strong>!</p>";
        let md = archiver.html_to_markdown(html, "https://example.com");
        assert!(md.contains("**world**"));
        assert!(md.contains("Hello"));
    }

    #[test]
    fn test_html_to_markdown_headings() {
        let archiver = TelegraphArchiver::new();
        let html = "<h2>Section Title</h2><p>Some text.</p>";
        let md = archiver.html_to_markdown(html, "https://example.com");
        assert!(md.contains("## Section Title"));
        assert!(md.contains("Some text."));
    }

    #[test]
    fn test_html_to_markdown_links() {
        let archiver = TelegraphArchiver::new();
        let html = r#"<p><a href="https://example.com">Click here</a></p>"#;
        let md = archiver.html_to_markdown(html, "https://example.com");
        assert!(md.contains("[Click here](https://example.com)"));
    }

    #[test]
    fn test_html_to_markdown_blockquote() {
        let archiver = TelegraphArchiver::new();
        let html = "<blockquote>Quoted text here.</blockquote>";
        let md = archiver.html_to_markdown(html, "https://example.com");
        assert!(md.contains("> Quoted text here."));
    }

    #[test]
    fn test_html_to_markdown_lists() {
        let archiver = TelegraphArchiver::new();
        let html = "<ul><li>Item one</li><li>Item two</li></ul>";
        let md = archiver.html_to_markdown(html, "https://example.com");
        assert!(md.contains("- Item one"));
        assert!(md.contains("- Item two"));
    }

    #[test]
    fn test_parse_generic_html_title() {
        let archiver = TelegraphArchiver::new();
        let html = r#"<html><head>
            <meta property="og:title" content="OG Title">
            <title>Page Title</title>
        </head><body><p>Content</p></body></html>"#;
        let page = archiver
            .parse_generic_html("https://example.com", html)
            .unwrap();
        assert_eq!(page.title, "OG Title");
    }

    #[test]
    fn test_parse_generic_html_author() {
        let archiver = TelegraphArchiver::new();
        let html = r#"<html><head></head><body>
            <meta name="author" content="Jane Doe">
            <p>Content</p>
        </body></html>"#;
        let page = archiver
            .parse_generic_html("https://example.com", html)
            .unwrap();
        assert_eq!(page.author.as_deref(), Some("Jane Doe"));
    }

    #[test]
    fn test_parse_generic_html_date_iso() {
        let archiver = TelegraphArchiver::new();
        let html = r#"<html><head>
            <meta property="article:published_time" content="2026-04-30T12:00:00+00:00">
        </head><body><p>Content</p></body></html>"#;
        let page = archiver
            .parse_generic_html("https://example.com", html)
            .unwrap();
        assert_eq!(page.date.as_deref(), Some("April 30, 2026"));
    }
}
