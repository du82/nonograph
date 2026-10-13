//! Website archiver.
//!
//! Converts an arbitrary web page into Nonograph markup. The design is
//! modular: each supported kind of site is handled by a [`SiteAdapter`]. An
//! adapter is responsible for pulling the meaningful pieces out of a fetched
//! page (title, author, body) so the rest of the pipeline can turn them into
//! Nonograph markup.
//!
//! This is the first, intentionally-basic iteration. It defines the core
//! types, a generic fallback adapter, and the fetch/convert pipeline used by
//! the `/save/` endpoint. More specialized adapters (Telegraph, Medium, etc.)
//! can be added later by implementing [`SiteAdapter`].

use chrono::Utc;

/// The meaningful parts extracted from a web page, independent of the source
/// site's markup.
#[derive(Debug, Clone, Default)]
pub struct ExtractedPage {
    pub title: String,
    pub author: Option<String>,
    /// The page body as Nonograph markdown (without frontmatter).
    pub body: String,
}

/// Errors that can occur while archiving a page.
#[derive(Debug)]
pub enum ArchiveError {
    InvalidUrl(String),
    Fetch(String),
    Unsupported(String),
    /// Used by adapters that parse a page body (not yet exercised by the
    /// generic adapter, but part of the public API).
    #[allow(dead_code)]
    Extract(String),
}

impl std::fmt::Display for ArchiveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ArchiveError::InvalidUrl(m) => write!(f, "invalid URL: {}", m),
            ArchiveError::Fetch(m) => write!(f, "failed to fetch page: {}", m),
            ArchiveError::Unsupported(m) => write!(f, "unsupported site: {}", m),
            ArchiveError::Extract(m) => write!(f, "failed to extract content: {}", m),
        }
    }
}

impl std::error::Error for ArchiveError {}

/// A fetched web page handed to adapters for extraction.
pub struct FetchedPage {
    pub url: url::Url,
    pub html: String,
}

/// A modular handler for a particular kind of site.
///
/// Implementors decide whether they can handle a given URL and, if so, how to
/// turn the fetched HTML into an [`ExtractedPage`].
pub trait SiteAdapter: Send + Sync {
    /// Human-readable name, used for logging/diagnostics.
    #[allow(dead_code)]
    fn name(&self) -> &'static str;

    /// Whether this adapter wants to handle the given URL.
    fn matches(&self, url: &url::Url) -> bool;

    /// Extract the meaningful content from the fetched page.
    fn extract(&self, page: &FetchedPage) -> Result<ExtractedPage, ArchiveError>;
}

/// A last-resort adapter that works on any site using generic heuristics
/// (`<title>`, `<meta>` author tags, and a naive body extraction).
pub struct GenericAdapter;

impl SiteAdapter for GenericAdapter {
    fn name(&self) -> &'static str {
        "generic"
    }

    fn matches(&self, _url: &url::Url) -> bool {
        // The generic adapter is the catch-all fallback.
        true
    }

    fn extract(&self, page: &FetchedPage) -> Result<ExtractedPage, ArchiveError> {
        let body_html = extract_body_region(&page.html);

        // Prefer, in order: the article's own <h1>, the OpenGraph title, then
        // the <title> tag (with any site-name suffix stripped). This keeps the
        // title field to just the article title.
        let title = extract_first_heading(body_html)
            .or_else(|| extract_meta_content(&page.html, "property", "og:title"))
            .or_else(|| extract_title(&page.html).map(|t| clean_title(&t)))
            .filter(|t| !t.is_empty())
            .unwrap_or_else(|| page.url.host_str().unwrap_or("Untitled").to_string());

        let author = extract_author(&page.html);

        let mut body = html_to_markup(body_html);
        // Avoid repeating the title as the first line of the body.
        body = strip_leading_title(&body, &title);

        Ok(ExtractedPage {
            title,
            author,
            body,
        })
    }
}

/// The archiver orchestrates fetching a URL, selecting an adapter, extracting
/// content, and rendering Nonograph markup.
pub struct Archiver {
    adapters: Vec<Box<dyn SiteAdapter>>,
    client: reqwest::Client,
}

