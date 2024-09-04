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

    /// Registers a new consumer key with the USOS API.
    ///
    /// IMPORTANT: This function performs calls to USOS API that modify your resources.
    /// Specifically, it sends a POST request to `{base_url}/developers/submit`. Use with caution.
    ///
    /// This function sends a registration request to the USOS API using the provided client, base URL,
    /// application name, and email. It also supports an optional website URL.
    ///
    /// # Arguments
    ///
    /// * `client` - The `reqwest::Client` used to make the HTTP requests.
    /// * `base_url` - The base URL (origin) of the USOS API **with trailing slash** (example: <https://apps.usos.pwr.edu.pl/>).
    /// * `app_name` - The name of the application to register.
    /// * `website_url` - An optional website URL associated with the application.
    /// * `email` - The developer's email address used for registration.
    ///
    /// # Returns
    ///
    /// Returns `Ok(Self)` with a newly registered `ConsumerKey` upon successful registration.
    /// If the registration fails due to a client or server error, an `AppError::Unexpected` error is returned.
    ///
    /// # Errors
    ///
    /// This function depends on USOS API and therefore can fail due to the api error, in which case an unexpected [`AppError`] is returned.
    ///
    /// # Panics
    ///
    /// Panics if the `base_url` does not end with a trailing slash or isn't a valid HTTP origin.
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

    /// Saves the consumer key information to a `.env` file.
    ///
    /// IMPORTANT: This call exposes the consumer secret and writes it to the file.
    ///
    /// This function writes the consumer key, consumer secret, and optionally the owner's email
    /// to a file in the current directory.
    ///
    /// The file contents will be formatted as environment variables:
    ///
    /// ```env
    /// USOS_CONSUMER_KEY=your_consumer_key
    /// USOS_CONSUMER_SECRET=your_consumer_secret
    /// USOS_CONSUMER_EMAIL=your_email@example.com
    ///
    /// # if no consumer email is provided:
    /// # USOS_CONSUMER_EMAIL=
    /// ```
    ///
    /// Example target file name: `2024-09-03_11-50_consumer_key.env`
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

    /// Revokes the consumer key with the USOS API.
    ///
    /// IMPORTANT: This function performs calls to USOS API that modify your resources. Use with caution.
    /// Also, after calling this endpoint you will receive an email from USOS to complete revocation process.
    ///
    /// This function sends a request to the `/services/oauth/revoke_consumer_key` endpoint of the USOS API,
    /// effectively revoking the consumer key and secret that were previously issued.
    /// This is useful when you want to invalidate a key, such as when it's no longer needed
    /// or has been compromised.
    ///
    /// # Arguments
    ///
    /// * `base_url` - The base URL (origin) of the USOS API, **with a trailing slash**.
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
