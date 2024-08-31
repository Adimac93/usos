use anyhow::anyhow;
use reqwest::StatusCode;
use thiserror::Error;

use crate::api::errors::UsosError;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Http error {code}")]
    Http {
        code: StatusCode,
        message: Option<UsosError>,
    },
    #[error(transparent)]
    Unexpected(#[from] anyhow::Error),
}

impl AppError {
    pub fn http(code: StatusCode, message: Option<UsosError>) -> Self {
        Self::Http { code, message }
    }

    pub fn usos_error(&self) -> Option<&UsosError> {
        match self {
            Self::Http { message, .. } => message.as_ref(),
            _ => None,
        }
    }
}

impl From<reqwest::Error> for AppError {
    fn from(value: reqwest::Error) -> Self {
        Self::Unexpected(anyhow!(value))
    }
}
