use std::{
    cell::LazyCell,
    collections::{BTreeMap, HashMap},
    env::VarError,
    fmt::{format, Debug},
    ops::Deref,
    sync::Arc,
};

use anyhow::{anyhow, bail};
use reqwest::{header::CONTENT_TYPE, IntoUrl, RequestBuilder, Response, StatusCode, Url};
use serde::{Serialize, Serializer};
use serde_json::{json, Value};

use crate::{
    api::{auth::AccessToken, errors::UsosError, oauth1::authorize, params::Params},
    errors::AppError,
    keys::ConsumerKey,
};

#[derive(Debug)]
pub struct Client {
    base_url: Url,
    client: reqwest::Client,
    auth: Option<ConsumerKey>,
}

impl Client {
    pub fn new(base_url: Url) -> Self {
        let client = reqwest::Client::builder()
            .user_agent(format!(
                "{}/{}",
                env!("CARGO_PKG_NAME"),
                env!("CARGO_PKG_VERSION")
            ))
            .cookie_store(true)
            .build()
            .unwrap();

        Self {
            base_url,
            client,
            auth: None,
        }
    }

    pub fn authorized_from_env(mut self) -> Result<Self, VarError> {
        let consumer_key = ConsumerKey::from_env()?;
        self.auth = Some(consumer_key);
        Ok(self)
    }

    pub async fn authorized(
        mut self,
        app_name: &str,
        website_url: Option<&str>,
        email: &str,
    ) -> crate::Result<Self> {
        let consumer_key =
            ConsumerKey::generate(&self.client, self.base_url(), app_name, website_url, email)
                .await?;
        self.auth = Some(consumer_key);
        Ok(self)
    }

    pub fn builder(&self, uri: impl AsRef<str>) -> UsosRequestBuilder {
        UsosRequestBuilder::new(
            &self.client,
            self.base_url
                .join("services/") // trailing slash is significant
                .unwrap()
                .join(uri.as_ref())
                .unwrap(),
            self.auth.clone(),
        )
    }

    pub fn base_url(&self) -> &Url {
        &self.base_url
    }
}

pub const CLIENT: LazyCell<Client> =
    LazyCell::new(|| Client::new(Url::parse("https://apps.usos.pwr.edu.pl").unwrap()));

#[derive(Default)]
struct Form<'a> {
    payload: Option<BTreeMap<String, String>>,
    auth: Option<(ConsumerKey, Option<&'a AccessToken>)>,
    payload_error: Option<serde_urlencoded::ser::Error>,
}

impl<'a> Form<'a> {
    fn new(
        payload: Option<BTreeMap<String, String>>,
        auth: Option<(ConsumerKey, Option<&'a AccessToken>)>,
    ) -> Self {
        Self {
            payload,
            auth,
            payload_error: None,
        }
    }
}

pub struct UsosRequestBuilder<'a> {
    request_builder: reqwest::RequestBuilder,
    uri: Url,
    form: Form<'a>,
}

impl<'a> UsosRequestBuilder<'a> {
    fn new(client: &reqwest::Client, uri: Url, consumer_key: Option<ConsumerKey>) -> Self {
        Self {
            request_builder: client.post(uri.as_ref()),
            uri,
            form: Form::new(None, consumer_key.map(|key| (key, None))),
        }
    }

    pub fn payload<T: Into<Params>>(mut self, payload: T) -> Self {
        self.form.payload = Some(payload.into().0);
        self
    }

    pub fn auth(
        mut self,
        consumer_key: Option<ConsumerKey>,
        access_token: Option<&'a AccessToken>,
    ) -> Self {
        if let Some((consumer, access)) = &mut self.form.auth {
            if let Some(consumer_key) = consumer_key {
                *consumer = consumer_key;
            }
            *access = access_token;
        }

        self
    }

    pub async fn request(mut self) -> Result<Response, AppError> {
        if let Some(e) = self.form.payload_error {
            // TODO: handle invalid form error
            return Err(AppError::Unexpected(anyhow::anyhow!(e)));
        }

        let signed_form = match self.form.auth {
            Some((consumer_key, token)) => authorize(
                "POST",
                self.uri.to_string(),
                &consumer_key,
                token,
                self.form.payload,
            ),
            None => self.form.payload.unwrap_or_else(BTreeMap::new),
        };

        self.request_builder = self.request_builder.form(&signed_form);

        let response = self.request_builder.send().await?;
        let status = response.status();
        if status.is_client_error() {
            if status == StatusCode::NOT_FOUND {
                return Err(AppError::http(status, None));
            }
            if status == StatusCode::UNAUTHORIZED {
                println!("Unauthorized, token expired (session expired / user logged out / user revoked all tokens)");
                return Err(AppError::http(status, None));
            }
            let error = response.json::<UsosError>().await?;
            println!("{}", error);
            return Err(AppError::http(status, Some(error)));
        }
        if status.is_server_error() {
            println!("Internal server error");
            return Err(AppError::http(status, None));
        }

        if status.is_redirection() {
            return Ok(response);
        }

        if status.is_informational() {
            return Err(AppError::Unexpected(anyhow!(
                "Status codes 100-199 are unexpected"
            )));
        }
        return Ok(response);
    }

    pub async fn request_json(mut self) -> Result<Value, AppError> {
        let res = self.request().await?;
        Ok(res.json().await?)
    }
}

#[tokio::test]
async fn test_usos_client() {
    let client = Client::new(Url::parse("https://apps.usos.pw.edu.pl").unwrap());
    let response = client
        .builder("apiref/method")
        .payload([
            ("fields", "name|short_name"),
            ("name", "services/apiref/method"),
        ])
        .request_json()
        .await
        .unwrap();

    println!("{:?}", response);
}

#[test]
fn relative_url_parts_joining_is_successful() {
    let base = "https://apps.usos.pwr.edu.pl";
    let client = Client::new(Url::parse(base).unwrap());
    let builder = client.builder("apiref/method");
    assert_eq!(
        builder.uri,
        Url::parse("https://apps.usos.pwr.edu.pl/services/apiref/method").unwrap()
    )
}

#[test]
fn relative_url_parts_joining_is_failing() {
    let base = "https://apps.usos.pwr.edu.pl";
    let client = Client::new(Url::parse(base).unwrap());
    let builder = client.builder("/apiref/method");
    assert_eq!(
        builder.uri,
        Url::parse("https://apps.usos.pwr.edu.pl/apiref/method").unwrap()
    )
}
