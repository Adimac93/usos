use std::{env::VarError, path::Path};

use anyhow::Context;
use fantoccini::{error::CmdError, Locator};
use secrecy::{ExposeSecret, Secret, SecretString};
use serde::{Deserialize, Serialize};
use time::macros::format_description;

use crate::{
    client::{UsosUri, CLIENT},
    errors::AppError,
};

const CONSUMER_KEY_NAME: &str = "USOS_CONSUMER_KEY";
const CONSUMER_SECRET_NAME: &str = "USOS_CONSUMER_SECRET";
const CONSUMER_KEY_OWNER: &str = "USOS_CONSUMER_EMAIL";

pub struct ConsumerKey {
    pub owner: Option<String>,
    pub key: String,
    pub secret: SecretString,
}

impl ConsumerKey {
    pub fn from_env() -> Result<Self, VarError> {
        let key = std::env::var(CONSUMER_KEY_NAME)?;
        let secret = SecretString::new(std::env::var(CONSUMER_SECRET_NAME)?);
        Ok(Self {
            key,
            secret,
            owner: std::env::var(CONSUMER_KEY_OWNER).ok(),
        })
    }

    pub async fn generate(
        app_name: &str,
        website_url: Option<&str>,
        email: &str,
    ) -> crate::Result<Self> {
        let form = RegistrationForm::new(app_name, website_url, email);
        let response = CLIENT.get(UsosUri::with_path("/developers")).send().await?;
        let csrf_token_cookie = response
            .cookies()
            .next()
            .ok_or(AppError::usos("Csrf token cookie expected but not found"))?;

        let response = CLIENT
            .post(UsosUri::with_path("/developers/submit"))
            .header(
                "Cookie",
                &format!("csrftoken={}", csrf_token_cookie.value()),
            )
            .header("Host", UsosUri::DOMAIN)
            .header("Origin", UsosUri::origin())
            .header("Referer", UsosUri::with_path("/developers"))
            .header("X-CSRFToken", csrf_token_cookie.value())
            .form(&form)
            .send()
            .await?;

        if !response.status().is_success() {
            println!("Registration not successful. Response: {response:#?}");
        }

        let reg: RegistrationResponse = response
            .json()
            .await
            .map_err(|e| AppError::invalid_json(e.to_string()))?;
        if reg.status != "success" {
            println!("Registration not successful. Status: {}", reg.status);
        }

        return Ok(Self {
            key: reg.consumer_key,
            secret: reg.consumer_secret,
            owner: Some(email.into()),
        });
    }

    pub async fn revoke(self) -> crate::Result<()> {
        let form = RevokeForm::new(self.key, self.secret.expose_secret());
        let response = CLIENT
            .get(UsosUri::with_path("/services/oauth/revoke_consumer_key"))
            .query(&form)
            .send()
            .await?;

        if response.status().is_client_error() {
            let json = response
                .json::<serde_json::Value>()
                .await
                .map_err(|e| AppError::invalid_json(e.to_string()))?;
            let message = json["message"].as_str();
            println!("Revocation error: {message:?}")
        }
        Ok(())
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

#[derive(Serialize)]
struct RevokeForm {
    consumer_key: String,
    consumer_secret: String,
}

impl RevokeForm {
    fn new(consumer_key: impl Into<String>, consumer_secret: impl Into<String>) -> Self {
        Self {
            consumer_key: consumer_key.into(),
            consumer_secret: consumer_secret.into(),
        }
    }
}

pub async fn gen_consumer_keys(
    app_name: &str,
    website_url: Option<&str>,
    email: &str,
) -> Result<(String, Secret<String>), CmdError> {
    let client = fantoccini::ClientBuilder::native()
        .connect("http://localhost:4444")
        .await
        .unwrap();

    client.goto(&UsosUri::with_path("/developers")).await?;
    client
        .find(Locator::Id("appname"))
        .await?
        .send_keys(app_name)
        .await?;

    client
        .find(Locator::Id("appurl"))
        .await?
        .send_keys(website_url.unwrap_or_default())
        .await?;

    client
        .find(Locator::Id("email"))
        .await?
        .send_keys(email)
        .await?;

    client.find(Locator::Id("submit")).await?.click().await?;

    let consumer_key = client
        .wait()
        .for_element(Locator::Id("kkey"))
        .await?
        .text()
        .await?;

    let consumer_secret = client.find(Locator::Id("ksecret")).await?.text().await?;

    client.close().await?;
    Ok((consumer_key, Secret::new(consumer_secret)))
}

async fn revoke_consumer_keys(consumer_key: &str, consumer_secret: &str) -> Result<(), CmdError> {
    let client = fantoccini::ClientBuilder::native()
        .connect("http://localhost:4444")
        .await
        .unwrap();

    client.goto(&UsosUri::with_path("/developers")).await?;
    client
        .find(Locator::Id("consumer_key"))
        .await?
        .send_keys(consumer_key)
        .await?;

    client
        .find(Locator::Id("consumer_secret"))
        .await?
        .send_keys(consumer_secret)
        .await?;

    client.find(Locator::Id("revoke")).await?.click().await?;
    client.accept_alert().await?;

    client.close().await?;

    println!("Check your email for confirmation.");
    Ok(())
}
