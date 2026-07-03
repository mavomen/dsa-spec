//! Crate root. Re-exports all modules for the `dsa-spec` binary.

pub mod assertion;
pub mod ast;
pub mod backend;
pub mod casing;
pub mod complexity;
pub mod context;
pub mod contracts;
pub mod csharp_backend;
pub mod doc_gen;
pub mod error;
pub mod go_backend;
pub mod migrate;
pub mod parser;
pub mod python_backend;
pub mod rust_backend;
pub mod spec_schema;
pub mod template_engine;
pub mod typescript_backend;
pub mod validator;
pub mod visualization;
