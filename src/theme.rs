use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    pub name: String,
    pub background: String,
    pub text: String,
    pub is_dark: bool,
    pub button_bg: String,
    pub button_hover: String,
    pub button_active: String,
    pub button_text: String,
    pub menu_bg: String,
    pub menu_text: String,
    pub menu_hover: String,
    pub menu_selected: String,
    pub menu_border: String,
    pub menu_shadow: String,
}

#[derive(Debug, Deserialize)]
struct ThemesConfig {
    themes: HashMap<String, Theme>,
}

#[derive(Debug)]
pub struct ThemeManager {
    themes: HashMap<String, Theme>,
    default_theme: String,
}

impl ThemeManager {
    /// Create a new ThemeManager by loading themes from the Themes.toml file
    pub fn new<P: AsRef<Path>>(config_path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let config_content = fs::read_to_string(config_path)?;
        let config: ThemesConfig = toml::from_str(&config_content)?;

        Ok(ThemeManager {
            themes: config.themes,
            default_theme: "light".to_string(),
        })
    }

    /// Get a theme by name
    pub fn get_theme(&self, name: &str) -> Option<&Theme> {
        self.themes.get(name)
    }

    /// Get the default theme
    pub fn get_default_theme(&self) -> &Theme {
        self.themes
            .get(&self.default_theme)
            .expect("Default theme should always exist")
    }

    /// Get all available theme names
    pub fn get_theme_names(&self) -> Vec<String> {
        self.themes.keys().cloned().collect()
    }

    /// Get all themes
    pub fn get_all_themes(&self) -> &HashMap<String, Theme> {
        &self.themes
    }

    /// Validate if a theme name exists
    pub fn is_valid_theme(&self, name: &str) -> bool {
        self.themes.contains_key(name)
    }

    /// Get light themes only
    pub fn get_light_themes(&self) -> Vec<(&String, &Theme)> {
        self.themes
            .iter()
            .filter(|(_, theme)| !theme.is_dark)
            .collect()
    }

    /// Get dark themes only
    pub fn get_dark_themes(&self) -> Vec<(&String, &Theme)> {
        self.themes
            .iter()
            .filter(|(_, theme)| theme.is_dark)
            .collect()
    }

    /// Set the default theme
    pub fn set_default_theme(&mut self, theme_name: String) -> Result<(), String> {
        if self.is_valid_theme(&theme_name) {
            self.default_theme = theme_name;
            Ok(())
        } else {
            Err(format!("Theme '{}' does not exist", theme_name))
        }
    }

