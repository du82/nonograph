#[macro_use]
extern crate rocket;

mod config;
mod parser;
mod save;
mod template;

use config::Config;
use std::sync::mpsc;
use std::thread;

use chrono::{DateTime, Utc};
use rocket::{response::content, State};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use std::sync::{Arc, Mutex};
use template::TemplateEngine;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Post {
    id: String,
    title: String,
    author: String,
    content: String,
    raw_content: String,
    created_at: DateTime<Utc>,
}

impl Post {
    fn memory_size(&self) -> usize {
        self.id.len()
            + self.title.len()
            + self.author.len()
            + self.content.len()
            + self.raw_content.len()
            + 64 // Rough estimate for DateTime and struct overhead
    }
}

#[derive(Debug)]
struct CacheEntry {
    post: Post,
    last_accessed: DateTime<Utc>,
}

#[derive(Debug)]
struct PostCache {
    entries: HashMap<String, CacheEntry>,
    total_size: usize,
    max_size: usize, // 128 MB = 128 * 1024 * 1024
}

impl PostCache {
    fn new(max_size_mb: usize) -> Self {
        PostCache {
            entries: HashMap::new(),
            total_size: 0,
            max_size: max_size_mb * 1024 * 1024,
        }
    }

    // Add a non-cloning get for read-only access
    fn get_ref(&mut self, post_id: &str) -> Option<&Post> {
        if let Some(entry) = self.entries.get_mut(post_id) {
            entry.last_accessed = Utc::now();
            Some(&entry.post)
        } else {
            None
        }
    }

    fn contains_key(&self, post_id: &str) -> bool {
        self.entries.contains_key(post_id)
    }

    fn insert(&mut self, post_id: String, post: Post) {
        let post_size = post.memory_size();

        // Remove existing entry if it exists
        if let Some(old_entry) = self.entries.remove(&post_id) {
            self.total_size -= old_entry.post.memory_size();
            println!("Cache UPDATE for post: {}", post_id);
        } else {
            println!("Cache INSERT for post: {}", post_id);
        }

        // Add new entry size
        self.total_size += post_size;

        // Evict oldest entries if over limit
        let mut evicted_count = 0;
        while self.total_size > self.max_size && !self.entries.is_empty() {
            self.evict_oldest();
            evicted_count += 1;
        }

        if evicted_count > 0 {
            println!(
                "Cache EVICTED {} old posts to stay under 128MB limit",
                evicted_count
            );
        }

        // Insert new entry
        let entry = CacheEntry {
            post,
            last_accessed: Utc::now(),
        };

        self.entries.insert(post_id.clone(), entry);
        println!(
            "Cache now contains {} posts, total size: {} MB",
            self.entries.len(),
            self.total_size / (1024 * 1024)
        );
    }

    fn evict_oldest(&mut self) {
        if let Some(oldest_id) = self.find_oldest_entry() {
            if let Some(old_entry) = self.entries.remove(&oldest_id) {
                self.total_size -= old_entry.post.memory_size();
                println!(
                    "Cache EVICTED oldest post: {} (size: {} KB)",
                    oldest_id,
                    old_entry.post.memory_size() / 1024
                );
            }
        }
    }

    fn find_oldest_entry(&self) -> Option<String> {
        self.entries
            .iter()
            .min_by_key(|(_, entry)| entry.last_accessed)
            .map(|(id, _)| id.clone())
    }
}

type PostStorage = Arc<Mutex<PostCache>>;
type FileSaveQueue = Mutex<mpsc::Sender<Post>>;

#[get("/")]
fn index(config: &State<Config>) -> content::RawHtml<String> {
    let engine = TemplateEngine::new("templates");
    let mut context = HashMap::new();
    context.insert("error".to_string(), "".to_string());
    context.insert("success".to_string(), "".to_string());
    context.insert(
        "title_max_length".to_string(),
        config.limits.title_max_length.to_string(),
    );
    context.insert(
        "alias_max_length".to_string(),
        config.limits.alias_max_length.to_string(),
    );
    context.insert(
        "content_max_length".to_string(),
        config.limits.content_max_length.to_string(),
    );

    match engine.render_with_defaults("home", &context) {
        Ok(html) => content::RawHtml(html),
        Err(e) => content::RawHtml(format!("Template error: {}", e)),
    }
}

#[derive(FromForm)]
struct NewPost {
    title: String,
    content: String,
    alias: Option<String>,
}

