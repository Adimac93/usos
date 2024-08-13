use anyhow::anyhow;
use reqwest::StatusCode;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Http error {code} - {message}")]
    Http { code: StatusCode, message: String },
    #[error(transparent)]
    Unexpected(#[from] anyhow::Error),
}

impl AppError {
    pub fn http(code: StatusCode, message: impl Into<String>) -> Self {
        Self::Http {
            code,
            message: message.into(),
        }
    }
}

impl From<reqwest::Error> for AppError {
    fn from(value: reqwest::Error) -> Self {
        Self::Unexpected(anyhow!(value))
    }
}