impl Archiver {
    /// Create an archiver with the default set of adapters.
    pub fn new() -> Self {
        Self {
            // Order matters: more specific adapters should come before the
            // generic fallback once they exist.
            adapters: vec![Box::new(GenericAdapter)],
            client: reqwest::Client::new(),
        }
    }

    /// Archive a URL, returning the rendered Nonograph markup (with
    /// frontmatter). Saving/persistence is handled by the caller.
    pub async fn archive(&self, raw_url: &str) -> Result<String, ArchiveError> {
        let extracted = self.extract(raw_url).await?;
        Ok(self.render_markup(&extracted))
    }

    /// Fetch a URL and return the structured content (title, author, body)
    /// without rendering frontmatter. Callers that persist the page
    /// themselves (e.g. the `/save` endpoint) use this.
    pub async fn extract(&self, raw_url: &str) -> Result<ExtractedPage, ArchiveError> {
        let url = url::Url::parse(raw_url).map_err(|e| ArchiveError::InvalidUrl(e.to_string()))?;

        if !matches!(url.scheme(), "http" | "https") {
            return Err(ArchiveError::InvalidUrl(format!(
                "unsupported scheme: {}",
                url.scheme()
            )));
        }

        let page = self.fetch(url).await?;
        let adapter = self.select_adapter(&page.url)?;
        adapter.extract(&page)
    }

    /// Fetch the raw HTML for a URL.
    async fn fetch(&self, url: url::Url) -> Result<FetchedPage, ArchiveError> {
        let response = self
            .client
            .get(url.clone())
            .send()
            .await
            .map_err(|e| ArchiveError::Fetch(e.to_string()))?;

        if !response.status().is_success() {
            return Err(ArchiveError::Fetch(format!(
                "server returned status {}",
                response.status()
            )));
        }

        let html = response
            .text()
            .await
            .map_err(|e| ArchiveError::Fetch(e.to_string()))?;

        Ok(FetchedPage { url, html })
    }

    /// Pick the first adapter that matches the URL.
    fn select_adapter(&self, url: &url::Url) -> Result<&dyn SiteAdapter, ArchiveError> {
        self.adapters
            .iter()
            .find(|a| a.matches(url))
            .map(|a| a.as_ref())
            .ok_or_else(|| {
                ArchiveError::Unsupported(url.host_str().unwrap_or("unknown").to_string())
            })
    }

    /// Render an [`ExtractedPage`] into Nonograph markup with frontmatter.
    fn render_markup(&self, page: &ExtractedPage) -> String {
        let mut out = String::new();
        out.push_str("---\n");
        out.push_str(&format!("title: {}\n", page.title));
        out.push_str(&format!("date: {}\n", Utc::now().format("%Y-%m-%d")));
        if let Some(author) = &page.author {
            if !author.is_empty() {
                out.push_str(&format!("author: {}\n", author));
            }
        }
        out.push_str(&format!(
            "generator: {} v{}\n",
            env!("CARGO_PKG_NAME"),
            env!("CARGO_PKG_VERSION")
        ));
        out.push_str("---\n\n");
        out.push_str(&page.body);
        out
    }
}

impl Default for Archiver {
    fn default() -> Self {
        Self::new()
    }
}

/// Extract the contents of the first `<title>` tag.
fn extract_title(html: &str) -> Option<String> {
    let lower = html.to_lowercase();
    let start = lower.find("<title")?;
    let gt = lower[start..].find('>')? + start + 1;
    let end = lower[gt..].find("</title>")? + gt;
    let title = html[gt..end].trim();
    if title.is_empty() {
        None
    } else {
        Some(decode_basic_entities(title))
    }
}

/// Extract an author from common `<meta>` tags.
fn extract_author(html: &str) -> Option<String> {
    // (attribute, expected value) pairs that identify an author meta tag.
    for (attr, value) in [("name", "author"), ("property", "article:author")] {
        if let Some(found) = extract_meta_content(html, attr, value) {
            if !found.is_empty() {
                return Some(found);
            }
        }
    }
    None
}