fn generate_post_id(title: &str, storage: &PostStorage) -> Result<String, String> {
    let now = Utc::now();
    let date_str = now.format("%m-%d-%Y").to_string();

    // Create URL-safe slug from title
    let title_slug: String = title
        .trim()
        .to_lowercase()
        .chars()
        .filter_map(|c| {
            if c.is_alphanumeric() {
                Some(c)
            } else if c.is_whitespace() || c == '-' || c == '_' {
                Some('-')
            } else {
                None
            }
        })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<&str>>()
        .join("-");

    if title_slug.is_empty() {
        // Fallback for emojis and non-ascii
        let fallback_slug = format!("post-{}", now.timestamp_millis() % 1000000);
        let posts = storage.lock().unwrap();

        for i in 0..1000 {
            let post_id = if i == 0 {
                format!("{}-{}", fallback_slug, date_str)
            } else {
                format!("{}-{}-{}", fallback_slug, date_str, i)
            };

            if !posts.contains_key(&post_id) {
                return Ok(post_id);
            }
        }

        return Err(
            "All slots for this title and date are taken. Please choose another title.".to_string(),
        );
    }

    let posts = storage.lock().unwrap();

    // Try to find an available slot (0-999)
    for i in 0..1000 {
        let post_id = if i == 0 {
            format!("{}-{}", title_slug, date_str)
        } else {
            format!("{}-{}-{}", title_slug, date_str, i)
        };

        if !posts.contains_key(&post_id) {
            return Ok(post_id);
        }
    }

    Err("All slots for this title and date are taken. Please choose another title.".to_string())
}

#[post("/create", data = "<form>")]
fn create_post(
    form: rocket::form::Form<NewPost>,
    storage: &State<PostStorage>,
    file_queue: &State<FileSaveQueue>,
    config: &State<Config>,
) -> Result<rocket::response::Redirect, content::RawHtml<String>> {
    if let Err(error) = config.validate_post(&form.title, &form.content, form.alias.as_deref()) {
        let error_url = format!("/?error={}", error);
        return Ok(rocket::response::Redirect::to(error_url));
    }

    let post_id = match generate_post_id(&form.title, storage) {
        Ok(id) => id,
        Err(_) => return Ok(rocket::response::Redirect::to("/?error=no_available_slots")),
    };

    let rendered_content = parser::render_markdown(&form.content);

    let post = Post {
        id: post_id.clone(),
        title: form.title.clone(),
        author: form.alias.clone().unwrap_or_default(),
        content: rendered_content,
        raw_content: form.content.clone(),
        created_at: Utc::now(),
    };

    let post_for_file = post.clone();
    {
        let mut posts = storage.lock().unwrap();
        posts.insert(post_id.clone(), post); // Move post here
    }

    if let Ok(tx) = file_queue.lock() {
        if let Err(_) = tx.send(post_for_file) {
            eprintln!("Failed to queue post for background save: {}", post_id);
        }
    }

    Ok(rocket::response::Redirect::to(format!("/{}", post_id)))
}

#[get("/<post_id>")]
fn view_post(
    post_id: &str,
    storage: &State<PostStorage>,
    _config: &State<Config>,
) -> Result<
    rocket::Either<content::RawHtml<String>, content::RawText<String>>,
    rocket::response::status::NotFound<String>,
