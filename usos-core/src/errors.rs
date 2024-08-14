use anyhow::anyhow;
use reqwest::StatusCode;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Http error {code} - {message}")]
    Http { code: StatusCode, message: String },
    #[error("Response parsing failed: {0}")]
    Parse(String),
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

    pub fn parse(msg: impl Into<String>) -> Self {
        Self::Parse(msg.into())
    }
}

impl From<reqwest::Error> for AppError {
    fn from(value: reqwest::Error) -> Self {
        Self::Unexpected(anyhow!(value))
    }
}
