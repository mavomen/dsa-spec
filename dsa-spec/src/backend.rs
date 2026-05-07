use crate::ast::Spec;

pub trait Backend {
    fn generate(&self, spec: &Spec) -> Result<String, String>;
}
