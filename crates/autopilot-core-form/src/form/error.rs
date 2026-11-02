use thiserror::Error;

#[derive(Debug, Error)]
pub enum FormError {
    #[error("invalid form schema: {0}")]
    InvalidSchema(String),
    #[error("invalid value for '{field}': {message}")]
    InvalidValue { field: String, message: String },
    #[error("missing required field '{0}'")]
    MissingField(String),
    #[error("unsupported schema: {0}")]
    Unsupported(String),
    #[error("form provider error: {0}")]
    Provider(String),
    #[error("form serialization error: {0}")]
    Serialization(String),
}

/// A validation failure tied to a single field, suitable for inline display.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FormFieldError {
    pub field: String,
    pub message: String,
}
