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
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_stickers_structure() -> TempDir {
        let temp_dir = TempDir::new().unwrap();
        let stickers_dir = temp_dir.path().join("stickers");

        // Create marsey pack
        let marsey_dir = stickers_dir.join("marsey");
        fs::create_dir_all(&marsey_dir).unwrap();
        fs::write(marsey_dir.join("happy.png"), b"fake png data").unwrap();
        fs::write(marsey_dir.join("crying.png"), b"fake png data").unwrap();

        // Create pepe pack
        let pepe_dir = stickers_dir.join("pepe");
        fs::create_dir_all(&pepe_dir).unwrap();
        fs::write(pepe_dir.join("smug.png"), b"fake png data").unwrap();
        fs::write(pepe_dir.join("sad.png"), b"fake png data").unwrap();

        temp_dir
    }

    #[test]
    fn test_sticker_store_creation() {
        // Test with non-existent directory
        let temp_dir = TempDir::new().unwrap();
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp_dir.path()).unwrap();

        let store = StickerStore::new().unwrap();
        assert_eq!(store.count(), 0);

        std::env::set_current_dir(original_dir).unwrap();
    }

    #[test]
    fn test_filesystem_scanning() {
        let temp_dir = create_test_stickers_structure();
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp_dir.path()).unwrap();

        let store = StickerStore::new().unwrap();
        assert_eq!(store.count(), 4); // 2 marsey + 2 pepe stickers

        // Check that stickers are properly named
        let marsey_happy = store.get_by_name("marsey.happy");
        assert!(marsey_happy.is_some());
        assert_eq!(marsey_happy.unwrap().pack, "marsey");
        assert_eq!(marsey_happy.unwrap().action, "happy");
        assert_eq!(marsey_happy.unwrap().url, "/stickers/marsey/happy.png");

        std::env::set_current_dir(original_dir).unwrap();
    }

    #[test]
    fn test_search_functionality() {
        let temp_dir = create_test_stickers_structure();
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp_dir.path()).unwrap();

        let store = StickerStore::new().unwrap();

        // Test empty query returns all
        let results = store.search("");
        assert_eq!(results.len(), 4);

        // Test pack search
        let results = store.search("marsey");
        assert_eq!(results.len(), 2);
        assert!(results.iter().all(|s| s.pack == "marsey"));

        // Test action search
        let results = store.search("happy");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "marsey.happy");

        // Test case insensitive search
        let results = store.search("PEPE");
        assert_eq!(results.len(), 2);
        assert!(results.iter().all(|s| s.pack == "pepe"));

        // Test no matches
        let results = store.search("nonexistent");
        assert_eq!(results.len(), 0);

        std::env::set_current_dir(original_dir).unwrap();
    }

    #[test]
    fn test_get_by_name() {
        let temp_dir = create_test_stickers_structure();
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp_dir.path()).unwrap();

        let store = StickerStore::new().unwrap();

        let result = store.get_by_name("marsey.happy");
        assert!(result.is_some());
        assert_eq!(result.unwrap().pack, "marsey");
        assert_eq!(result.unwrap().action, "happy");

        let result = store.get_by_name("nonexistent.sticker");
        assert!(result.is_none());

        std::env::set_current_dir(original_dir).unwrap();
    }
}
