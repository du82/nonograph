use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sticker {
    pub name: String,
    pub tags: Vec<String>,
    pub base64: String,
}

#[derive(Debug, Deserialize)]
struct StickersConfig {
    sticker: Vec<Sticker>,
}

#[derive(Debug, Clone)]
pub struct StickerStore {
    stickers: Vec<Sticker>,
}

impl StickerStore {
    pub fn new() -> Result<Self, String> {
        let stickers_path = Path::new("Stickers.toml");

        if !stickers_path.exists() {
            return Ok(StickerStore {
                stickers: Vec::new(),
            });
        }

        let content = fs::read_to_string(stickers_path)
            .map_err(|e| format!("Failed to read Stickers.toml: {}", e))?;

        let config: StickersConfig = toml::from_str(&content)
            .map_err(|e| format!("Failed to parse Stickers.toml: {}", e))?;

        Ok(StickerStore {
            stickers: config.sticker,
        })
    }

    pub fn search(&self, query: &str) -> Vec<&Sticker> {
        if query.is_empty() {
            return self.stickers.iter().collect();
        }

        let query_lower = query.to_lowercase();

        self.stickers
            .iter()
            .filter(|sticker| {
                // Search in sticker name
                sticker.name.to_lowercase().contains(&query_lower) ||
                // Search in tags
                sticker.tags.iter().any(|tag| tag.to_lowercase().contains(&query_lower))
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_test_stickers_file() -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(
            file,
            r#"
[[sticker]]
name = "marsey.happy"
tags = ["happy", "emotion", "face", "marsey"]
base64 = "data:image/png;base64,test1"

[[sticker]]
name = "pepe.sad"
tags = ["sad", "emotion", "pepe"]
base64 = "data:image/png;base64,test2"

[[sticker]]
name = "react.fire"
tags = ["hot", "amazing", "cool", "react"]
base64 = "data:image/png;base64,test3"
"#
        )
        .unwrap();
        file
    }

    #[test]
    fn test_sticker_store_creation() {
        // Test with non-existent file by temporarily changing directory
        let original_dir = std::env::current_dir().unwrap();
        let temp_dir = tempfile::tempdir().unwrap();
        std::env::set_current_dir(temp_dir.path()).unwrap();

        let store = StickerStore::new().unwrap();
        assert_eq!(store.count(), 0);

        // Restore original directory
        std::env::set_current_dir(original_dir).unwrap();
    }

    #[test]
    fn test_search_functionality() {
        let stickers = vec![
            Sticker {
                name: "marsey.happy".to_string(),
                tags: vec![
                    "happy".to_string(),
                    "emotion".to_string(),
                    "marsey".to_string(),
                ],
                base64: "data:image/png;base64,test1".to_string(),
            },
            Sticker {
                name: "pepe.sad".to_string(),
                tags: vec!["sad".to_string(), "emotion".to_string(), "pepe".to_string()],
                base64: "data:image/png;base64,test2".to_string(),
            },
            Sticker {
                name: "react.fire".to_string(),
                tags: vec![
                    "hot".to_string(),
                    "amazing".to_string(),
                    "react".to_string(),
                ],
                base64: "data:image/png;base64,test3".to_string(),
            },
        ];

        let store = StickerStore { stickers };

        // Test empty query returns all
        let results = store.search("");
        assert_eq!(results.len(), 3);

        // Test pack name search
        let results = store.search("marsey");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "marsey.happy");

        // Test partial pack name search
        let results = store.search("pepe");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "pepe.sad");

        // Test tag search
        let results = store.search("emotion");
        assert_eq!(results.len(), 2); // marsey.happy and pepe.sad both have emotion tag

        // Test case insensitive search
        let results = store.search("HAPPY");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "marsey.happy");

        // Test no matches
        let results = store.search("nonexistent");
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_get_by_name() {
        let stickers = vec![Sticker {
            name: "marsey.happy".to_string(),
            tags: vec!["happy".to_string(), "marsey".to_string()],
            base64: "data:image/png;base64,test1".to_string(),
        }];

        let store = StickerStore { stickers };

        let result = store.get_by_name("marsey.happy");
        assert!(result.is_some());
        assert_eq!(result.unwrap().name, "marsey.happy");

        let result = store.get_by_name("nonexistent");
        assert!(result.is_none());
    }

    #[test]
    fn test_packname_action_search() {
        let stickers = vec![
            Sticker {
                name: "marsey.happy".to_string(),
                tags: vec!["happy".to_string(), "marsey".to_string()],
                base64: "data:image/png;base64,test1".to_string(),
            },
            Sticker {
                name: "marsey.crying".to_string(),
                tags: vec!["sad".to_string(), "marsey".to_string()],
                base64: "data:image/png;base64,test2".to_string(),
            },
            Sticker {
                name: "pepe.smug".to_string(),
                tags: vec!["smug".to_string(), "pepe".to_string()],
                base64: "data:image/png;base64,test3".to_string(),
            },
        ];

        let store = StickerStore { stickers };

        // Test searching by pack name should find all stickers in that pack
        let results = store.search("marsey");
        assert_eq!(results.len(), 2);

        let results = store.search("pepe");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "pepe.smug");

        // Test searching by action should find stickers with that action
        let results = store.search("happy");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "marsey.happy");
    }
}
