use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Storage error: {0}")]
    Storage(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Manifest error: {0}")]
    Manifest(String),

    #[error("Signing error: {0}")]
    Signing(String),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Initialization error: {0}")]
    InitializationError(String),

    #[error("Hex decode error: {0}")]
    HexDecode(#[from] hex::FromHexError),

    #[error("CC Attestation error: {0}")]
    CCAttestationError(String),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
