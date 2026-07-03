//! Hand-rolled error types for parser/validation (`SpecError`) and
//! template/formatter failures (`BackendError`).

use std::fmt;

/// Errors produced during spec parsing and validation.
///
/// Carries line/column information when available for parse errors,
/// and JSON Schema instance path for validation errors.
#[derive(Debug)]
pub enum SpecError {
    /// YAML parse failure with optional source location.
    ParseError {
        message: String,
        line: Option<usize>,
        column: Option<usize>,
    },
    /// JSON Schema validation failure with path.
    ValidationError { message: String, path: String },
    /// Internal schema compilation failure.
    SchemaError { message: String },
    /// Spec version is incompatible with the current binary.
    VersionMismatch { expected: String, found: String },
    /// Filesystem I/O error.
    IoError { message: String },
}

impl fmt::Display for SpecError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SpecError::ParseError {
                message,
                line,
                column,
            } => {
                if let (Some(l), Some(c)) = (line, column) {
                    write!(f, "parse error at line {l}, column {c}: {message}")
                } else {
                    write!(f, "parse error: {message}")
                }
            }
            SpecError::ValidationError { message, path } => {
                write!(f, "validation error at {path}: {message}")
            }
            SpecError::SchemaError { message } => {
                write!(f, "schema error: {message}")
            }
            SpecError::VersionMismatch { expected, found } => {
                write!(f, "version mismatch: expected {expected}, found {found}")
            }
            SpecError::IoError { message } => {
                write!(f, "I/O error: {message}")
            }
        }
    }
}

impl std::error::Error for SpecError {}

/// Errors produced during code generation and formatting.
#[derive(Debug)]
pub enum BackendError {
    /// Tera template engine initialization failure.
    TemplateInit { message: String },
    /// Tera template rendering failure.
    TemplateRender { message: String },
    /// External code formatter failure (rustfmt, black, etc.).
    Formatter { message: String },
}

impl fmt::Display for BackendError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BackendError::TemplateInit { message } => {
                write!(f, "template init error: {message}")
            }
            BackendError::TemplateRender { message } => {
                write!(f, "template render error: {message}")
            }
            BackendError::Formatter { message } => {
                write!(f, "formatter error: {message}")
            }
        }
    }
}

impl std::error::Error for BackendError {}