    /// Convert theme index to theme name based on frontend theme array order
    /// This matches the order in home.html themes array
    pub fn index_to_theme_name(&self, index: &str) -> Option<String> {
        // Frontend theme order matching home.html themes array
        let theme_order = vec![
            "light",
            "sepia",
            "rose",
            "peach",
            "cream",
            "mint",
            "sage",
            "sky",
            "lavender",
            "dark",
            "black",
            "slate",
            "burgundy",
            "rust",
            "warm_coffee",
            "forest_green",
            "olive",
            "midnight_blue",
            "steel_blue",
            "deep_teal",
            "deep_purple",
            "plum",
        ];

        if let Ok(idx) = index.parse::<usize>() {
            if idx < theme_order.len() {
                Some(theme_order[idx].to_string())
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Validate theme input - accepts both theme names and indices
    pub fn is_valid_theme_input(&self, input: &str) -> Option<String> {
        // First try as theme name directly
        if self.is_valid_theme(input) {
            return Some(input.to_string());
        }

        // Then try as index
        if let Some(theme_name) = self.index_to_theme_name(input) {
            if self.is_valid_theme(&theme_name) {
                return Some(theme_name);
            }
        }

        None
    }

    /// Get themes in frontend-compatible JSON format
    pub fn get_frontend_themes(&self) -> serde_json::Value {
        let mut themes_array = Vec::new();

        // Order must match the frontend theme array order
        let theme_order = vec![
            "light",
            "sepia",
            "rose",
            "peach",
            "cream",
            "mint",
            "sage",
            "sky",
            "lavender",
            "dark",
            "black",
            "slate",
            "burgundy",
            "rust",
            "warm_coffee",
            "forest_green",
            "olive",
            "midnight_blue",
            "steel_blue",
            "deep_teal",
            "deep_purple",
            "plum",
        ];

        for theme_key in theme_order {
            if let Some(theme) = self.themes.get(theme_key) {
                themes_array.push(serde_json::json!({
                    "name": theme.name,
                    "bg": theme.background,
                    "text": theme.text,
                    "buttonBg": theme.button_bg,
                    "buttonHover": theme.button_hover,
                    "buttonActive": theme.button_active,
                    "buttonText": theme.button_text,
                    "menuBg": theme.menu_bg,
                    "menuText": theme.menu_text,
                    "menuHover": theme.menu_hover,
                    "menuSelected": theme.menu_selected,
                    "menuBorder": theme.menu_border,
                    "menuShadow": theme.menu_shadow
                }));
            }
        }

        serde_json::Value::Array(themes_array)
    }
}

impl Theme {
    /// Validate that all color values are valid hex colors
    pub fn validate_colors(&self) -> Result<(), String> {
        let colors = vec![
            (&self.background, "background"),
            (&self.text, "text"),
            (&self.button_bg, "button_bg"),
            (&self.button_hover, "button_hover"),
            (&self.button_active, "button_active"),
            (&self.button_text, "button_text"),
            (&self.menu_bg, "menu_bg"),
            (&self.menu_text, "menu_text"),
            (&self.menu_hover, "menu_hover"),
            (&self.menu_selected, "menu_selected"),
            (&self.menu_border, "menu_border"),
        ];

        for (color, field_name) in colors {
            if !is_valid_hex_color(color) && !is_valid_rgba_color(color) {
                return Err(format!(
                    "Invalid color '{}' for field '{}'",
                    color, field_name
                ));
            }
        }

        // Special validation for menu_shadow which can be rgba
        if !is_valid_rgba_color(&self.menu_shadow) && !is_valid_hex_color(&self.menu_shadow) {
            return Err(format!(
                "Invalid color '{}' for field 'menu_shadow'",
                self.menu_shadow
            ));
        }

        Ok(())
    }

    /// Convert theme to CSS variables string
    pub fn to_css_variables(&self) -> String {
        format!(
            ":root {{\n\
            \t--button-bg: {};\n\
            \t--button-hover: {};\n\
            \t--button-active: {};\n\
            \t--button-text: {};\n\
            \t--menu-bg: {};\n\
            \t--menu-text: {};\n\
            \t--menu-hover: {};\n\
            \t--menu-selected: {};\n\
            \t--menu-border: {};\n\
            \t--menu-shadow: {};\n\
            \t--background: {};\n\
            \t--text: {};\n\
            }}",
            self.button_bg,
            self.button_hover,
            self.button_active,
            self.button_text,
            self.menu_bg,
            self.menu_text,
            self.menu_hover,
            self.menu_selected,
            self.menu_border,
            self.menu_shadow,
            self.background,
            self.text
        )
    }

    /// Create a theme JSON object for frontend consumption
    pub fn to_json_object(&self) -> serde_json::Value {
        serde_json::json!({
            "name": self.name,
            "background": self.background,
            "text": self.text,
            "isDark": self.is_dark,
            "buttonBg": self.button_bg,
            "buttonHover": self.button_hover,
            "buttonActive": self.button_active,
            "buttonText": self.button_text,
            "menuBg": self.menu_bg,
            "menuText": self.menu_text,
            "menuHover": self.menu_hover,
            "menuSelected": self.menu_selected,
            "menuBorder": self.menu_border,
            "menuShadow": self.menu_shadow
        })
    }
}

/// Validate if a string is a valid hex color (e.g., #ffffff, #fff)
fn is_valid_hex_color(color: &str) -> bool {
    if !color.starts_with('#') {
        return false;
    }

    let hex_part = &color[1..];
    if hex_part.len() != 3 && hex_part.len() != 6 {
        return false;
    }

    hex_part.chars().all(|c| c.is_ascii_hexdigit())
}

/// Validate if a string is a valid rgba color (e.g., rgba(0, 0, 0, 0.5))
fn is_valid_rgba_color(color: &str) -> bool {
    if !color.starts_with("rgba(") || !color.ends_with(')') {
        return false;
    }

    let inner = &color[5..color.len() - 1];
    let parts: Vec<&str> = inner.split(',').map(|s| s.trim()).collect();

    if parts.len() != 4 {
        return false;
    }

    // Check RGB values (0-255)
    for i in 0..3 {
        if let Ok(val) = parts[i].parse::<u8>() {
            if val > 255 {
                return false;
            }
        } else {
            return false;
        }
    }

    // Check alpha value (0.0-1.0)
    if let Ok(alpha) = parts[3].parse::<f64>() {
        alpha >= 0.0 && alpha <= 1.0
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hex_color_validation() {
        assert!(is_valid_hex_color("#ffffff"));
        assert!(is_valid_hex_color("#000000"));
        assert!(is_valid_hex_color("#fff"));
        assert!(is_valid_hex_color("#123abc"));

        assert!(!is_valid_hex_color("ffffff"));
        assert!(!is_valid_hex_color("#gggggg"));
        assert!(!is_valid_hex_color("#ffff"));
        assert!(!is_valid_hex_color(""));
    }

    #[test]
    fn test_rgba_color_validation() {
        assert!(is_valid_rgba_color("rgba(0, 0, 0, 0.5)"));
        assert!(is_valid_rgba_color("rgba(255, 255, 255, 1.0)"));
        assert!(is_valid_rgba_color("rgba(128, 64, 32, 0.0)"));

        assert!(!is_valid_rgba_color("rgb(0, 0, 0)"));
        assert!(!is_valid_rgba_color("rgba(256, 0, 0, 0.5)"));
        assert!(!is_valid_rgba_color("rgba(0, 0, 0, 1.5)"));
        assert!(!is_valid_rgba_color("rgba(0, 0, 0)"));
    }

    #[test]
    fn test_theme_validation() {
        let theme = Theme {
            name: "Test".to_string(),
            background: "#ffffff".to_string(),
            text: "#000000".to_string(),
            is_dark: false,
            button_bg: "#333333".to_string(),
            button_hover: "#000000".to_string(),
            button_active: "#111111".to_string(),
            button_text: "#ffffff".to_string(),
            menu_bg: "#ffffff".to_string(),
            menu_text: "#333333".to_string(),
            menu_hover: "#f5f5f5".to_string(),
            menu_selected: "#e8e8e8".to_string(),
            menu_border: "#dddddd".to_string(),
            menu_shadow: "rgba(0, 0, 0, 0.15)".to_string(),
        };

        assert!(theme.validate_colors().is_ok());
    }

    #[test]
    fn test_theme_css_variables() {
        let theme = Theme {
            name: "Test".to_string(),
            background: "#ffffff".to_string(),
            text: "#000000".to_string(),
            is_dark: false,
            button_bg: "#333333".to_string(),
            button_hover: "#000000".to_string(),
            button_active: "#111111".to_string(),
            button_text: "#ffffff".to_string(),
            menu_bg: "#ffffff".to_string(),
            menu_text: "#333333".to_string(),
            menu_hover: "#f5f5f5".to_string(),
            menu_selected: "#e8e8e8".to_string(),
            menu_border: "#dddddd".to_string(),
            menu_shadow: "rgba(0, 0, 0, 0.15)".to_string(),
        };

        let css = theme.to_css_variables();
        assert!(css.contains("--button-bg: #333333;"));
        assert!(css.contains("--background: #ffffff;"));
        assert!(css.contains("--menu-shadow: rgba(0, 0, 0, 0.15);"));
    }

    #[test]
    fn test_index_to_theme_name() {
        let theme_manager = ThemeManager::new("Themes.toml").unwrap();

        // Test valid indices
        assert_eq!(
            theme_manager.index_to_theme_name("0"),
            Some("light".to_string())
        );
        assert_eq!(
            theme_manager.index_to_theme_name("1"),
            Some("sepia".to_string())
        );
        assert_eq!(
            theme_manager.index_to_theme_name("9"),
            Some("dark".to_string())
        );
        assert_eq!(
            theme_manager.index_to_theme_name("10"),
            Some("black".to_string())
        );

        // Test invalid indices
        assert_eq!(theme_manager.index_to_theme_name("100"), None);
        assert_eq!(theme_manager.index_to_theme_name("-1"), None);
        assert_eq!(theme_manager.index_to_theme_name("invalid"), None);
    }

    #[test]
    fn test_is_valid_theme_input() {
        let theme_manager = ThemeManager::new("Themes.toml").unwrap();

        // Test theme name directly
        assert_eq!(
            theme_manager.is_valid_theme_input("light"),
            Some("light".to_string())
        );
        assert_eq!(
            theme_manager.is_valid_theme_input("dark"),
            Some("dark".to_string())
        );

        // Test theme index
        assert_eq!(
            theme_manager.is_valid_theme_input("0"),
            Some("light".to_string())
        );
        assert_eq!(
            theme_manager.is_valid_theme_input("9"),
            Some("dark".to_string())
        );

        // Test invalid inputs
        assert_eq!(theme_manager.is_valid_theme_input("nonexistent"), None);
        assert_eq!(theme_manager.is_valid_theme_input("100"), None);
        assert_eq!(theme_manager.is_valid_theme_input(""), None);
    }

    #[test]
    fn test_get_frontend_themes() {
        let theme_manager = ThemeManager::new("Themes.toml").unwrap();
        let frontend_themes = theme_manager.get_frontend_themes();

        // Should be an array
        assert!(frontend_themes.is_array());
        let themes_array = frontend_themes.as_array().unwrap();

        // Should have the correct number of themes
        assert_eq!(themes_array.len(), 22); // 9 light + 13 dark themes

        // Check first theme (light)
        let first_theme = &themes_array[0];
        assert_eq!(first_theme["name"], "Light");
        assert_eq!(first_theme["bg"], "#ffffff");
        assert_eq!(first_theme["text"], "#1a1a1a");
        assert!(first_theme["buttonBg"].is_string());

        // Check a dark theme (index 9 should be dark)
        let dark_theme = &themes_array[9];
        assert_eq!(dark_theme["name"], "Dark");
        assert_eq!(dark_theme["bg"], "#1a1f25");
        assert_eq!(dark_theme["text"], "#e0e4e8");
    }
}
