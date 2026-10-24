use thiserror::Error;

#[derive(Debug, Error)]
pub enum FsError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Invalid file id: {0}")]
    InvalidId(String),

    #[error("Filesystem paths have not been initialized")]
    NotInitialized,

    #[error("Filesystem initialization error: {0}")]
    Init(String),
}

pub type Result<T> = std::result::Result<T, FsError>;
