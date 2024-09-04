#[cfg(feature = "keygen")]
pub mod generation;

use std::{env::VarError, ops::Deref, path::Path, sync::Arc};

use anyhow::Context;
use reqwest::{
    header::{COOKIE, HOST, ORIGIN, REFERER},
    Url,
};
use secrecy::{ExposeSecret, Secret, SecretString};
use serde::{Deserialize, Serialize};
use time::macros::format_description;

use crate::{api::errors::UsosError, errors::AppError};

const CONSUMER_KEY_NAME: &str = "USOS_CONSUMER_KEY";
const CONSUMER_SECRET_NAME: &str = "USOS_CONSUMER_SECRET";
const CONSUMER_KEY_OWNER: &str = "USOS_CONSUMER_EMAIL";

/// USOS API consumer.
///
/// This is the identifier of a single application using USOS API. An application should have one consumer key.
///
/// The Consumer key is required to call a majority of USOS API endpoints, including authentication of students.
///
/// Cloning this struct is cheap, because it contains an inner `Arc`.
///
/// Contains a consumer key, a consumer secret and an application developer email. For more details, go to [`ConsumerKeyRef`] and [the USOS API reference](https://apps.usos.pw.edu.pl/developers/api/authorization/).
#[derive(Debug, Clone)]
pub struct ConsumerKey {
    inner: Arc<ConsumerKeyRef>,
}

impl Deref for ConsumerKey {
    type Target = ConsumerKeyRef;

    fn deref(&self) -> &Self::Target {
        self.inner.as_ref()
    }
}

/// Inner contents of [`ConsumerKey`].
#[derive(Debug, Clone)]
pub struct ConsumerKeyRef {
    /// Developer email. Not required for calling any endpoint.
    pub owner: Option<String>,
    pub key: String,
    pub secret: SecretString,
}

impl ConsumerKey {
    pub fn new(key: String, secret: SecretString, owner: Option<String>) -> Self {
        Self {
            inner: Arc::new(ConsumerKeyRef { owner, key, secret }),
        }
    }

    /// Constructs `ConsumerKey` from environment variables.
    ///
    /// The names of environment variables are as follows:
    /// - consumer key (required) - USOS_CONSUMER_KEY
    /// - consumer secret (required) - USOS_CONSUMER_SECRET
    /// - consumer key owner's email (optional) - USOS_CONSUMER_EMAIL
    pub fn from_env() -> Result<Self, VarError> {
        let key = std::env::var(CONSUMER_KEY_NAME)?;
        let secret = SecretString::new(std::env::var(CONSUMER_SECRET_NAME)?);
        Ok(Self {
            inner: Arc::new(ConsumerKeyRef {
                key,
                secret,
                owner: std::env::var(CONSUMER_KEY_OWNER).ok(),
            }),
        })
    }
}
