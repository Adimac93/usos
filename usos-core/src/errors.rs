use anyhow::anyhow;
use reqwest::StatusCode;
use thiserror::Error;

use crate::api::errors::UsosError;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Http error {code} - {message}")]
    Http {
        code: StatusCode,
        message: UsosError,
    },
    #[error("Called unknown method {0}")]
    UnknownMethod(String),
    #[error("Response parsing failed: {0}")]
    Parse(String),
    #[error(transparent)]
    Unexpected(#[from] anyhow::Error),
}

impl AppError {
    pub fn http(code: StatusCode, message: UsosError) -> Self {
        Self::Http { code, message }
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
