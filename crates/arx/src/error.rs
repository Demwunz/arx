use thiserror::Error;

#[derive(Debug, Error)]
pub enum ArxError {
    #[error("entry not found")]
    NotFound,

    #[error("invalid entry type")]
    InvalidType,

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("yaml error: {0}")]
    Yaml(#[from] serde_yml::Error),

    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("{0}")]
    Other(String),
}
