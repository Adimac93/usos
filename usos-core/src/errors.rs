//! Crate errors, including both USOS API errors and internal unexpected errors.

use anyhow::anyhow;
use reqwest::StatusCode;
use thiserror::Error;

use crate::api::errors::UsosError;

#[derive(Error, Debug)]
pub enum AppError {
    /// Error returned from USOS API. See [`UsosError`] and [the USOS API reference](https://apps.usos.pw.edu.pl/developers/api/definitions/errors/).
    #[error("Http error {code}")]
    Http {
        code: StatusCode,
        message: Option<UsosError>,
    },
    /// Unexpected error caused by the crate or any of its dependencies.
    #[error(transparent)]
    Unexpected(#[from] anyhow::Error),
}

impl AppError {
    /// Constructs an Http variant.
    pub fn http(code: StatusCode, message: Option<UsosError>) -> Self {
        Self::Http { code, message }
    }

    /// Tries to extract [`UsosError`].
    ///
    /// In case of the `Http` variant, returns the inner [`UsosError`] if it is present.
    ///
    /// Calling this method on the `Unexpected` variant always results in `None`.
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
