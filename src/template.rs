use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub struct TemplateEngine {
    templates_dir: String,
}

impl TemplateEngine {
    pub fn new(templates_dir: &str) -> Self {
        Self {
            templates_dir: templates_dir.to_string(),
        }
    }

    pub fn render(
        &self,
        template_name: &str,
        context: &HashMap<String, String>,
    ) -> Result<String, String> {
        let template_path = Path::new(&self.templates_dir).join(format!("{}.html", template_name));

        let template_content = fs::read_to_string(&template_path)
            .map_err(|e| format!("Failed to read template {}: {}", template_name, e))?;

        let mut result = template_content;

        // Replace all {{variable}} patterns with values from context
        for (key, value) in context {
            let pattern = format!("{{{{{}}}}}", key);
            result = result.replace(&pattern, value);
        }

        // Check for any remaining unreplaced variables and warn
        if result.contains("{{") && result.contains("}}") {
            eprintln!(
                "Warning: Template {} contains unreplaced variables",
                template_name
            );
        }

        Ok(result)
    }

    pub fn render_with_defaults(
        &self,
        template_name: &str,
        context: &HashMap<String, String>,
    ) -> Result<String, String> {
        let mut full_context = HashMap::new();

        // Set default values
        full_context.insert("title".to_string(), "Telegraph-rs".to_string());
        full_context.insert("content".to_string(), "".to_string());
        full_context.insert("error".to_string(), "".to_string());
        full_context.insert("success".to_string(), "".to_string());

        // Override with provided context
        for (key, value) in context {
            full_context.insert(key.clone(), value.clone());
        }

        self.render(template_name, &full_context)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_simple_template_rendering() {
        let dir = tempdir().unwrap();
        let templates_path = dir.path().to_str().unwrap();

        // Create a test template
        let template_content = "<h1>{{title}}</h1><p>{{content}}</p>";
        fs::write(dir.path().join("test.html"), template_content).unwrap();

        let engine = TemplateEngine::new(templates_path);
        let mut context = HashMap::new();
        context.insert("title".to_string(), "Hello World".to_string());
        context.insert("content".to_string(), "This is content".to_string());

        let result = engine.render("test", &context).unwrap();
        assert_eq!(result, "<h1>Hello World</h1><p>This is content</p>");
    }

    #[test]
    fn test_template_with_missing_variables() {
        let dir = tempdir().unwrap();
        let templates_path = dir.path().to_str().unwrap();

        let template_content = "<h1>{{title}}</h1><p>{{missing}}</p>";
        fs::write(dir.path().join("test.html"), template_content).unwrap();

        let engine = TemplateEngine::new(templates_path);
        let mut context = HashMap::new();
        context.insert("title".to_string(), "Hello".to_string());

        let result = engine.render("test", &context).unwrap();
        assert_eq!(result, "<h1>Hello</h1><p>{{missing}}</p>");
    }

    #[test]
    fn test_render_with_defaults() {
        let dir = tempdir().unwrap();
        let templates_path = dir.path().to_str().unwrap();

        let template_content = "<title>{{title}}</title><div>{{content}}</div>";
        fs::write(dir.path().join("page.html"), template_content).unwrap();

        let engine = TemplateEngine::new(templates_path);
        let mut context = HashMap::new();
        context.insert("content".to_string(), "Custom content".to_string());

        let result = engine.render_with_defaults("page", &context).unwrap();
        assert_eq!(
            result,
            "<title>Telegraph-rs</title><div>Custom content</div>"
        );
    }

    #[test]
    fn test_html_escaping_not_performed() {
        let dir = tempdir().unwrap();
        let templates_path = dir.path().to_str().unwrap();

        let template_content = "<div>{{content}}</div>";
        fs::write(dir.path().join("test.html"), template_content).unwrap();

        let engine = TemplateEngine::new(templates_path);
        let mut context = HashMap::new();
        context.insert(
            "content".to_string(),
            "<script>alert('xss')</script>".to_string(),
        );

        let result = engine.render("test", &context).unwrap();
        // Note: Our simple template engine doesn't escape HTML - this will be handled by ammonia
        assert_eq!(result, "<div><script>alert('xss')</script></div>");
    }
}
