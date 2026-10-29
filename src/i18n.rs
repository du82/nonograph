use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

const FALLBACK_LOCALE: &str = "en";

#[derive(Debug, Clone, Default, Deserialize)]
pub struct Strings {
    #[serde(flatten)]
    locales: HashMap<String, HashMap<String, String>>,
}

impl Strings {
    pub fn load() -> Result<Self, String> {
        let path = Path::new("Strings.toml");
        if !path.exists() {
            return Ok(Strings::default());
        }

        let content =
            fs::read_to_string(path).map_err(|e| format!("Failed to read Strings.toml: {}", e))?;
        toml::from_str(&content).map_err(|e| format!("Failed to parse Strings.toml: {}", e))
    }

    pub fn load_with_logging() -> Self {
        match Self::load() {
            Ok(strings) => {
                if strings.locales.is_empty() {
                    eprintln!("Nonograph: No Strings.toml found; homepage will not be localized");
                } else {
                    let mut names: Vec<&str> = strings.locales.keys().map(String::as_str).collect();
                    names.sort_unstable();
                    println!(
                        "Nonograph: Loaded {} homepage locale(s): {}",
                        names.len(),
                        names.join(", ")
                    );
                }
                strings
            }
            Err(e) => {
                eprintln!("Nonograph: Localization error: {}", e);
                eprintln!("Nonograph: Homepage will fall back to template defaults");
                Strings::default()
            }
        }
    }

    fn resolve_locale(&self, accept_language: Option<&str>) -> Option<&str> {
        let header = accept_language?;

        for tag in parse_accept_language(header) {
            if let Some((name, _)) = self.locales.get_key_value(&tag) {
                return Some(name);
            }
            if let Some((primary, _)) = tag.split_once('-') {
                if let Some((name, _)) = self.locales.get_key_value(primary) {
                    return Some(name);
                }
            }
        }

        None
    }

    pub fn context_for(&self, accept_language: Option<&str>) -> HashMap<String, String> {
        let mut context: HashMap<String, String> = self
            .locales
            .get(FALLBACK_LOCALE)
            .cloned()
            .unwrap_or_default();

        if let Some(name) = self.resolve_locale(accept_language) {
            if name != FALLBACK_LOCALE {
                if let Some(locale) = self.locales.get(name) {
                    for (key, value) in locale {
                        context.insert(key.clone(), value.clone());
                    }
                }
            }
        }

        context
    }
}

fn parse_accept_language(header: &str) -> Vec<String> {
    let mut entries: Vec<(f32, usize, String)> = Vec::new();

    for (index, part) in header.split(',').enumerate() {
        let mut segments = part.split(';');
        let tag = match segments.next() {
            Some(t) => t.trim().to_ascii_lowercase(),
            None => continue,
        };

        if tag.is_empty() || tag == "*" {
            continue;
        }

        let mut quality = 1.0_f32;
        for segment in segments {
            let segment = segment.trim();
            if let Some(q) = segment.strip_prefix("q=") {
                if let Ok(parsed) = q.trim().parse::<f32>() {
                    quality = parsed;
                }
            }
        }

        entries.push((quality, index, tag));
    }

    entries.sort_by(|a, b| {
        b.0.partial_cmp(&a.0)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then(a.1.cmp(&b.1))
    });

    entries.into_iter().map(|(_, _, tag)| tag).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Strings {
        let toml = r#"
[en]
publish = "Publish"
about = "About"

[zh]
publish = "发布"

[ru]
publish = "Опубликовать"
about = "О сервисе"
"#;
        toml::from_str(toml).unwrap()
    }

    #[test]
    fn parses_accept_language_by_quality() {
        let tags = parse_accept_language("en-US,en;q=0.9,zh;q=0.8");
        assert_eq!(tags, vec!["en-us", "en", "zh"]);
    }

    #[test]
    fn higher_quality_wins_regardless_of_order() {
        let tags = parse_accept_language("en;q=0.2,zh;q=0.9");
        assert_eq!(tags, vec!["zh", "en"]);
    }

    #[test]
    fn ignores_wildcards_and_blank_tags() {
        let tags = parse_accept_language("*,  ,zh");
        assert_eq!(tags, vec!["zh"]);
    }

    #[test]
    fn resolves_exact_and_primary_subtag() {
        let strings = sample();
        assert_eq!(strings.resolve_locale(Some("zh-CN")), Some("zh"));
        assert_eq!(strings.resolve_locale(Some("zh")), Some("zh"));
        assert_eq!(strings.resolve_locale(Some("ru-RU,ru;q=0.9")), Some("ru"));
    }

    #[test]
    fn unsupported_language_resolves_to_none() {
        let strings = sample();
        assert_eq!(strings.resolve_locale(Some("fr-FR,fr;q=0.9")), None);
        assert_eq!(strings.resolve_locale(None), None);
    }

    #[test]
    fn context_backfills_missing_keys_from_english() {
        let strings = sample();
        let ctx = strings.context_for(Some("zh-CN"));
        assert_eq!(ctx.get("publish").map(String::as_str), Some("发布"));
        assert_eq!(ctx.get("about").map(String::as_str), Some("About"));
    }

    #[test]
    fn context_uses_english_for_unknown_language() {
        let strings = sample();
        let ctx = strings.context_for(Some("fr"));
        assert_eq!(ctx.get("publish").map(String::as_str), Some("Publish"));
        assert_eq!(ctx.get("about").map(String::as_str), Some("About"));
    }

    #[test]
    fn missing_file_yields_empty_context() {
        let strings = Strings::default();
        assert!(strings.context_for(Some("en")).is_empty());
    }
}
