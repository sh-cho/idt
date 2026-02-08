use thiserror::Error;

#[derive(Error, Debug)]
pub enum IdtError {
    #[error("Invalid ID format: {0}")]
    InvalidFormat(String),

    #[error("Unknown ID type: {0}")]
    UnknownType(String),

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Encoding error: {0}")]
    EncodingError(String),

    #[error("Generation error: {0}")]
    GenerationError(String),

    #[error("Invalid argument: {0}")]
    InvalidArgument(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("Detection failed: could not determine ID type")]
    DetectionFailed,

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Conversion not supported: {from} -> {to}")]
    ConversionNotSupported { from: String, to: String },
}

pub type Result<T> = std::result::Result<T, IdtError>;
