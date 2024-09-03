use std::{env::VarError, path::Path, sync::Arc};

use anyhow::Context;
use reqwest::{
    header::{COOKIE, HOST, ORIGIN, REFERER},
    Url,
};
use secrecy::{ExposeSecret, Secret, SecretString};
use serde::{Deserialize, Serialize};
use time::macros::format_description;

use crate::{api::errors::UsosError, errors::AppError};

use super::{
    ConsumerKey, ConsumerKeyRef, CONSUMER_KEY_NAME, CONSUMER_KEY_OWNER, CONSUMER_SECRET_NAME,
};

impl ConsumerKey {
    // TODO: either create an error type dedicated for keygen scripts, or allow the function to panic on many occasions related to user input
    // TODO: consider creating a CLI tool for generating keys that uses this code underneath.
    pub async fn generate(
        client: &reqwest::Client,
        base_url: &Url,
        app_name: &str,
        website_url: Option<&str>,
        email: &str,
    ) -> crate::Result<Self> {
        let form = RegistrationForm::new(app_name, website_url, email);
        let response = client
            .get(base_url.join("developers").unwrap())
            .send()
            .await?;

        let csrf_token_cookie = response
            .cookies()
            .next()
            .context("Csrf token cookie expected but not found")?;

        let response = client
            .post(base_url.join("developers/submit").unwrap())
            .header(COOKIE, &format!("csrftoken={}", csrf_token_cookie.value()))
            .header(HOST, base_url.host_str().unwrap())
            .header(ORIGIN, base_url.as_str())
            .header(REFERER, base_url.join("developers").unwrap().as_str())
            .header("X-CSRFToken", csrf_token_cookie.value())
            .form(&form)
            .send()
            .await?;

        if response.status().is_client_error() || response.status().is_server_error() {
            return Err(AppError::Unexpected(anyhow::anyhow!(
                "Registering the consumer key failed. Response from USOS: {response:#?}"
            )));
        }

        let reg: RegistrationResponse = response.json().await?;
        if reg.status != "success" {
            return Err(AppError::Unexpected(anyhow::anyhow!(
                "Registering the consumer key failed. Registration status from USOS: {:#?}",
                reg.status
            )));
        }

        return Ok(Self {
            inner: Arc::new(ConsumerKeyRef {
                key: reg.consumer_key,
                secret: reg.consumer_secret,
                owner: Some(email.into()),
            }),
        });
    }

    pub async fn save_to_file(&self) -> Result<(), tokio::io::Error> {
        tokio::fs::write(
            Path::new(&format!(
                "./{}_consumer_key.env",
                time::OffsetDateTime::now_utc()
                    .format(format_description!("[year]-[month]-[day]_[hour]-[minute]"))
                    .unwrap()
            )),
            format!(
                "{CONSUMER_KEY_NAME}={}\n{CONSUMER_SECRET_NAME}={}\n{CONSUMER_KEY_OWNER}={}\n",
                self.key,
                self.secret.expose_secret(),
                self.owner.as_deref().unwrap_or_default(),
            ),
        )
        .await
    }

    pub async fn revoke(self, base_url: &Url) -> crate::Result<()> {
        let url = base_url.join("services/oauth/revoke_consumer_key").unwrap();

        let response = reqwest::Client::new()
            .post(url)
            .form(&[
                ("consumer_key", &*self.key),
                ("consumer_secret", &*self.secret.expose_secret()),
            ])
            .send()
            .await?;

        let status_code = response.status();

        if status_code.is_client_error() || status_code.is_server_error() {
            let error: UsosError = response.json().await?;

            return Err(AppError::Http {
                code: status_code,
                message: Some(error),
            });
        }

        Ok(())
    }
}

#[derive(Debug, Serialize)]
struct RegistrationForm {
    #[serde(rename = "appname")]
    app_name: String,
    #[serde(rename = "appurl")]
    website_url: Option<String>,
    email: String,
}

impl RegistrationForm {
    fn new(
        app_name: impl Into<String>,
        website_url: Option<impl Into<String>>,
        email: impl Into<String>,
    ) -> Self {
        Self {
            app_name: app_name.into(),
            website_url: website_url.map(Into::<String>::into),
            email: email.into(),
        }
    }
}

#[derive(Deserialize)]
struct RegistrationResponse {
    status: String,
    consumer_key: String,
    consumer_secret: SecretString,
}

impl RegistrationResponse {
    fn new(
        status: impl Into<String>,
        client_id: impl Into<String>,
        client_secret: impl Into<SecretString>,
    ) -> Self {
        Self {
            status: status.into(),
            consumer_key: client_id.into(),
            consumer_secret: client_secret.into(),
        }
    }
}
