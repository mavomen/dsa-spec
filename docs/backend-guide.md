# Backend Implementation Guide

This guide explains how to add a new language backend to DSA-SPEC.

## Architecture overview

```
spec.yaml  ->  parser  ->  AST  ->  language backend  ->  generated code
```

The **AST** (in `src/ast.rs`) is language-agnostic. Each backend translates the AST into idiomatic code for one target language.

## Steps to add a backend

### 1. Create the backend module

Add a new file `src/<language>_backend.rs`. It must:

- Contain a public struct (e.g. `PythonBackend`)
- Implement the `Backend` trait defined in `src/backend.rs`
- Provide a constructor that accepts the template directory

```rust
use crate::ast::Spec;
use crate::backend::Backend;
use crate::error::BackendError;
use crate::template_engine::TemplateEngine;

impl Backend for MyBackend {
    fn generate(&self, spec: &Spec) -> Result<String, BackendError> {
        // 1. Build a Tera context from the AST
        // 2. Render the template
        // 3. Optionally pipe through a code formatter
    }
}
```

### 2. Create a Tera template

Place it in `templates/<language>.<ext>.tera`. The template receives the context built by your backend. Use the existing templates (`rust.rs.tera`, `python.py.tera`, etc.) as reference.

### 3. Register the module

Add `pub mod <language>_backend;` to `src/lib.rs`.

### 4. Wire the CLI

Update `src/main.rs` to accept the new language in the `--lang` flag and instantiate your backend.

### 5. Add integration tests

Create `tests/<language>_backend.rs`. Use the existing integration tests as a template.

### 6. Update CI

If the language requires external tools, add them to `.github/workflows/ci.yml`.

## Type translation

The `Type` enum in `ast.rs` has two variants:

- `Type::Simple(String)` -- e.g. `"i32"`, `"Vec<T>"`
- `Type::Parameterized { base, params }` -- e.g. `HashMap<K, V>`

Your backend should provide a `to_<lang>_type` function that converts AST types to the target language's type syntax. Each backend also registers its supported type mappings in the `casing` module for validation-time reporting.

## Casing conventions

Spec authors write field and method names in `snake_case`. Backends use the `casing` module (`src/casing.rs`) to convert to the target language's convention:

| Language     | Convention   | Example          |
|--------------|--------------|------------------|
| Rust         | snake_case   | `is_empty`       |
| Python       | snake_case   | `is_empty`       |
| C#           | PascalCase   | `IsEmpty`        |
| TypeScript   | camelCase    | `isEmpty`        |
| Go           | PascalCase   | `IsEmpty`        |

## Contract assertion injection

If `--contracts` is enabled, each method's `injected_assertions` vector contains precondition, postcondition, and invariant strings. Your backend should render these as language-appropriate runtime assertions before the method stub.

## Formatter integration

Most backends pipe the raw template output through a code formatter before returning it. If the formatter is not installed, the raw code is returned as a fallback. See the `format_python`, `format_csharp`, `format_typescript`, `format_go`, or `format_rust` methods for examples.

## Stub patterns

Every method must be generated with a placeholder:

| Language     | Stub pattern                          |
|--------------|---------------------------------------|
| Rust         | `todo!()`                             |
| Python       | `raise NotImplementedError`           |
| C#           | `throw new NotImplementedException();`|
| TypeScript   | `throw new Error('Not implemented');` |
| Go           | `panic("not implemented")`            |

## Verification

Before submitting a backend, run:

```
cargo build && cargo test && cargo clippy -- -D warnings && cargo fmt --check
```

All generated output must pass the target language's standard linter and formatter with zero warnings.
