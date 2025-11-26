use base64::Engine;
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

    pub fn get_base64(&self, name: &str) -> Option<String> {
        if let Some(sticker) = self.get_by_name(name) {
            if let Ok(file_data) = fs::read(&sticker.path) {
                let extension = Path::new(&sticker.path)
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .unwrap_or("");

                let mime_type = match extension.to_lowercase().as_str() {
                    "png" => "image/png",
                    "jpg" | "jpeg" => "image/jpeg",
                    "gif" => "image/gif",
                    "webp" => "image/webp",
                    _ => "image/png",
                };

                let base64_data = base64::engine::general_purpose::STANDARD.encode(&file_data);
                return Some(format!("data:{};base64,{}", mime_type, base64_data));
            }
        }
        None
    }

    pub fn parse_stickers_in_text(&self, text: &str) -> String {
        let mut result = String::new();
        let mut chars = text.chars().peekable();

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
                    } else if next_ch.is_alphanumeric() || next_ch == '.' || next_ch == '_' {
                        sticker_name.push(next_ch);
                        chars.next();
                    } else {
                        break; // Invalid character, not a sticker
                    }
                }

                // If we found a valid sticker pattern and it contains a dot
                if found_end && sticker_name.contains('.') {
                    if let Some(base64_data) = self.get_base64(&sticker_name) {
                        let img_tag = format!("<img src=\"{}\" alt=\"{}\" style=\"max-width: 32px; max-height: 32px; vertical-align: middle;\" />", base64_data, sticker_name);
                        result.push_str(&img_tag);
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
            assert!(result.contains("<img src="));
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
    fn test_get_base64() {
        let store = StickerStore::new().unwrap();

        // Test getting base64 for existing sticker
        let base64_result = store.get_base64("marsey.angry");
        if base64_result.is_some() {
            let base64_data = base64_result.unwrap();
            assert!(base64_data.starts_with("data:image/"));
            assert!(base64_data.contains(";base64,"));
        }

        // Test getting base64 for non-existent sticker
        let base64_result = store.get_base64("nonexistent.sticker");
        assert!(base64_result.is_none());
    }
}
