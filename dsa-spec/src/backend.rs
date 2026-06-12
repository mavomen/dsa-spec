use crate::ast::Spec;
use crate::error::BackendError;

pub trait Backend {
    fn generate(&self, spec: &Spec) -> Result<String, BackendError>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{Contracts, Metadata, Spec, Verification};
    use crate::csharp_backend::CSharpBackend;
    use crate::go_backend::GoBackend;
    use crate::python_backend::PythonBackend;
    use crate::rust_backend::RustBackend;
    use crate::typescript_backend::TypeScriptBackend;

    fn minimal_spec() -> Spec {
        Spec {
            spec_version: "1.0".into(),
            metadata: Metadata {
                name: "Minimal".into(),
                category: "test".into(),
                ..Default::default()
            },
            contracts: Contracts::default(),
            structs: vec![],
            methods: vec![],
            verification: Verification::default(),
        }
    }

    #[test]
    fn test_all_backends_implement_trait() {
        let spec = minimal_spec();
        let backends: Vec<(&str, Box<dyn Backend>)> = vec![
            ("rust", Box::new(RustBackend::new("templates").unwrap())),
            ("python", Box::new(PythonBackend::new("templates").unwrap())),
            ("csharp", Box::new(CSharpBackend::new("templates").unwrap())),
            (
                "typescript",
                Box::new(TypeScriptBackend::new("templates").unwrap()),
            ),
            ("go", Box::new(GoBackend::new("templates").unwrap())),
        ];

        for (name, backend) in &backends {
            let result = backend.generate(&spec);
            assert!(
                result.is_ok(),
                "Backend {} failed: {}",
                name,
                result.unwrap_err()
            );
            let code = result.unwrap();
            assert!(!code.is_empty(), "Backend {} produced empty code", name);
        }
    }
}
