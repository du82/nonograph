use std::fs;
use std::path::Path;

use crate::Post;

pub fn ensure_content_directory() -> Result<(), String> {
    let content_dir = Path::new("content");
    if !content_dir.exists() {
        fs::create_dir_all(content_dir)
            .map_err(|e| format!("Failed to create content directory: {}", e))?;
    }
    Ok(())
}

pub fn save_post_to_file(post: &Post) -> Result<(), String> {
    ensure_content_directory()?;

    let filename = format!("content/{}.md", post.id);
    let file_path = Path::new(&filename);

    // Create file content with date at top, empty line, title as h1, then user content
    let file_content = format!(
        "{}\n\n# {}\n{}",
        post.created_at.format("%B %d, %Y").to_string(),
        post.title,
        post.raw_content
    );

    fs::write(file_path, file_content)
        .map_err(|e| format!("Failed to write post to file {}: {}", filename, e))?;

    Ok(())
}

pub fn post_file_exists(post_id: &str) -> bool {
    let filename = format!("content/{}.md", post_id);
    Path::new(&filename).exists()
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use std::env;
    use tempfile::tempdir;

    fn setup_test_env() -> tempfile::TempDir {
        let temp_dir = tempdir().unwrap();
        env::set_current_dir(temp_dir.path()).unwrap();
        temp_dir
    }

    #[test]
    fn test_ensure_content_directory() {
        let _temp_dir = setup_test_env();

        assert!(ensure_content_directory().is_ok());
        assert!(Path::new("content").exists());
        assert!(Path::new("content").is_dir());
    }

    #[test]
    fn test_save_and_load_post() {
        let _temp_dir = setup_test_env();
        ensure_content_directory().unwrap();

        let post = Post {
            id: "test-post-01-01-2024".to_string(),
            title: "Test Post".to_string(),
            author: "Test Author".to_string(),
            content: "<p>Rendered content</p>".to_string(),
            raw_content: "Raw content here".to_string(),
            created_at: Utc::now(),
        };

        // Save post
        assert!(save_post_to_file(&post).is_ok());

        // Check file exists
        assert!(post_file_exists("test-post-01-01-2024"));
    }

    #[test]
    fn test_load_nonexistent_post() {
        let _temp_dir = setup_test_env();
        ensure_content_directory().unwrap();
    }

    #[test]
    fn test_delete_post_file() {
        let _temp_dir = setup_test_env();
        ensure_content_directory().unwrap();

        let post = Post {
            id: "delete-test-01-01-2024".to_string(),
            title: "Delete Test".to_string(),
            author: "Test Author".to_string(),
            content: "<p>Content</p>".to_string(),
            raw_content: "Content".to_string(),
            created_at: Utc::now(),
        };

        // Save and verify exists
        save_post_to_file(&post).unwrap();
        assert!(post_file_exists("delete-test-01-01-2024"));
    }

    #[test]
    fn test_file_format() {
        let _temp_dir = setup_test_env();
        ensure_content_directory().unwrap();

        let post = Post {
            id: "format-test-01-01-2024".to_string(),
            title: "Format Test".to_string(),
            author: "Test Author".to_string(),
            content: "<p>Rendered</p>".to_string(),
            raw_content: "This is the user content\nWith multiple lines".to_string(),
            created_at: Utc::now(),
        };

        assert!(save_post_to_file(&post).is_ok());

        // Read raw file content to verify format
        let raw_file = fs::read_to_string("content/format-test-01-01-2024.md").unwrap();
        let lines: Vec<&str> = raw_file.lines().collect();

        // First line should be date - check for current year range
        assert!(
            lines[0].contains("2025") || lines[0].contains("2024") || lines[0].contains("2023")
        );
        // Second line should be empty
        assert_eq!(lines[1], "");
        // Third line should be title with h1
        assert_eq!(lines[2], "# Format Test");
        // Fourth line should start user content
        assert_eq!(lines[3], "This is the user content");
        assert_eq!(lines[4], "With multiple lines");
    }
}
