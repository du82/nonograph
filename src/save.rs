use std::fs;
use std::path::Path;

use crate::Post;

pub fn save_post_to_file(post: &Post) -> Result<(), String> {
    save_post_to_file_in_dir(post, ".")
}

pub fn save_post_to_file_in_dir(post: &Post, base_dir: &str) -> Result<(), String> {
    let content_dir = Path::new(base_dir).join("content");
    if !content_dir.exists() {
        fs::create_dir_all(&content_dir)
            .map_err(|e| format!("Failed to create content directory: {}", e))?;
    }

    let filename = format!("{}.md", post.id);
    let file_path = content_dir.join(filename);

    // Create file content with date at top, optionally author with pipe, empty line, title as h1, then user content
    let header = if post.author.is_empty() {
        post.created_at.format("%B %d, %Y").to_string()
    } else {
        format!("{} | {}", post.created_at.format("%B %d, %Y"), post.author)
    };

    let file_content = format!("{}\n\n# {}\n{}", header, post.title, post.raw_content);

    fs::write(&file_path, file_content)
        .map_err(|e| format!("Failed to write post to file {:?}: {}", file_path, e))?;

    Ok(())
}

pub fn post_file_exists(post_id: &str) -> bool {
    post_file_exists_in_dir(post_id, ".")
}

pub fn post_file_exists_in_dir(post_id: &str, base_dir: &str) -> bool {
    let filename = format!("{}.md", post_id);
    Path::new(base_dir).join("content").join(filename).exists()
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Datelike, Utc};
    use serial_test::serial;
    use std::path::PathBuf;
    use tempfile::tempdir;

    fn setup_test_env() -> (tempfile::TempDir, PathBuf) {
        let temp_dir = tempdir().unwrap();
        let content_dir = temp_dir.path().join("content");
        std::fs::create_dir_all(&content_dir).unwrap();
        (temp_dir, content_dir)
    }

    #[test]
    fn test_ensure_content_directory() {
        let (_temp_dir, content_dir) = setup_test_env();

        // Test that our setup worked - content directory should exist
        assert!(content_dir.exists());
        assert!(content_dir.is_dir());
    }

    #[test]
    #[serial]
    fn test_save_and_load_post() {
        let (temp_dir, _content_dir) = setup_test_env();

        let post = Post {
            id: "test-post-01-01-2024".to_string(),
            title: "Test Post".to_string(),
            author: "Test Author".to_string(),
            content: "<p>Rendered content</p>".to_string(),
            raw_content: "Raw content here".to_string(),
            created_at: Utc::now(),
        };

        let temp_path = temp_dir.path().to_str().unwrap();

        // Save post
        assert!(save_post_to_file_in_dir(&post, temp_path).is_ok());

        // Check file exists
        assert!(post_file_exists_in_dir("test-post-01-01-2024", temp_path));
    }

    #[test]
    #[serial]
    fn test_load_nonexistent_post() {
        let (temp_dir, _content_dir) = setup_test_env();

        let temp_path = temp_dir.path().to_str().unwrap();

        // Test that a non-existent post doesn't exist
        assert!(!post_file_exists_in_dir("nonexistent-post", temp_path));
    }

    #[test]
    #[serial]
    fn test_delete_post_file() {
        let (temp_dir, _content_dir) = setup_test_env();

        let temp_path = temp_dir.path().to_str().unwrap();

        let post = Post {
            id: "delete-test-01-01-2024".to_string(),
            title: "Delete Test".to_string(),
            author: "Test Author".to_string(),
            content: "<p>Content</p>".to_string(),
            raw_content: "Content".to_string(),
            created_at: Utc::now(),
        };

        // Save and verify exists
        save_post_to_file_in_dir(&post, temp_path).unwrap();
        assert!(post_file_exists_in_dir("delete-test-01-01-2024", temp_path));

        // Delete the file
        let file_path = temp_dir
            .path()
            .join("content")
            .join("delete-test-01-01-2024.md");
        fs::remove_file(file_path).unwrap();
        assert!(!post_file_exists_in_dir(
            "delete-test-01-01-2024",
            temp_path
        ));
    }

    #[test]
    #[serial]
    fn test_file_format() {
        let (temp_dir, _content_dir) = setup_test_env();

        let temp_path = temp_dir.path().to_str().unwrap();

        let post = Post {
            id: "format-test-01-01-2024".to_string(),
            title: "Format Test".to_string(),
            author: "Test Author".to_string(),
            content: "<p>Rendered</p>".to_string(),
            raw_content: "This is the user content\nWith multiple lines".to_string(),
            created_at: Utc::now(),
        };

        assert!(save_post_to_file_in_dir(&post, temp_path).is_ok());

        // Read raw file content to verify format
        let file_path = temp_dir
            .path()
            .join("content")
            .join("format-test-01-01-2024.md");
        let raw_file = fs::read_to_string(file_path).unwrap();
        let lines: Vec<&str> = raw_file.lines().collect();

        // First line should be date with author - check for a reasonable year range and author
        let current_year = post.created_at.year();
        assert!(lines[0].contains(&current_year.to_string()));
        assert!(lines[0].contains(" | Test Author"));
        // Second line should be empty
        assert_eq!(lines[1], "");
        // Third line should be title with h1
        assert_eq!(lines[2], "# Format Test");
        // Fourth line should start user content
        assert!(lines[3] == "This is the user content");
        assert!(lines[4] == "With multiple lines");
    }

    #[test]
    #[serial]
    fn test_file_format_no_author() {
        let (temp_dir, _content_dir) = setup_test_env();

        let temp_path = temp_dir.path().to_str().unwrap();

        let post = Post {
            id: "no-author-test-01-01-2024".to_string(),
            title: "No Author Test".to_string(),
            author: "".to_string(),
            content: "<p>Rendered</p>".to_string(),
            raw_content: "Content without author".to_string(),
            created_at: Utc::now(),
        };

        assert!(save_post_to_file_in_dir(&post, temp_path).is_ok());

        // Read raw file content to verify format
        let file_path = temp_dir
            .path()
            .join("content")
            .join("no-author-test-01-01-2024.md");
        let raw_file = fs::read_to_string(file_path).unwrap();
        let lines: Vec<&str> = raw_file.lines().collect();

        // First line should be date only (no pipe or author)
        let current_year = post.created_at.year();
        assert!(lines[0].contains(&current_year.to_string()));
        assert!(!lines[0].contains(" | "));
        // Second line should be empty
        assert_eq!(lines[1], "");
        // Third line should be title with h1
        assert_eq!(lines[2], "# No Author Test");
        // Fourth line should start user content
        assert!(lines[3] == "Content without author");
    }
}
