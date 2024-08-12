use anyhow::anyhow;
use reqwest::StatusCode;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("{code} - {message}")]
    Client { code: StatusCode, message: String },
    #[error("Error from USOS API: {0}")]
    Usos(String),
    #[error("Invalid JSON: {0}")]
    InvalidJson(String),
    #[error(transparent)]
    Unexpected(#[from] anyhow::Error),
}

impl AppError {
    pub fn client(code: StatusCode, message: impl Into<String>) -> Self {
        Self::Client {
            code,
            message: message.into(),
        }
    }

    pub fn usos(msg: impl Into<String>) -> Self {
        Self::Usos(msg.into())
    }

    pub fn invalid_json(msg: impl Into<String>) -> Self {
        Self::InvalidJson(msg.into())
    }
}

impl From<reqwest::Error> for AppError {
    fn from(value: reqwest::Error) -> Self {
        Self::Unexpected(anyhow!(value))
    }
}