/// Find the `content` of the first `<meta>` tag whose `attr` equals `value`.
fn extract_meta_content(html: &str, attr: &str, value: &str) -> Option<String> {
    let lower = html.to_lowercase();
    let mut search_from = 0;
    while let Some(meta_rel) = lower[search_from..].find("<meta") {
        let meta_start = search_from + meta_rel;
        let tag_end = lower[meta_start..].find('>').map(|i| meta_start + i)?;
        let tag = &html[meta_start..tag_end];

        let attr_matches = extract_attr(tag, attr)
            .map(|v| v.eq_ignore_ascii_case(value))
            .unwrap_or(false);

        if attr_matches {
            if let Some(content) = extract_attr(tag, "content") {
                return Some(decode_basic_entities(content.trim()));
            }
        }
        search_from = tag_end + 1;
    }
    None
}

/// Extract an attribute value (handles both single and double quotes).
fn extract_attr(tag: &str, attr: &str) -> Option<String> {
    let lower = tag.to_lowercase();
    let attr_pos = lower.find(&format!("{}=", attr))?;
    let after = &tag[attr_pos + attr.len() + 1..];
    let quote = after.chars().next()?;
    if quote == '"' || quote == '\'' {
        let rest = &after[1..];
        let end = rest.find(quote)?;
        Some(rest[..end].to_string())
    } else {
        None
    }
}

/// Decode a small set of common HTML entities.
fn decode_basic_entities(s: &str) -> String {
    s.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&apos;", "'")
        .replace("&nbsp;", " ")
}

/// Narrow the HTML down to the region most likely to contain the body.
///
/// Prefers `<article>`, then `<main>`, then `<body>`, falling back to the
/// whole document. This is a heuristic shared by the generic adapter.
fn extract_body_region(html: &str) -> &str {
    for (open, close) in [
        ("<article", "</article>"),
        ("<main", "</main>"),
        ("<body", "</body>"),
    ] {
        if let Some(region) = extract_between_tag(html, open, close) {
            return region;
        }
    }
    html
}

/// Return the inner HTML of the first `open`..`close` tag pair, if present.
fn extract_between_tag<'a>(html: &'a str, open: &str, close: &str) -> Option<&'a str> {
    let lower = html.to_lowercase();
    let open_start = lower.find(open)?;
    // Skip to the end of the opening tag.
    let gt = lower[open_start..].find('>')? + open_start + 1;
    let close_rel = lower[gt..].find(close)?;
    Some(&html[gt..gt + close_rel])
}

/// Extract the text of the first `<h1>` (falling back to `<h2>`) in the HTML,
/// stripped of inner tags. Used to get the article's own title.
fn extract_first_heading(html: &str) -> Option<String> {
    for (open, close) in [("<h1", "</h1>"), ("<h2", "</h2>")] {
        if let Some(inner) = extract_between_tag(html, open, close) {
            let text = strip_all_tags(inner);
            let text = decode_basic_entities(&text);
            let collapsed = text.split_whitespace().collect::<Vec<_>>().join(" ");
            if !collapsed.is_empty() {
                return Some(collapsed);
            }
        }
    }
    None
}

/// Strip all HTML tags from a fragment, keeping only text.
fn strip_all_tags(html: &str) -> String {
    let mut out = String::new();
    let mut depth = 0usize;
    for c in html.chars() {
        match c {
            '<' => depth += 1,
            '>' => depth = depth.saturating_sub(1),
            _ if depth == 0 => out.push(c),
            _ => {}
        }
    }
    out
}

/// Clean a `<title>` tag value by removing a trailing site-name suffix such as
/// `"Article — Site Name"` or `"Article | Site Name"`.
fn clean_title(title: &str) -> String {
    let title = title.trim();
    // Common separators, longest first so we match " — " before "-".
    for sep in [" — ", " – ", " | ", " · ", " :: ", " - "] {
        if let Some(pos) = title.find(sep) {
            let candidate = title[..pos].trim();
            if !candidate.is_empty() {
                return candidate.to_string();
            }
        }
    }
    title.to_string()
}

/// Remove a leading heading/line from the body if it merely repeats the title.
fn strip_leading_title(body: &str, title: &str) -> String {
    let title_norm = normalize_for_compare(title);
    if title_norm.is_empty() {
        return body.to_string();
    }

    // Inspect the first non-empty line.
    let mut lines = body.lines();
    let mut skipped = String::new();
    for line in lines.by_ref() {
        if line.trim().is_empty() {
            skipped.push('\n');
            continue;
        }
        // Strip a leading markup heading marker for comparison.
        let content = line.trim_start_matches('#').trim();
        if normalize_for_compare(content) == title_norm {
            // Drop this line; return the remainder trimmed.
            return lines
                .collect::<Vec<_>>()
                .join("\n")
                .trim_start()
                .to_string();
        }
        break;
    }
    body.to_string()
}

