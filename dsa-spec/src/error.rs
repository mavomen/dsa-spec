use std::fmt;

#[derive(Debug)]
#[allow(dead_code)] // variants used by migrate & emitter modules (future)
pub enum SpecError {
    ParseError {
        message: String,
        line: Option<usize>,
        column: Option<usize>,
    },
    ValidationError {
        message: String,
        path: String,
    },
    SchemaError {
        message: String,
    },
    VersionMismatch {
        expected: String,
        found: String,
    },
    IoError {
        message: String,
    },
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

#[derive(Debug)]
#[allow(dead_code)] // TypeMapping & Io variants used by future emitter & validate
pub enum BackendError {
    TemplateInit { message: String },
    TemplateRender { message: String },
    Formatter { message: String },
    TypeMapping { message: String },
    Io { message: String },
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
            BackendError::TypeMapping { message } => {
                write!(f, "type mapping error: {message}")
            }
            BackendError::Io { message } => {
                write!(f, "I/O error: {message}")
            }
        }
    }
}

impl std::error::Error for BackendError {}
