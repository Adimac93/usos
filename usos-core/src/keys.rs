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

#[derive(Debug)]
pub struct ConsumerKeyRef {
    /// Developer email
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
