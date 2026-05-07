use tera::{Context, Tera};

pub struct TemplateEngine {
    tera: Tera,
}

impl TemplateEngine {
    pub fn new(template_dir: &str) -> Result<Self, String> {
        let tera = Tera::new(&format!("{}/**/*", template_dir))
            .map_err(|e| format!("Failed to init Tera: {}", e))?;
        Ok(TemplateEngine { tera })
    }

    pub fn render(&self, template_name: &str, context: &Context) -> Result<String, String> {
        self.tera
            .render(template_name, context)
            .map_err(|e| format!("Template render error: {}", e))
    }
}