/// Lowercased, whitespace-collapsed form for loose title comparison.
fn normalize_for_compare(s: &str) -> String {
    s.to_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// Convert a subset of HTML into Nonograph markup.
///
/// This is deliberately lightweight (no DOM): it walks the HTML, emitting
/// Nonograph markup for the block and inline elements Nonograph supports
/// (headings, paragraphs, lists, blockquotes, links, bold/italic, code).
/// Unknown tags are dropped but their text content is kept.
fn html_to_markup(html: &str) -> String {
    // Remove entire non-content elements, including their contents.
    let stripped = strip_elements(html, &["script", "style", "noscript", "svg", "head"]);

    let mut out = String::new();
    let bytes = stripped.as_bytes();
    let mut i = 0;
    // Tracks inline wrappers we've opened so we can emit matching markers.
    while i < bytes.len() {
        if bytes[i] == b'<' {
            if let Some(close_rel) = stripped[i..].find('>') {
                let tag = &stripped[i..i + close_rel + 1];
                emit_tag_markup(tag, &mut out);
                i += close_rel + 1;
                continue;
            }
        }
        // Text node: accumulate until the next '<'.
        let next = stripped[i..]
            .find('<')
            .map(|r| i + r)
            .unwrap_or(bytes.len());
        let text = &stripped[i..next];
        let decoded = decode_basic_entities(text);
        let collapsed = collapse_ws(&decoded);
        if !collapsed.is_empty() {
            out.push_str(&collapsed);
        }
        i = next;
    }

    normalize_blank_lines(&out)
}

/// Emit Nonograph markup corresponding to a single HTML tag.
fn emit_tag_markup(tag: &str, out: &mut String) {
    let name = tag_name(tag);
    match name.as_str() {
        "h1" => out.push_str("\n\n# "),
        "h2" => out.push_str("\n\n## "),
        "h3" => out.push_str("\n\n### "),
        "h4" | "h5" | "h6" => out.push_str("\n\n#### "),
        "/h1" | "/h2" | "/h3" | "/h4" | "/h5" | "/h6" => out.push_str("\n\n"),
        "p" | "/p" | "div" | "/div" | "section" | "/section" => out.push_str("\n\n"),
        "br" => out.push('\n'),
        "li" => out.push_str("\n- "),
        "/li" => {}
        "ul" | "/ul" | "ol" | "/ol" => out.push_str("\n\n"),
        "blockquote" => out.push_str("\n\n> "),
        "/blockquote" => out.push_str("\n\n"),
        "strong" | "b" | "/strong" | "/b" => out.push_str("**"),
        "em" | "i" | "/em" | "/i" => out.push('*'),
        "code" | "/code" => out.push('`'),
        "a" => {
            if let Some(href) = extract_attr(tag, "href") {
                // Open a Nonograph link: [text](href) — we buffer the opening
                // and rely on the closing tag to append the destination. To
                // keep this single-pass, we emit "[" here and store href via a
                // sentinel the closing tag reads back from `out`.
                out.push('[');
                out.push_str(&format!("\u{0}{}\u{0}", href));
            } else {
                out.push('[');
                out.push_str("\u{0}\u{0}");
            }
        }
        "/a" => finish_link(out),
        _ => {}
    }
}

/// Close a link opened by [`emit_tag_markup`], turning the buffered
/// `[\0href\0text` into `[text](href)`.
fn finish_link(out: &mut String) {
    // Find the last unclosed link marker.
    if let Some(bracket) = out.rfind('[') {
        let after = &out[bracket + 1..];
        if let Some(first_nul) = after.find('\u{0}') {
            let rest = &after[first_nul + 1..];
            if let Some(second_nul) = rest.find('\u{0}') {
                let href = rest[..second_nul].to_string();
                let text = rest[second_nul + 1..].trim().to_string();
                let replacement = if href.is_empty() {
                    text
                } else if text.is_empty() {
                    format!("[{}]({})", href, href)
                } else {
                    format!("[{}]({})", text, href)
                };
                out.replace_range(bracket.., &replacement);
            }
        }
    }
}

/// Return the (lowercased) tag name, e.g. `h1`, `/p`, `a`.
fn tag_name(tag: &str) -> String {
    let inner = tag.trim_start_matches('<').trim_end_matches('>').trim();
    let mut name = String::new();
    for c in inner.chars() {
        if c.is_whitespace() {
            break;
        }
        name.push(c.to_ascii_lowercase());
    }
    name
}

/// Remove the given elements and their contents entirely.
fn strip_elements(html: &str, names: &[&str]) -> String {
    let mut result = html.to_string();
    for name in names {
        let open = format!("<{}", name);
        let close = format!("</{}>", name);
        loop {
            let lower = result.to_lowercase();
            let Some(start) = lower.find(&open) else {
                break;
            };
            match lower[start..].find(&close) {
                Some(rel) => {
                    let end = start + rel + close.len();
                    result.replace_range(start..end, "");
                }
                None => {
                    result.replace_range(start.., "");
                    break;
                }
            }
        }
    }
    result
}

/// Collapse runs of whitespace within a text node to single spaces, preserving
/// a single leading/trailing space if present (so words don't glue together).
fn collapse_ws(s: &str) -> String {
    let leading = s.starts_with(char::is_whitespace);
    let trailing = s.ends_with(char::is_whitespace);
    let core = s.split_whitespace().collect::<Vec<_>>().join(" ");
    if core.is_empty() {
        return String::new();
    }
    let mut out = String::new();
    if leading {
        out.push(' ');
    }
    out.push_str(&core);
    if trailing {
        out.push(' ');
    }
    out
}

/// Collapse 3+ consecutive newlines to 2 and trim the result.
fn normalize_blank_lines(s: &str) -> String {
    let mut result = s.to_string();
    while result.contains("\n\n\n") {
        result = result.replace("\n\n\n", "\n\n");
    }
    // Trim trailing spaces on each line.
    let trimmed: Vec<String> = result.lines().map(|l| l.trim_end().to_string()).collect();
    trimmed.join("\n").trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_title() {
        let html = "<html><head><title>Hello World</title></head></html>";
        assert_eq!(extract_title(html).as_deref(), Some("Hello World"));
    }

    #[test]
    fn test_extract_title_with_attrs() {
        let html = "<title data-x=\"1\">  Spaced  </title>";
        assert_eq!(extract_title(html).as_deref(), Some("Spaced"));
    }

    #[test]
    fn test_extract_title_missing() {
        assert_eq!(extract_title("<html></html>"), None);
    }

    #[test]
    fn test_extract_author_meta() {
        let html = r#"<meta name="author" content="Jane Doe">"#;
        assert_eq!(extract_author(html).as_deref(), Some("Jane Doe"));
    }

    #[test]
    fn test_extract_author_single_quotes() {
        let html = "<meta name='author' content='John Smith'>";
        assert_eq!(extract_author(html).as_deref(), Some("John Smith"));
    }

    #[test]
    fn test_extract_author_missing() {
        assert_eq!(extract_author("<html></html>"), None);
    }

    #[test]
    fn test_entities() {
        assert_eq!(decode_basic_entities("a &amp; b &lt;c&gt;"), "a & b <c>");
    }

    #[test]
    fn test_generic_adapter_matches_any() {
        let adapter = GenericAdapter;
        let url = url::Url::parse("https://example.com/post").unwrap();
        assert!(adapter.matches(&url));
    }

    #[test]
    fn test_generic_adapter_extract_title() {
        let adapter = GenericAdapter;
        let page = FetchedPage {
            url: url::Url::parse("https://example.com/post").unwrap(),
            html: r#"<title>My Article</title><meta name="author" content="A. Writer">"#
                .to_string(),
        };
        let extracted = adapter.extract(&page).unwrap();
        assert_eq!(extracted.title, "My Article");
        assert_eq!(extracted.author.as_deref(), Some("A. Writer"));
    }

    #[test]
    fn test_html_to_markup_paragraphs_and_headings() {
        let html = "<h1>Title</h1><p>Hello <strong>world</strong>.</p>";
        let md = html_to_markup(html);
        assert!(md.contains("# Title"));
        assert!(md.contains("Hello **world**."));
    }

    #[test]
    fn test_html_to_markup_list() {
        let html = "<ul><li>one</li><li>two</li></ul>";
        let md = html_to_markup(html);
        assert!(md.contains("- one"));
        assert!(md.contains("- two"));
    }

    #[test]
    fn test_html_to_markup_link() {
        let html = r#"<p>See <a href="https://example.com">this site</a>.</p>"#;
        let md = html_to_markup(html);
        assert!(md.contains("[this site](https://example.com)"), "got: {md}");
    }

    #[test]
    fn test_html_to_markup_strips_scripts() {
        let html = "<p>Keep</p><script>alert('x')</script><style>.a{}</style>";
        let md = html_to_markup(html);
        assert!(md.contains("Keep"));
        assert!(!md.contains("alert"));
        assert!(!md.contains(".a{}"));
    }

    #[test]
    fn test_extract_body_region_prefers_article() {
        let html = "<body><nav>menu</nav><article><p>Real content</p></article></body>";
        let region = extract_body_region(html);
        assert!(region.contains("Real content"));
        assert!(!region.contains("menu"));
    }

    #[test]
    fn test_clean_title_strips_site_suffix() {
        assert_eq!(clean_title("My Article — argmin.net"), "My Article");
        assert_eq!(clean_title("My Article | Site Name"), "My Article");
        assert_eq!(clean_title("My Article - Blog"), "My Article");
        assert_eq!(clean_title("No Separator Here"), "No Separator Here");
    }

    #[test]
    fn test_extract_first_heading() {
        let html = "<article><h1>Real <em>Title</em></h1><p>body</p></article>";
        assert_eq!(extract_first_heading(html).as_deref(), Some("Real Title"));
    }

    #[test]
    fn test_strip_leading_title_removes_duplicate() {
        let body = "# My Article\n\nFirst paragraph.";
        let out = strip_leading_title(body, "My Article");
        assert_eq!(out, "First paragraph.");
    }

    #[test]
    fn test_strip_leading_title_keeps_distinct_heading() {
        let body = "# Introduction\n\nText.";
        let out = strip_leading_title(body, "My Article");
        assert!(out.starts_with("# Introduction"));
    }

    #[test]
    fn test_adapter_title_from_h1_not_duplicated_in_body() {
        let adapter = GenericAdapter;
        let page = FetchedPage {
            url: url::Url::parse("https://argmin.net/p/x").unwrap(),
            html: concat!(
                "<title>Faster Fourier Transforms — argmin.net</title>",
                "<article><h1>Faster Fourier Transforms</h1>",
                "<p>Intro paragraph.</p></article>"
            )
            .to_string(),
        };
        let extracted = adapter.extract(&page).unwrap();
        assert_eq!(extracted.title, "Faster Fourier Transforms");
        assert!(!extracted.body.contains("Faster Fourier Transforms"));
        assert!(extracted.body.contains("Intro paragraph."));
    }

    #[test]
    fn test_generic_adapter_extract_body() {
        let adapter = GenericAdapter;
        let page = FetchedPage {
            url: url::Url::parse("https://example.com/post").unwrap(),
            html: "<article><h1>Heading</h1><p>Body paragraph.</p><p>More.</p></article>"
                .to_string(),
        };
        let extracted = adapter.extract(&page).unwrap();
        // The <h1> becomes the title, so it is not repeated in the body.
        assert_eq!(extracted.title, "Heading");
        assert!(!extracted.body.contains("# Heading"));
        assert!(extracted.body.contains("Body paragraph."));
    }

    #[test]
    fn test_render_markup_has_frontmatter() {
        let archiver = Archiver::new();
        let page = ExtractedPage {
            title: "Test".to_string(),
            author: Some("Someone".to_string()),
            body: "Body text".to_string(),
        };
        let markup = archiver.render_markup(&page);
        assert!(markup.starts_with("---\n"));
        assert!(markup.contains("title: Test\n"));
        assert!(markup.contains("author: Someone\n"));
        assert!(markup.contains("Body text"));
    }
}
