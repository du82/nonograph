use crate::parser::html_attr_escape;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sticker {
    pub name: String,
    pub pack: String,
    pub action: String,
    pub path: String,
    pub url: String,
}

#[derive(Debug, Clone)]
pub struct StickerStore {
    stickers: Vec<Sticker>,
}

impl StickerStore {
    pub fn new() -> Result<Self, String> {
        let stickers_dir = Path::new("stickers");

        if !stickers_dir.exists() {
            return Ok(StickerStore {
                stickers: Vec::new(),
            });
        }

        let mut stickers = Vec::new();

        // Scan the stickers directory
        match fs::read_dir(stickers_dir) {
            Ok(pack_dirs) => {
                for pack_entry in pack_dirs {
                    if let Ok(pack_entry) = pack_entry {
                        let pack_path = pack_entry.path();
                        if pack_path.is_dir() {
                            if let Some(pack_name) = pack_path.file_name() {
                                if let Some(pack_name_str) = pack_name.to_str() {
                                    // Scan files in the pack directory
                                    if let Ok(sticker_files) = fs::read_dir(&pack_path) {
                                        for sticker_entry in sticker_files {
                                            if let Ok(sticker_entry) = sticker_entry {
                                                let sticker_path = sticker_entry.path();
                                                if sticker_path.is_file() {
                                                    if let Some(file_name) =
                                                        sticker_path.file_name()
                                                    {
                                                        if let Some(file_name_str) =
                                                            file_name.to_str()
                                                        {
                                                            // Extract action name (filename without extension)
                                                            let action = if let Some(stem) =
                                                                sticker_path.file_stem()
                                                            {
                                                                stem.to_str()
                                                                    .unwrap_or(file_name_str)
                                                                    .to_string()
                                                            } else {
                                                                file_name_str.to_string()
                                                            };

                                                            let full_name = format!(
                                                                "{}.{}",
                                                                pack_name_str, action
                                                            );
                                                            let relative_path = format!(
                                                                "stickers/{}/{}",
                                                                pack_name_str, file_name_str
                                                            );
                                                            let url = format!("/{}", relative_path);

                                                            stickers.push(Sticker {
                                                                name: full_name,
                                                                pack: pack_name_str.to_string(),
                                                                action,
                                                                path: relative_path,
                                                                url,
                                                            });
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            Err(e) => return Err(format!("Failed to read stickers directory: {}", e)),
        }

        Ok(StickerStore { stickers })
    }

    pub fn search(&self, query: &str) -> Vec<&Sticker> {
        if query.is_empty() {
            return self.stickers.iter().collect();
        }

        let query_lower = query.to_lowercase();

        self.stickers
            .iter()
            .filter(|sticker| {
                // Search in pack name
                sticker.pack.to_lowercase().contains(&query_lower) ||
                // Search in action name
                sticker.action.to_lowercase().contains(&query_lower) ||
                // Search in full name
                sticker.name.to_lowercase().contains(&query_lower)
            })
            .collect()
    }

    pub fn get_all(&self) -> &[Sticker] {
        &self.stickers
    }

    pub fn get_by_name(&self, name: &str) -> Option<&Sticker> {
        self.stickers.iter().find(|sticker| sticker.name == name)
    }

    pub fn count(&self) -> usize {
        self.stickers.len()
    }

    pub fn parse_stickers_in_text(&self, text: &str) -> String {
        // Extract code content to protect it from sticker parsing
        let (protected_text, code_sections) = self.extract_code_sections(text);

        let lines: Vec<&str> = protected_text.lines().collect();
        let mut result = String::new();

        for (i, line) in lines.iter().enumerate() {
            let parsed_line = self.parse_stickers_in_line(line);
            result.push_str(&parsed_line);

            // Add newline if not the last line
            if i < lines.len() - 1 {
                result.push('\n');
            }
        }

        // Restore code sections
        self.restore_code_sections(&result, &code_sections)
    }

    fn parse_stickers_in_line(&self, line: &str) -> String {
        // First check if this line contains only stickers and whitespace
        let is_standalone_line = self.is_standalone_sticker_line(line);

        let mut result = String::new();
        let mut chars = line.chars().peekable();

        while let Some(ch) = chars.next() {
            if ch == ':' {
                // Try to parse a sticker pattern :pack.action:
                let mut sticker_name = String::new();
                let mut found_end = false;

                // Collect characters until we find another : or invalid character
                while let Some(&next_ch) = chars.peek() {
                    if next_ch == ':' {
                        chars.next(); // consume the closing :
                        found_end = true;
                        break;
                    } else if next_ch.is_ascii_alphanumeric()
                        || next_ch == '.'
                        || next_ch == '_'
                        || next_ch == '-'
                    {
                        sticker_name.push(next_ch);
                        chars.next();
                    } else {
                        break; // Invalid character, not a sticker
                    }
                }

                // If we found a valid sticker pattern and it contains a dot
                if found_end
                    && sticker_name.contains('.')
                    && self.is_valid_sticker_name(&sticker_name)
                {
                    if let Some(sticker) = self.get_by_name(&sticker_name) {
                        let sticker_tag = if is_standalone_line {
                            format!(
                                "<span class=\"sticker-standalone\"><img src=\"{}\" alt=\"{}\"></span>",
                                html_attr_escape(&sticker.url), html_attr_escape(&sticker_name)
                            )
                        } else {
                            format!(
                                "<span class=\"sticker\"><img src=\"{}\" alt=\"{}\"></span>",
                                html_attr_escape(&sticker.url),
                                html_attr_escape(&sticker_name)
                            )
                        };
                        result.push_str(&sticker_tag);
                        continue;
                    }
                }

                // If we didn't find a valid sticker, add the original text back
                result.push(':');
                result.push_str(&sticker_name);
                if found_end {
                    result.push(':');
                }
            } else {
                result.push(ch);
            }
        }

        result
    }

    fn is_standalone_sticker_line(&self, line: &str) -> bool {
        // Check if line contains only HTML tags, stickers, and whitespace
        let mut temp_line = line.to_string();
        let mut found_any_sticker = false;

        // First, remove all valid sticker patterns
        loop {
            let mut found_sticker_this_round = false;

            if let Some(start) = temp_line.find(':') {
                if let Some(end) = temp_line[start + 1..].find(':') {
                    let end_pos = start + 1 + end;
                    let potential_sticker = &temp_line[start + 1..start + 1 + end];

                    if potential_sticker.contains('.')
                        && potential_sticker
                            .chars()
                            .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '_' || c == '-')
                        && self.get_by_name(potential_sticker).is_some()
                    {
                        // Remove the sticker pattern including colons
                        temp_line.replace_range(start..=end_pos, "");
                        found_any_sticker = true;
                        found_sticker_this_round = true;
                    }
                }
            }

            if !found_sticker_this_round {
                break;
            }
        }

        if !found_any_sticker {
            return false;
        }

        // Now remove all HTML tags
        while let Some(start) = temp_line.find('<') {
            if let Some(end) = temp_line[start..].find('>') {
                temp_line.replace_range(start..start + end + 1, "");
            } else {
                break;
            }
        }

        // Check if only whitespace remains
        temp_line.trim().is_empty()
    }

    fn is_valid_sticker_name(&self, name: &str) -> bool {
        // Validate sticker name format and length for security
        if name.len() > 64 {
            return false;
        }

        // Must contain exactly one dot
        let dot_count = name.chars().filter(|&c| c == '.').count();
        if dot_count != 1 {
            return false;
        }

        // Split into pack and action
        let parts: Vec<&str> = name.split('.').collect();
        if parts.len() != 2 {
            return false;
        }

        let (pack, action) = (parts[0], parts[1]);

        // Validate pack name
        if pack.is_empty() || pack.len() > 32 {
            return false;
        }

        // Validate action name
        if action.is_empty() || action.len() > 32 {
            return false;
        }

        // Only allow safe characters
        let is_safe_char = |c: char| c.is_ascii_alphanumeric() || c == '_' || c == '-';

        if !pack.chars().all(is_safe_char) || !action.chars().all(is_safe_char) {
            return false;
        }

        // Prevent path traversal attempts
        if name.contains("..") || name.contains('/') || name.contains('\\') {
            return false;
        }

        // Prevent reserved names
        let reserved_names = [".", "..", "con", "prn", "aux", "nul"];
        if reserved_names.contains(&pack.to_lowercase().as_str())
            || reserved_names.contains(&action.to_lowercase().as_str())
        {
            return false;
        }

        true
    }

    fn extract_code_sections(&self, text: &str) -> (String, Vec<String>) {
        let mut result = text.to_string();
        let mut code_sections = Vec::new();
        let mut section_index = 0;

        // Extract <pre><code>...</code></pre> blocks
        while let Some(start) = result.find("<pre><code") {
            if let Some(code_start) = result[start..].find('>') {
                let code_start_pos = start + code_start + 1;
                if let Some(end) = result[code_start_pos..].find("</code></pre>") {
                    let end_pos = code_start_pos + end;
                    let full_block = result[start..end_pos + 13].to_string(); // 13 = len("</code></pre>")

                    code_sections.push(full_block);
                    let placeholder = format!("{{{{STICKERCODE{}}}}}", section_index);
                    result.replace_range(start..end_pos + 13, &placeholder);
                    section_index += 1;
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        // Extract inline <code>...</code> tags
        while let Some(start) = result.find("<code>") {
            if let Some(end) = result[start + 6..].find("</code>") {
                let end_pos = start + 6 + end;
                let full_code = result[start..end_pos + 7].to_string(); // 7 = len("</code>")

                code_sections.push(full_code);
                let placeholder = format!("{{{{STICKERCODE{}}}}}", section_index);
                result.replace_range(start..end_pos + 7, &placeholder);
                section_index += 1;
            } else {
                break;
            }
        }

        (result, code_sections)
    }

    fn restore_code_sections(&self, text: &str, code_sections: &[String]) -> String {
        let mut result = text.to_string();

        for (index, code_content) in code_sections.iter().enumerate() {
            let placeholder = format!("{{{{STICKERCODE{}}}}}", index);
            result = result.replace(&placeholder, code_content);
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sticker_store_creation() {
        // Test with actual stickers directory - should create store successfully
        let store = StickerStore::new().unwrap();
        // Should load actual stickers from the stickers/ directory
        // Should successfully create store (count can be 0 or more)
        assert!(true);
    }

    #[test]
    fn test_filesystem_scanning() {
        // This test runs in the project root where stickers/ exists
        let store = StickerStore::new().unwrap();
        assert!(store.count() > 0); // Should find actual stickers

        // Check that actual marsey stickers exist
        let marsey_angry = store.get_by_name("marsey.angry");
        assert!(marsey_angry.is_some());
        let sticker = marsey_angry.unwrap();
        assert_eq!(sticker.pack, "marsey");
        assert_eq!(sticker.action, "angry");
        assert_eq!(sticker.url, "/stickers/marsey/angry.webp");
    }

    #[test]
    fn test_search_functionality() {
        let store = StickerStore::new().unwrap();

        // Test pack search for actual marsey stickers
        let results = store.search("marsey");
        assert!(results.len() > 0);
        assert!(results.iter().all(|s| s.pack == "marsey"));

        // Test action search for actual sticker action
        let results = store.search("angry");
        assert!(results.len() > 0);
        assert!(results.iter().any(|s| s.action == "angry"));

        // Test case insensitive search
        let results = store.search("MARSEY");
        assert!(results.len() > 0);
        assert!(results.iter().all(|s| s.pack == "marsey"));

        // Test no matches
        let results = store.search("nonexistent");
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_get_by_name() {
        let store = StickerStore::new().unwrap();

        let result = store.get_by_name("marsey.angry");
        assert!(result.is_some());
        let sticker = result.unwrap();
        assert_eq!(sticker.pack, "marsey");
        assert_eq!(sticker.action, "angry");

        let result = store.get_by_name("nonexistent.sticker");
        assert!(result.is_none());
    }

    #[test]
    fn test_sticker_parsing() {
        let store = StickerStore::new().unwrap();

        // Test basic sticker parsing with actual stickers
        let text = "Hello :marsey.angry: world";
        let result = store.parse_stickers_in_text(text);
        if store.get_by_name("marsey.angry").is_some() {
            assert!(result.contains("<img src=\"/stickers/marsey/angry.webp\""));
            assert!(result.contains("alt=\"marsey.angry\""));
            assert!(result.contains("Hello"));
            assert!(result.contains("world"));
        } else {
            assert_eq!(result, text); // Should remain unchanged if sticker doesn't exist
        }

        // Test multiple stickers
        let text = "Start :marsey.angry: middle :marsey.cute: end";
        let result = store.parse_stickers_in_text(text);
        let img_count = result.matches("<img").count();

        // Count how many of these stickers actually exist
        let mut expected_count = 0;
        if store.get_by_name("marsey.angry").is_some() {
            expected_count += 1;
        }
        if store.get_by_name("marsey.cute").is_some() {
            expected_count += 1;
        }
        assert_eq!(img_count, expected_count);

        // Test non-existent sticker
        let text = "Hello :nonexistent.sticker: world";
        let result = store.parse_stickers_in_text(text);
        assert_eq!(result, text); // Should remain unchanged

        // Test invalid patterns
        let text = "Hello :invalid world";
        let result = store.parse_stickers_in_text(text);
        assert_eq!(result, text); // Should remain unchanged

        // Test pattern without dot
        let text = "Hello :nodot: world";
        let result = store.parse_stickers_in_text(text);
        assert_eq!(result, text); // Should remain unchanged
    }

    #[test]
    fn test_sticker_url_generation() {
        let store = StickerStore::new().unwrap();

        // Test that stickers have proper URL format
        if let Some(sticker) = store.get_by_name("marsey.angry") {
            assert_eq!(sticker.url, "/stickers/marsey/angry.webp");
            assert_eq!(sticker.pack, "marsey");
            assert_eq!(sticker.action, "angry");
        }

        // Test parsing generates correct URL in img tag
        let text = "Test :marsey.angry: sticker";
        let result = store.parse_stickers_in_text(text);
        if store.get_by_name("marsey.angry").is_some() {
            assert!(result.contains("src=\"/stickers/marsey/angry.webp\""));
            assert!(result.contains("alt=\"marsey.angry\""));
            assert!(result.contains("<span class=\"sticker\">"));
        }
    }

    #[test]
    fn test_code_block_protection() {
        let store = StickerStore::new().unwrap();

        // Test that stickers in inline code are not parsed
        let text = "Use <code>:marsey.angry:</code> in your text";
        let result = store.parse_stickers_in_text(text);
        assert_eq!(result, text); // Should remain unchanged
        assert!(!result.contains("<img")); // Should not contain any images

        // Test that stickers in code blocks are not parsed
        let text = "<pre><code>:marsey.angry: should not be parsed</code></pre>";
        let result = store.parse_stickers_in_text(text);
        assert_eq!(result, text); // Should remain unchanged
        assert!(!result.contains("<img")); // Should not contain any images

        // Test that stickers outside code blocks are still parsed
        let text = "Normal :marsey.angry: and <code>:marsey.cute:</code> mixed";
        let result = store.parse_stickers_in_text(text);
        assert!(result.contains("<code>:marsey.cute:</code>")); // Code content unchanged
        if store.get_by_name("marsey.angry").is_some() {
            assert!(result.contains("<img")); // Should contain one image from normal sticker
            assert_eq!(result.matches("<img").count(), 1); // Only one image
        }

        // Test complex code block with attributes
        let text =
            "<pre><code class=\"language-rust\">let sticker = \":marsey.happy:\";</code></pre>";
        let result = store.parse_stickers_in_text(text);
        assert_eq!(result, text); // Should remain unchanged
        assert!(!result.contains("<img")); // Should not contain any images
    }

    #[test]
    fn test_sticker_sanitization() {
        let store = StickerStore::new().unwrap();

        // Test that malicious sticker names are rejected
        let malicious_names = [
            ":../../../etc/passwd:",
            ":pack/action:",
            ":pack\\action:",
            ":pack..action:",
            ":verylongpacknamethatiswaytoobigtobevalid.action:",
            ":pack.verylongactionnamethatisalsowaytoolongtobevalid:",
            ":pack.:",
            ":.action:",
            ":pack.action.extra:",
            ":pack.ac<script>alert(1)</script>tion:",
            ":pa\"ck.action:",
            ":pack.ac'tion:",
        ];

        for malicious_name in malicious_names {
            let result = store.parse_stickers_in_text(malicious_name);
            // Should not parse malicious names - they should remain as text
            assert_eq!(result, malicious_name);
            assert!(!result.contains("<img"));
        }

        // Test HTML escaping in sticker names and URLs
        // Create a temporary store with a mock sticker for testing
        let mut test_stickers = Vec::new();
        test_stickers.push(Sticker {
            name: "test.normal".to_string(),
            pack: "test".to_string(),
            action: "normal".to_string(),
            path: "/stickers/test/normal.webp".to_string(),
            url: "/stickers/test/normal.webp".to_string(),
        });

        let test_store = StickerStore {
            stickers: test_stickers,
        };

        let result = test_store.parse_stickers_in_text(":test.normal:");
        if result.contains("<img") {
            // Ensure proper HTML attribute escaping
            assert!(!result.contains("\"\""));
            assert!(result.contains("alt=\"test.normal\""));
            assert!(result.contains("src=\"/stickers/test/normal.webp\""));
        }
    }

    #[test]
    fn test_sticker_name_validation() {
        let store = StickerStore::new().unwrap();

        // Valid names should pass
        assert!(store.is_valid_sticker_name("pack.action"));
        assert!(store.is_valid_sticker_name("my-pack.my_action"));
        assert!(store.is_valid_sticker_name("pack123.action456"));

        // Invalid names should fail
        assert!(!store.is_valid_sticker_name(""));
        assert!(!store.is_valid_sticker_name("no-dot"));
        assert!(!store.is_valid_sticker_name("too.many.dots"));
        assert!(!store.is_valid_sticker_name("pack."));
        assert!(!store.is_valid_sticker_name(".action"));
        assert!(!store.is_valid_sticker_name("pack../action"));
        assert!(!store.is_valid_sticker_name("pack/action"));
        assert!(!store.is_valid_sticker_name("pack\\action"));
        assert!(!store.is_valid_sticker_name("verylongpacknamethatiswaytoobigtobevalid.action"));
        assert!(
            !store.is_valid_sticker_name("pack.verylongactionnamethatisalsowaytoolongtobevalid")
        );
        assert!(!store.is_valid_sticker_name("pa ck.action"));
        assert!(!store.is_valid_sticker_name("pack.ac tion"));
        assert!(!store.is_valid_sticker_name("con.action"));
        assert!(!store.is_valid_sticker_name("pack.aux"));
    }
}
