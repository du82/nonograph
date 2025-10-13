use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub limits: Limits,
    pub server: Server,
    pub cache: Cache,
    pub performance: Performance,
    pub security: Security,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Limits {
    pub title_max_length: usize,
    pub alias_max_length: usize,
    pub content_max_length: usize,
    pub form_data_limit_kb: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Server {
    pub port: u16,
    pub address: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cache {
    pub max_cache_size_mb: usize,
    pub stream_buffer_size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Performance {
    pub large_content_threshold: usize,
    pub streaming_threshold: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Security {
    pub max_url_length: usize,
    pub external_link_security: bool,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            limits: Limits {
                title_max_length: 128,
                alias_max_length: 32,
                content_max_length: 128000,
                form_data_limit_kb: 512,
            },
            server: Server {
                port: 8000,
                address: "127.0.0.1".to_string(),
            },
            cache: Cache {
                max_cache_size_mb: 128,
                stream_buffer_size: 8192,
            },
            performance: Performance {
                large_content_threshold: 30000,
                streaming_threshold: 50000,
            },
            security: Security {
                max_url_length: 4096,
                external_link_security: true,
            },
        }
    }
}

impl Config {
    pub fn load() -> Result<Self, String> {
        let config_path = Path::new("Config.toml");

        if config_path.exists() {
            let content = fs::read_to_string(config_path)
                .map_err(|e| format!("Failed to read Config.toml: {}", e))?;

            toml::from_str(&content).map_err(|e| format!("Failed to parse Config.toml: {}", e))
        } else {
            Ok(Config::default())
        }
    }

    pub fn load_with_logging() -> Self {
        match Self::load() {
            Ok(config) => {
                println!("✅ Configuration loaded successfully");
                println!("   Title limit: {} chars", config.limits.title_max_length);
                println!("   Alias limit: {} chars", config.limits.alias_max_length);
                println!(
                    "   Content limit: {} chars",
                    config.limits.content_max_length
                );
                println!("   Cache size: {} MB", config.cache.max_cache_size_mb);
                config
            }
            Err(e) => {
                eprintln!("⚠️  Configuration error: {}", e);
                eprintln!("   Using default configuration");
                Config::default()
            }
        }
    }

    pub fn form_data_limit_bytes(&self) -> u32 {
        self.limits.form_data_limit_kb * 1024
    }

    pub fn validate_post(
        &self,
        title: &str,
        content: &str,
        alias: Option<&str>,
    ) -> Result<(), String> {
        if title.trim().is_empty() {
            return Err("title_required".to_string());
        }

        if content.trim().is_empty() {
            return Err("content_required".to_string());
        }

        if title.len() > self.limits.title_max_length {
            return Err("title_too_long".to_string());
        }

        if content.len() > self.limits.content_max_length {
            return Err("content_too_long".to_string());
        }

        if let Some(alias) = alias {
            if alias.len() > self.limits.alias_max_length {
                return Err("alias_too_long".to_string());
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.limits.title_max_length, 128);
        assert_eq!(config.limits.alias_max_length, 32);
        assert_eq!(config.limits.content_max_length, 128000);
        assert_eq!(config.server.port, 8000);
    }

    #[test]
    fn test_form_data_limit_bytes() {
        let config = Config::default();
        assert_eq!(config.form_data_limit_bytes(), 512 * 1024);
    }

    #[test]
    fn test_post_validation() {
        let config = Config::default();

        // Valid post
        assert!(config
            .validate_post("Test Title", "Test content", Some("Author"))
            .is_ok());

        // Empty title
        assert_eq!(
            config.validate_post("", "Test content", None).unwrap_err(),
            "title_required"
        );

        // Empty content
        assert_eq!(
            config.validate_post("Title", "", None).unwrap_err(),
            "content_required"
        );

        // Title too long
        let long_title = "x".repeat(200);
        assert_eq!(
            config
                .validate_post(&long_title, "Content", None)
                .unwrap_err(),
            "title_too_long"
        );

        // Content too long
        let long_content = "x".repeat(130000);
        assert_eq!(
            config
                .validate_post("Title", &long_content, None)
                .unwrap_err(),
            "content_too_long"
        );

        // Alias too long
        let long_alias = "x".repeat(50);
        assert_eq!(
            config
                .validate_post("Title", "Content", Some(&long_alias))
                .unwrap_err(),
            "alias_too_long"
        );
    }
}
