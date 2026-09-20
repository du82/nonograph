use crate::parser::html_attr_escape;
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

        // Resolve `{{#if key}}...{{/if}}` blocks before variable substitution.
        // A block is kept when the context contains `key` with a truthy value
        // ("true" or "1"), and removed entirely otherwise. This keeps the engine
        // simple while letting templates hide optional sections.
        result = resolve_conditionals(&result, context);

        // Inject the writemark.js editor source when requested, so templates can
        // embed the editor inline via `{{writemark_js}}`. The file ships as an ES
        // module (it ends with an `export { ... }` statement), but we inline it
        // into a classic `<script>` tag where `export` is a syntax error that
        // would abort the whole script. Strip any top-level `export` statements
        // before injecting; the element self-registers via `customElements.define`,
        // so the exports are unnecessary for inline browser use.
        if result.contains("{{writemark_js}}") {
            let script_path = Path::new(&self.templates_dir).join("writemark.js");
            let script = fs::read_to_string(&script_path).map_err(|e| {
                format!(
                    "Failed to read writemark.js for template {}: {}",
                    template_name, e
                )
            })?;
            let script = strip_module_exports_and_line_comments(&script);
            result = result.replace("{{writemark_js}}", &script);
        }

        // Replace all {{variable}} patterns with values from context
        for (key, value) in context {
            let pattern = format!("{{{{{}}}}}", key);
            let escaped_value = if key == "content" {
                value.clone()
            } else {
                html_attr_escape(value)
            };
            result = result.replace(&pattern, &escaped_value);
        }

        // Check for any remaining unreplaced variables and warn
        if result.contains("{{") && result.contains("}}") {
            eprintln!(
                "Nonograph: Warning: Template {} contains unreplaced variables",
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
        full_context.insert("title".to_string(), "Nonograph".to_string());
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

/// Resolve `{{#if key}}...{{/if}}` conditional blocks.
///
/// The block body is kept only when `context[key]` is truthy ("true" or "1").
/// Blocks are non-nested; the search restarts after each fully processed block.
fn resolve_conditionals(template: &str, context: &HashMap<String, String>) -> String {
    let mut result = template.to_string();

    loop {
        let Some(open_start) = result.find("{{#if ") else {
            break;
        };
        let after_open = open_start + "{{#if ".len();
        let Some(rel_open_end) = result[after_open..].find("}}") else {
            break;
        };
        let open_end = after_open + rel_open_end;
        let key = result[after_open..open_end].trim().to_string();
        let body_start = open_end + "}}".len();

        let Some(rel_close) = result[body_start..].find("{{/if}}") else {
            break;
        };
        let close_start = body_start + rel_close;
        let block_end = close_start + "{{/if}}".len();

        let keep = matches!(
            context.get(&key).map(|v| v.as_str()),
            Some("true") | Some("1")
        );

        let replacement = if keep {
            result[body_start..close_start].to_string()
        } else {
            String::new()
        };

        result.replace_range(open_start..block_end, &replacement);
    }

    result
}

fn strip_module_exports_and_line_comments(source: &str) -> String {
    source
        .lines()
        .filter(|line| {
            let trimmed = line.trim_start();
            !trimmed.starts_with("export ") && !trimmed.starts_with("//")
        })
        .collect::<Vec<_>>()
        .join("\n")
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
        assert_eq!(result, "<title>Nonograph</title><div>Custom content</div>");
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

    #[test]
    fn test_conditional_blocks() {
        let dir = tempdir().unwrap();
        let templates_path = dir.path().to_str().unwrap();

        let template_content =
            "<ul>{{#if show_a}}<li>A</li>{{/if}}{{#if show_b}}<li>B</li>{{/if}}</ul>";
        fs::write(dir.path().join("test.html"), template_content).unwrap();

        let engine = TemplateEngine::new(templates_path);
        let mut context = HashMap::new();
        context.insert("show_a".to_string(), "true".to_string());
        context.insert("show_b".to_string(), "false".to_string());

        let result = engine.render("test", &context).unwrap();
        assert_eq!(result, "<ul><li>A</li></ul>");
    }

    #[test]
    fn test_conditional_block_missing_key_is_removed() {
        let dir = tempdir().unwrap();
        let templates_path = dir.path().to_str().unwrap();

        let template_content = "<div>{{#if maybe}}kept{{/if}}done</div>";
        fs::write(dir.path().join("test.html"), template_content).unwrap();

        let engine = TemplateEngine::new(templates_path);
        let context = HashMap::new();

        let result = engine.render("test", &context).unwrap();
        assert_eq!(result, "<div>done</div>");
    }

    #[test]
    fn test_strip_module_exports_and_line_comments() {
        let source = "\
// full line comment
  // indented full line comment
const re = /https:\\/\\//g; // trailing comment stays
const url = \"https://example.com\";
export { Thing };
  export default Thing;
const x = 1;";

        let result = strip_module_exports_and_line_comments(source);

        // Full-line comments (including indented ones) are dropped.
        assert!(!result.contains("full line comment"));
        assert!(!result.contains("indented full line comment"));
        // Top-level exports are dropped.
        assert!(!result.contains("export"));
        // Code with `//` inside a regex or string is preserved verbatim.
        assert!(result.contains("const re = /https:\\/\\//g; // trailing comment stays"));
        assert!(result.contains("const url = \"https://example.com\";"));
        assert!(result.contains("const x = 1;"));
    }
}