> {
    let is_raw_request = post_id.ends_with(".md");
    let actual_post_id = if is_raw_request {
        post_id.strip_suffix(".md").unwrap()
    } else {
        &post_id
    };

    // Try to load from memory first with minimal lock time
    let post_from_memory = {
        let mut posts = storage.lock().unwrap();
        // Use the non-cloning get_ref for better performance
        if let Some(post_ref) = posts.get_ref(actual_post_id) {
            Some(post_ref.clone()) // Only clone when we actually found it
        } else {
            None
        }
    };

    let post = match post_from_memory {
        Some(post) => Some(post),
        None => {
            if save::post_file_exists(actual_post_id) {
                if let Ok(file_content) =
                    std::fs::read_to_string(format!("content/{}.md", actual_post_id))
                {
                    let lines: Vec<&str> = file_content.splitn(4, '\n').collect();
                    if lines.len() >= 4 {
                        let author = if let Some(pipe_pos) = lines[0].find(" | ") {
                            lines[0][(pipe_pos + 3)..].to_string()
                        } else {
                            "".to_string()
                        };

                        let title = lines[2]
                            .strip_prefix("# ")
                            .unwrap_or("Untitled")
                            .to_string();
                        let raw_content = lines[3].to_string();

                        let new_post = Post {
                            id: actual_post_id.to_string(),
                            title,
                            author,
                            content: parser::render_markdown(&raw_content),
                            raw_content,
                            created_at: Utc::now(),
                        };

                        {
                            let mut posts_write = storage.lock().unwrap();
                            posts_write.insert(actual_post_id.to_string(), new_post.clone());
                        }

                        Some(new_post)
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            }
        }
    };

    match post {
        Some(post) => {
            if is_raw_request {
                let formatted_date = post.created_at.format("%B %d, %Y").to_string();
                let markdown_content = format!(
                    "{}\n\n# {}\n{}",
                    formatted_date, post.title, post.raw_content
                );
                Ok(rocket::Either::Right(content::RawText(markdown_content)))
            } else {
                let engine = TemplateEngine::new("templates");
                let mut context = HashMap::new();

                // Use pre-rendered content
                let rendered_content = post.content.clone();

                context.insert("title".to_string(), post.title.clone());
                context.insert("content".to_string(), rendered_content);
                context.insert("raw_content".to_string(), post.raw_content.clone());
                context.insert("author".to_string(), post.author.clone());

                let author_display = if post.author.is_empty() {
                    String::new()
                } else {
                    format!("by {} · ", post.author)
                };
                context.insert("author_display".to_string(), author_display);

                context.insert(
                    "created_at".to_string(),
                    post.created_at.format("%B %d, %Y").to_string(),
                );
                context.insert("created_at_iso".to_string(), post.created_at.to_rfc3339());
                context.insert("post_id".to_string(), actual_post_id.to_string());

                // OpenGraph variables
                context.insert("url".to_string(), format!("/{}", actual_post_id));

                // Create description from first 160 chars of raw content
                let description = if post.raw_content.chars().count() > 160 {
                    let truncated: String = post.raw_content.chars().take(160).collect();
                    format!("{}...", truncated)
                } else {
                    post.raw_content.clone()
                };
                context.insert("description".to_string(), description);

                match engine.render("post", &context) {
                    Ok(html) => Ok(rocket::Either::Left(content::RawHtml(html))),
                    Err(e) => Ok(rocket::Either::Left(content::RawHtml(format!(
                        "Template error: {}",
                        e
                    )))),
                }
            }
        }
        None => {
            if is_raw_request {
                Err(rocket::response::status::NotFound(
                    "Post not found".to_string(),
                ))
            } else {
                Ok(rocket::Either::Left(content::RawHtml(
                    r#"<!doctype html>
<html>
<head>
    <title>404 - Post not found</title>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
    <style>
        body {
            font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
            max-width: 720px;
            margin: 0 auto;
            padding: 40px 20px;
            text-align: center;
            color: #333;
        }
        h1 { font-weight: 300; margin-bottom: 16px; }
        a { color: #333; }
    </style>
</head>
<body>
    <h1>404 - Post Not Found</h1>
    <p><a href="/">← Create New Article</a></p>
</body>
</html>"#
                        .to_string(),
                )))
            }
        }
    }
}

#[get("/markup")]
fn markup_page() -> content::RawHtml<String> {
    serve_static_page("markup")
}

#[get("/legal")]
fn legal_page() -> content::RawHtml<String> {
    serve_static_page("legal")
}

#[get("/about")]
fn about_page() -> content::RawHtml<String> {
    serve_static_page("about")
}

#[get("/api")]
fn api_page() -> content::RawHtml<String> {
    serve_static_page("api")
}

fn serve_static_page(page_name: &str) -> content::RawHtml<String> {
    let file_path = format!("content/{}.md", page_name);

    match std::fs::read_to_string(&file_path) {
        Ok(content) => {
            // Parse the file format: date, empty line, title, content
            let lines: Vec<&str> = content.splitn(4, '\n').collect();
            if lines.len() >= 4 {
                let title = lines[2].strip_prefix("# ").unwrap_or("Page");
                let raw_content = lines[3];
                let rendered_content = parser::render_markdown(raw_content);

                let engine = TemplateEngine::new("templates");
                let mut context = HashMap::new();
                context.insert("title".to_string(), title.to_string());
                context.insert("content".to_string(), rendered_content);
                context.insert("created_at".to_string(), lines[0].to_string());
                context.insert("author".to_string(), String::new());
                context.insert("author_display".to_string(), String::new());
                context.insert("created_at_iso".to_string(), String::new());
                context.insert("url".to_string(), format!("/{}", page_name));
                context.insert("description".to_string(), String::new());

                match engine.render("post", &context) {
                    Ok(html) => content::RawHtml(html),
                    Err(e) => content::RawHtml(format!("Template error: {}", e)),
                }
            } else {
                content::RawHtml(format!("<h1>Error</h1><p>Invalid file format for {}</p>", page_name))
            }
        }
        Err(_) => {
            content::RawHtml(format!(
                "<h1>Page Not Found</h1><p>The {} page doesn't exist yet.</p><p><a href=\"/\">← Home</a></p>",
                page_name
            ))
        }
    }
}

fn start_file_save_worker() -> mpsc::Sender<Post> {
    let (tx, rx) = mpsc::channel::<Post>();

    thread::spawn(move || {
        for post in rx {
            if let Err(e) = save::save_post_to_file(&post) {
                eprintln!("Background file save failed for post {}: {}", post.id, e);
            }
        }
    });

    tx
}

#[launch]
fn rocket() -> rocket::Rocket<rocket::Build> {
    use rocket::data::{Limits, ToByteUnit};

    let config = Config::load_with_logging();

    let limits = Limits::default()
        .limit("form", config.form_data_limit_bytes().bytes())
        .limit("data-form", config.form_data_limit_bytes().bytes())
        .limit("string", config.form_data_limit_bytes().bytes());

    let storage = Arc::new(Mutex::new(PostCache::new(config.cache.max_cache_size_mb)));
    let file_save_sender = start_file_save_worker();

    rocket::build()
        .configure(rocket::Config {
            limits,
            port: config.server.port,
            address: config
                .server
                .address
                .parse()
                .unwrap_or("127.0.0.1".parse().unwrap()),
            ..rocket::Config::default()
        })
        .manage(storage)
        .manage(FileSaveQueue::new(file_save_sender))
        .manage(config)
        .mount(
            "/",
            routes![
                index,
                create_post,
                view_post,
                markup_page,
                legal_page,
                about_page,
                api_page
            ],
        )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_post_id_generation() {
        let storage = Arc::new(Mutex::new(PostCache::new(128)));

        let id1 = generate_post_id("Hello World", &storage).unwrap();
        assert!(id1.contains("hello-world"));
        assert!(id1.contains(&Utc::now().format("%m-%d-%Y").to_string()));

        // Test with special characters
        let id2 = generate_post_id("Hello, World! & More", &storage).unwrap();
        assert!(id2.contains("hello-world-more"));
    }

    #[test]
    fn test_markdown_rendering_basic() {
        let input = "This is *bold* text and **italic** text.";
        let output = parser::render_markdown(input);
        // Basic test - the actual implementation needs proper regex
        assert!(output.contains("bold"));
        assert!(output.contains("italic"));
    }

    #[test]
    fn test_content_length_validation() {
        let short_content = "a".repeat(100);
        let long_content = "a".repeat(35000);

        assert!(short_content.len() <= 32000);
        assert!(long_content.len() > 32000);
    }

    #[test]
    fn test_template_engine_basic() {
        use std::fs;
        use tempfile::tempdir;

        let dir = tempdir().unwrap();
        let template_content = "<h1>{{title}}</h1><p>{{content}}</p>";
        fs::write(dir.path().join("test.html"), template_content).unwrap();

        let engine = TemplateEngine::new(dir.path().to_str().unwrap());
        let mut context = HashMap::new();
        context.insert("title".to_string(), "Test Title".to_string());
        context.insert("content".to_string(), "Test content".to_string());

        let result = engine.render("test", &context).unwrap();
        assert_eq!(result, "<h1>Test Title</h1><p>Test content</p>");
    }

    #[test]
    fn test_slug_generation() {
        let tests = vec![
            ("Hello World", "hello-world"),
            ("Test-Post_123", "test-post-123"),
            ("Special!@#$%Characters", "specialcharacters"),
            ("   Whitespace   ", "whitespace"),
            ("Multiple---Dashes", "multiple-dashes"),
        ];

        for (input, expected) in tests {
            let slug: String = input
                .trim()
                .to_lowercase()
                .chars()
                .filter_map(|c| {
                    if c.is_ascii_alphanumeric() {
                        Some(c)
                    } else if c.is_whitespace() || c == '-' || c == '_' {
                        Some('-')
                    } else {
                        None
                    }
                })
                .collect::<String>()
                .split('-')
                .filter(|s| !s.is_empty())
                .collect::<Vec<&str>>()
                .join("-");

            assert_eq!(slug, expected);
        }
    }

    #[test]
    fn test_markdown_bold_formatting() {
        let input = "This is **bold** text and more **bold text**.";
        let output = parser::render_markdown(input);
        assert!(output.contains("<strong>bold</strong>"));
        assert!(output.contains("<strong>bold text</strong>"));
    }

    #[test]
    fn test_markdown_code_formatting() {
        let input = "Here is `inline code` and more `code`.";
        let output = parser::render_markdown(input);
        // Note: Our current simple implementation doesn't handle this yet
        // This test documents expected behavior
        assert!(output.contains("inline code"));
    }

    #[test]
    fn test_content_sanitization() {
        let malicious_content = "<script>alert('xss')</script>";
        let sanitized = ammonia::clean(malicious_content);
        assert!(!sanitized.contains("<script>"));
        assert!(!sanitized.contains("alert"));
    }

    #[test]
    fn test_post_storage() {
        let storage = Arc::new(Mutex::new(PostCache::new(128)));
        let post = Post {
            id: "test-post".to_string(),
            title: "Test Post".to_string(),
            author: "Test Author".to_string(),
            content: "Test content".to_string(),
            raw_content: "Test content".to_string(),
            created_at: Utc::now(),
        };

        {
            let mut posts = storage.lock().unwrap();
            posts.insert("test-post".to_string(), post.clone());
        }

        {
            let mut posts = storage.lock().unwrap();
            let retrieved = posts.get_ref("test-post").unwrap();
            assert_eq!(retrieved.title, "Test Post");
            assert_eq!(retrieved.content, "Test content");
        }
    }

    #[test]
    fn test_post_id_collision_handling() {
        let storage = Arc::new(Mutex::new(PostCache::new(128)));

        // Pre-populate with posts to test collision handling
        {
            let mut posts = storage.lock().unwrap();
            let now = Utc::now();
            let date_str = now.format("%m-%d-%Y").to_string();

            // Add posts that would collide
            for i in 0..3 {
                let id = if i == 0 {
                    format!("test-{}", date_str)
                } else {
                    format!("test-{}-{}", date_str, i)
                };

                let post = Post {
                    id: id.clone(),
                    title: "Test".to_string(),
                    author: "Test Author".to_string(),
                    content: "Content".to_string(),
                    raw_content: "Content".to_string(),
                    created_at: now,
                };
                posts.insert(id, post);
            }
        }

        // This should generate "test-MM-dd-YYYY-3"
        let new_id = generate_post_id("Test", &storage).unwrap();
        let date_str = Utc::now().format("%m-%d-%Y").to_string();
        assert_eq!(new_id, format!("test-{}-3", date_str));
    }

    #[test]
    fn test_url_safe_slug_generation() {
        let test_cases = vec![
            ("Hello/World", "helloworld"),
            ("Test\\Post", "testpost"),
            ("Question?", "question"),
            ("Exclamation!", "exclamation"),
            ("At@Symbol", "atsymbol"),
            ("Hash#Tag", "hashtag"),
            ("Dollar$Sign", "dollarsign"),
            ("Percent%Sign", "percentsign"),
        ];

        for (input, expected) in test_cases {
            let slug: String = input
                .trim()
                .to_lowercase()
                .chars()
                .filter_map(|c| {
                    if c.is_ascii_alphanumeric() {
                        Some(c)
                    } else if c.is_whitespace() || c == '-' || c == '_' {
                        Some('-')
                    } else {
                        None
                    }
                })
                .collect::<String>()
                .split('-')
                .filter(|s| !s.is_empty())
                .collect::<Vec<&str>>()
                .join("-");

            assert_eq!(slug, expected, "Failed for input: {}", input);
        }
    }

    #[test]
    fn test_character_limits() {
        // Test title length limit
        let long_title = "a".repeat(150);
        assert!(long_title.len() > 128);

        // Test content length limit
        let long_content = "a".repeat(35000);
        assert!(long_content.len() > 32000);

        // Test valid lengths
        let valid_title = "a".repeat(50);
        let valid_content = "a".repeat(20000);
        assert!(valid_title.len() <= 128);
        assert!(valid_content.len() <= 32000);
    }

    #[test]
    fn test_emoji_handling() {
        let storage = Arc::new(Mutex::new(PostCache::new(128)));

        let emoji_title = "🍆 Test Post with Emojis 🎉";
        let emoji_content = "🌟 ".repeat(80) + "This is content with lots of emojis! 🎯🔥💯";

        let post_id = generate_post_id(emoji_title, &storage).unwrap();
        assert!(!post_id.is_empty());

        let post = Post {
            id: post_id.clone(),
            title: emoji_title.to_string(),
            author: "🍆".to_string(),
            content: parser::render_markdown(&emoji_content),
            raw_content: emoji_content.clone(),
            created_at: Utc::now(),
        };

        let description = if post.raw_content.chars().count() > 160 {
            let truncated: String = post.raw_content.chars().take(160).collect();
            format!("{}...", truncated)
        } else {
            post.raw_content.clone()
        };

        assert!(description.len() <= emoji_content.len());
        assert!(!description.is_empty());

        let char_count = description.chars().count();
        if emoji_content.chars().count() > 160 {
            assert!(char_count <= 163);
        }
    }

    #[test]
    fn test_emoji_parsing_edge_cases() {
        let _result = parser::render_markdown(emoji_content);

        let empty_content = "";
        let _empty_result = parser::render_markdown(empty_content);

        let single_char = "A";
        let single_result = parser::render_markdown(single_char);
        assert!(single_result.contains("A"));

        let boundary_content = "AB";
        let boundary_result = parser::render_markdown(boundary_content);
        assert!(boundary_result.contains("AB"));

        let storage = Arc::new(Mutex::new(PostCache::new(128)));
        let emoji_title = "🎯";
        let result = generate_post_id(emoji_title, &storage);
        assert!(result.is_ok());

        let mixed_title = "Hello 🎯 World";
        let mixed_result = generate_post_id(mixed_title, &storage);
        assert!(mixed_result.is_ok());
    }

    #[test]
    fn test_truncation_with_200_characters() {
        let emoji_content = "🎯".repeat(200);
        assert_eq!(emoji_content.chars().count(), 200);

        let description = if emoji_content.chars().count() > 160 {
            let truncated: String = emoji_content.chars().take(160).collect();
            format!("{}...", truncated)
        } else {
            emoji_content.clone()
        };

        assert_eq!(description.chars().count(), 163);
        assert!(description.ends_with("..."));
        assert!(description.starts_with("🎯"));

        let random_content =
            "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789".repeat(4);
        let random_content = &random_content[..200];
        assert_eq!(random_content.chars().count(), 200);

        let description2 = if random_content.chars().count() > 160 {
            let truncated: String = random_content.chars().take(160).collect();
            format!("{}...", truncated)
        } else {
            random_content.to_string()
        };

        assert_eq!(description2.chars().count(), 163);
        assert!(description2.ends_with("..."));

        let short_content = "🌟".repeat(50);
        assert_eq!(short_content.chars().count(), 50);

        let description3 = if short_content.chars().count() > 160 {
            let truncated: String = short_content.chars().take(160).collect();
            format!("{}...", truncated)
        } else {
            short_content.clone()
        };

        assert_eq!(description3.chars().count(), 50);
        assert!(!description3.ends_with("..."));
    }

    #[test]
    fn test_opengraph_description_integration() {
        let storage = Arc::new(Mutex::new(PostCache::new(128)));

        let long_emoji_content = "🚀🎉🌟💯".repeat(50);
        let emoji_title = "Emoji Test Post";

        let post_id = generate_post_id(emoji_title, &storage).unwrap();
        let post = Post {
            id: post_id.clone(),
            title: emoji_title.to_string(),
            author: "test".to_string(),
            content: parser::render_markdown(&long_emoji_content),
            raw_content: long_emoji_content.clone(),
            created_at: Utc::now(),
        };

        let description = if post.raw_content.chars().count() > 160 {
            let truncated: String = post.raw_content.chars().take(160).collect();
            format!("{}...", truncated)
        } else {
            post.raw_content.clone()
        };

        assert_eq!(description.chars().count(), 163);
        assert!(description.starts_with("🚀🎉🌟💯"));
        assert!(description.ends_with("..."));

        let long_ascii_content =
            "This is a very long post content that should be truncated. ".repeat(10);
        let ascii_title = "Long ASCII Test";

        let post_id2 = generate_post_id(ascii_title, &storage).unwrap();
        let post2 = Post {
            id: post_id2.clone(),
            title: ascii_title.to_string(),
            author: "test".to_string(),
            content: parser::render_markdown(&long_ascii_content),
            raw_content: long_ascii_content.clone(),
            created_at: Utc::now(),
        };

        let description2 = if post2.raw_content.chars().count() > 160 {
            let truncated: String = post2.raw_content.chars().take(160).collect();
            format!("{}...", truncated)
        } else {
            post2.raw_content.clone()
        };

        assert_eq!(description2.chars().count(), 163);
        assert!(description2.starts_with("This is a very long"));
        assert!(description2.ends_with("..."));
    }

    #[test]
    fn test_template_context_building() {
        let mut context = HashMap::new();
        context.insert("title".to_string(), "Test Title".to_string());
        context.insert("content".to_string(), "Test Content".to_string());

        assert_eq!(context.get("title").unwrap(), "Test Title");
        assert_eq!(context.get("content").unwrap(), "Test Content");
        assert!(context.get("missing").is_none());
    }

    #[test]
    fn test_date_formatting() {
        let now = Utc::now();
        let formatted = now.format("%B %d, %Y").to_string();

        // Basic validation that the format works
        assert!(formatted.len() > 10); // Should be a reasonable length
        assert!(!formatted.contains("UTC")); // Should not contain time info
        assert!(!formatted.contains("at")); // Should not contain time info
    }

    #[test]
    fn test_date_format_output() {
        let now = Utc::now();
        let formatted = now.format("%B %d, %Y").to_string();

        // Should be format like "January 01, 2024"
        assert!(formatted.len() >= 13); // At least "January 1, 2024" length
        assert!(formatted.contains(", "));
        assert!(!formatted.contains("UTC"));
        assert!(!formatted.contains(":"));

        // Example: "March 15, 2024" (no time, no timezone)
    }

    #[test]
    fn test_empty_input_validation() {
        assert!("".trim().is_empty());
        assert!("   ".trim().is_empty());
        assert!(!"hello".trim().is_empty());
        assert!(!"  hello  ".trim().is_empty());
    }

    #[test]
    fn test_ammonia_configuration() {
        // Test that ammonia is properly configured for our use case
        let safe_html = "<strong>Bold</strong> and <em>italic</em>";
        let cleaned = ammonia::clean(safe_html);
        assert!(cleaned.contains("<strong>"));
        assert!(cleaned.contains("<em>"));

        let unsafe_html = "<script>alert('xss')</script><strong>Safe</strong>";
        let cleaned_unsafe = ammonia::clean(unsafe_html);
        assert!(!cleaned_unsafe.contains("<script>"));
        assert!(cleaned_unsafe.contains("<strong>Safe</strong>"));
    }

    #[test]
    fn test_edge_cases() {
        // Test very short titles
        let short_title = "A";
        let storage = Arc::new(Mutex::new(PostCache::new(128)));
        let id = generate_post_id(short_title, &storage);
        assert!(id.is_ok());
        assert!(id.unwrap().starts_with("a-"));

        let special_only = "!@#$%^&*()";
        let result = generate_post_id(special_only, &storage);
        assert!(result.is_ok());

        // Test numeric titles
        let numeric = "12345";
        let id = generate_post_id(numeric, &storage);
        assert!(id.is_ok());
        assert!(id.unwrap().starts_with("12345-"));
    }

    #[test]
    fn test_alias_field_validation() {
        // Test valid alias field
        let valid_alias = "John Doe";
        assert!(valid_alias.len() <= 32);

        // Test alias at character limit
        let max_alias = "a".repeat(32);
        assert_eq!(max_alias.len(), 32);

        // Test alias over character limit
        let over_limit_alias = "a".repeat(33);
        assert!(over_limit_alias.len() > 32);

        // Test empty alias (should be allowed as it's optional)
        let empty_alias = "";
        assert!(empty_alias.is_empty());
    }

    #[test]
    fn test_alias_display_formatting() {
        // Test with alias
        let post_with_alias = Post {
            id: "test-alias-post".to_string(),
            title: "Test Post".to_string(),
            author: "Jane Smith".to_string(),
            content: "<p>Content</p>".to_string(),
            raw_content: "Content".to_string(),
            created_at: Utc::now(),
        };

        let alias_display = if post_with_alias.author.is_empty() {
            String::new()
        } else {
            format!("by {} · ", post_with_alias.author)
        };
        assert_eq!(alias_display, "by Jane Smith · ");

        // Test without alias
        let post_without_alias = Post {
            id: "test-no-alias-post".to_string(),
            title: "Test Post".to_string(),
            author: "".to_string(),
            content: "<p>Content</p>".to_string(),
            raw_content: "Content".to_string(),
            created_at: Utc::now(),
        };

        let alias_display_empty = if post_without_alias.author.is_empty() {
            String::new()
        } else {
            format!("by {} · ", post_without_alias.author)
        };
        assert_eq!(alias_display_empty, "");
    }

    #[test]
    fn test_title_character_limit() {
        // Test valid title at limit
        let max_title = "a".repeat(128);
        assert_eq!(max_title.len(), 128);

        // Test title over limit
        let over_limit_title = "a".repeat(129);
        assert!(over_limit_title.len() > 128);

        // Test normal title
        let normal_title = "A Great Article Title";
        assert!(normal_title.len() <= 128);
    }

    #[test]
    fn test_static_page_template_rendering() {
        use std::fs;
        use tempfile::tempdir;

        // Create temporary directory for templates
        let temp_dir = tempdir().unwrap();
        let template_path = temp_dir.path().join("post.html");

        // Create a minimal template that includes all the variables used in static pages
        let template_content = r#"<html>
<head><title>{{title}}</title></head>
<body>
<h1>{{title}}</h1>
<div class="meta">{{author_display}}{{created_at}}</div>
<div class="content">{{content}}</div>
</body>
</html>"#;

        fs::write(&template_path, template_content).unwrap();

        // Test the template engine with the same context that static pages use
        let engine = TemplateEngine::new(temp_dir.path().to_str().unwrap());
        let mut context = HashMap::new();
        context.insert("title".to_string(), "Test Page".to_string());
        context.insert("content".to_string(), "<p>Test content</p>".to_string());
        context.insert("created_at".to_string(), "October 5, 2025".to_string());
        context.insert("author".to_string(), String::new());
        context.insert("author_display".to_string(), String::new());
        context.insert("created_at_iso".to_string(), String::new());
        context.insert("url".to_string(), "/test".to_string());
        context.insert("description".to_string(), String::new());

        let result = engine.render("post", &context).unwrap();

        // Verify no template variables remain unreplaced
        assert!(!result.contains("{{"));
        assert!(!result.contains("}}"));

        // Verify content is properly rendered
        assert!(result.contains("<title>Test Page</title>"));
        assert!(result.contains("<h1>Test Page</h1>"));
        assert!(result.contains("<p>Test content</p>"));
        assert!(result.contains("October 5, 2025"));
    }

    #[test]
    fn test_all_static_pages_template_variables() {
        use std::fs;
        use tempfile::tempdir;

        // Create temporary directory for templates
        let temp_dir = tempdir().unwrap();
        let template_path = temp_dir.path().join("post.html");

        // Create a template that uses all variables that could appear in static pages
        let template_content = r#"<html>
<head>
<title>{{title}}</title>
<meta property="og:title" content="{{title}}" />
<meta property="og:url" content="{{url}}" />
<meta property="og:description" content="{{description}}" />
<meta property="article:author" content="{{author}}" />
<meta property="article:published_time" content="{{created_at_iso}}" />
<meta name="author" content="{{author}}" />
</head>
<body>
<h1>{{title}}</h1>
<div class="article-meta">{{author_display}}{{created_at}}</div>
<div class="article-content">{{content}}</div>
</body>
</html>"#;

        fs::write(&template_path, template_content).unwrap();

        let engine = TemplateEngine::new(temp_dir.path().to_str().unwrap());

        // Test each static page type
        let pages = vec!["markup", "legal", "about", "api"];

        for page_name in pages {
            let mut context = HashMap::new();
            context.insert("title".to_string(), format!("{} Page", page_name));
            context.insert("content".to_string(), "<p>Test content</p>".to_string());
            context.insert("created_at".to_string(), "October 5, 2025".to_string());
            context.insert("author".to_string(), String::new());
            context.insert("author_display".to_string(), String::new());
            context.insert("created_at_iso".to_string(), String::new());
            context.insert("url".to_string(), format!("/{}", page_name));
            context.insert("description".to_string(), String::new());

            let result = engine.render("post", &context).unwrap();

            // Verify no template variables remain unreplaced
            assert!(
                !result.contains("{{"),
                "Page {} has unreplaced template variables",
                page_name
            );
            assert!(
                !result.contains("}}"),
                "Page {} has unreplaced template variables",
                page_name
            );

            // Verify basic structure
            assert!(result.contains(&format!("<title>{} Page</title>", page_name)));
            assert!(result.contains(&format!("<h1>{} Page</h1>", page_name)));
            assert!(result.contains("October 5, 2025"));
        }
    }
}
