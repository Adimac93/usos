use std::path::Path;

use fantoccini::{error::CmdError, Locator};
use secrecy::{ExposeSecret, Secret, SecretString};
use serde::{Deserialize, Serialize};

use crate::client::{BASE_URL, CLIENT};

const CONSUMER_KEY_NAME: &str = "USOS_CONSUMER_KEY";
const CONSUMER_SECRET_NAME: &str = "USOS_CONSUMER_SECRET";

pub struct ConsumerKey {
    owner: Option<String>,
    key: String,
    secret: SecretString,
}

impl ConsumerKey {
    pub fn from_env() -> Self {
        let key = std::env::var(CONSUMER_KEY_NAME).unwrap();
        let secret = SecretString::new(std::env::var(CONSUMER_SECRET_NAME).unwrap());
        Self {
            key,
            secret,
            owner: None,
        }
    }
    pub async fn generate(
        app_name: &str,
        website_url: Option<&str>,
        email: &str,
    ) -> reqwest::Result<Self> {
        let form = RegistrationForm::new(app_name, website_url, email);
        let response = CLIENT.get(format!("{BASE_URL}/developers")).send().await?;

        let response = CLIENT
            .post(format!("{BASE_URL}/developers/submit"))
            .header(
                "Cookie",
                &format!("csrftoken={}", response.cookies().next().unwrap().value()),
            )
            .header("Host", "apps.usos.pwr.edu.pl")
            .header("Origin", "https://apps.usos.pwr.edu.pl")
            .header("Referer", "https://apps.usos.pwr.edu.pl/developers/")
            .header(
                "X-CSRFToken",
                response
                    .cookies()
                    .find(|cookie| cookie.name() == "csrftoken")
                    .unwrap()
                    .value(),
            )
            .form(&form)
            .send()
            .await?;
        if response.status().is_client_error() {
            println!("Registration client error");
        }
        let reg: RegistrationResponse = response.json().await.unwrap();
        if reg.status != "success" {
            println!("Unexpected status: {}", reg.status);
        }
        return Ok(Self {
            key: reg.consumer_key,
            secret: reg.consumer_secret,
            owner: Some(email.into()),
        });
    }
    pub async fn revoke(self) -> reqwest::Result<()> {
        let form = RevokeForm::new(self.key, self.secret.expose_secret());
        let response = CLIENT
            .get(format!("{BASE_URL}/services/oauth/revoke_consumer_key"))
            .query(&form)
            .send()
            .await?;

        if response.status().is_client_error() {
            let json = response.json::<serde_json::Value>().await.unwrap();
            let message = json["message"].as_str();
            println!("Revokation error: {message:?}")
        }
        Ok(())
    }
    pub async fn save_to_file(&self) -> Result<(), tokio::io::Error> {
        tokio::fs::write(
            Path::new(&format!(
                "./{}_consumer_key.env",
                time::OffsetDateTime::now_utc()
            )),
            format!(
                "{CONSUMER_KEY_NAME}={}\n{CONSUMER_SECRET_NAME}={}\nUSOS_CONSUMER_EMAIL={}\n",
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

    client.goto(&format!("{BASE_URL}/developers/")).await?;
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

    client.goto(&format!("{BASE_URL}/developers/")).await?;
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
