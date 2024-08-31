use std::{
    cell::LazyCell,
    collections::HashMap,
    fmt::{format, Debug},
    ops::Deref,
};

use anyhow::{anyhow, bail};
use reqwest::{header::CONTENT_TYPE, RequestBuilder, Response, StatusCode};
use serde::{Serialize, Serializer};
use serde_json::{json, Value};

use crate::{
    api::{auth::AccessToken, errors::UsosError, oauth1::authorize, params::IntoParams},
    errors::AppError,
    keys::ConsumerKey,
};

pub struct UsosUri;

impl UsosUri {
    pub const DOMAIN: &'static str = "apps.usos.pwr.edu.pl";

    pub fn origin() -> String {
        format!("https://{}/", Self::DOMAIN)
    }

    pub fn with_path(path: impl AsRef<str>) -> String {
        format!("{}{}", Self::origin(), path.as_ref())
    }
}

pub struct Client {
    base_url: &'static str,
    client: reqwest::Client,
}

impl Client {
    fn new(base_url: &'static str) -> Self {
        let client = reqwest::Client::builder()
            .user_agent(format!(
                "{}/{}",
                env!("CARGO_PKG_NAME"),
                env!("CARGO_PKG_VERSION")
            ))
            .cookie_store(true)
            .build()
            .unwrap();

        Self { base_url, client }
    }

    fn builder(&self, uri: String) -> UsosRequestBuilder {
        UsosRequestBuilder::new(&self.client, format!("{}/services/{uri}", self.base_url))
    }
}

impl Deref for Client {
    type Target = reqwest::Client;

    fn deref(&self) -> &Self::Target {
        &self.client
    }
}

pub const CLIENT: LazyCell<Client> = LazyCell::new(|| Client::new("https://apps.usos.pwr.edu.pl/"));

#[async_trait::async_trait]
pub trait UsosDebug {
    async fn debug(self) -> Result<Response, UsosError>;
}

#[async_trait::async_trait]
impl UsosDebug for reqwest::Response {
    async fn debug(self) -> Result<Response, UsosError> {
        if self.status().is_client_error() {
            let error = self.json::<UsosError>().await.unwrap();
            return Err(error);
        }
        Ok(self)
    }
}

#[derive(Default)]
struct Form<'a> {
    payload: Option<HashMap<String, String>>,
    auth: Option<(&'a ConsumerKey, Option<&'a AccessToken>)>,
    payload_error: Option<serde_urlencoded::ser::Error>,
}

impl<'a> Form<'a> {
    fn new(
        payload: Option<HashMap<String, String>>,
        auth: Option<(&'a ConsumerKey, Option<&'a AccessToken>)>,
    ) -> Self {
        Self {
            payload,
            auth,
            payload_error: None,
        }
    }
}

struct UsosRequestBuilder<'a> {
    request_builder: reqwest::RequestBuilder,
    uri: String,
    form: Form<'a>,
}

impl<'a> UsosRequestBuilder<'a> {
    fn new(client: &reqwest::Client, uri: String) -> Self {
        Self {
            request_builder: client.post(&uri),
            uri,
            form: Form::new(None, None),
        }
    }

    fn payload<T: IntoParams>(mut self, payload: T) -> Self {
        self.form.payload = Some(payload.into_params());
        self
    }

    fn auth(
        mut self,
        consumer_key: &'a ConsumerKey,
        access_token: Option<&'a AccessToken>,
    ) -> Self {
        self.form.auth = Some((consumer_key, access_token));
        self
    }

    async fn request(mut self) -> Result<Response, AppError> {
        if let Some(e) = self.form.payload_error {
            // TODO: handle invalid form error
            return Err(AppError::Unexpected(anyhow::anyhow!(e)));
        }

        let signed_form = match self.form.auth {
            Some((consumer_key, token)) => {
                authorize("POST", &self.uri, consumer_key, token, self.form.payload)
            }
            None => self.form.payload.unwrap_or_else(HashMap::new),
        };

        self.request_builder = self.request_builder.form(&signed_form);

        let response = self.request_builder.send().await?;
        let status = response.status();
        if status.is_client_error() {
            if status == StatusCode::NOT_FOUND {
                return Err(AppError::UnknownMethod(self.uri));
            }
            if status == StatusCode::UNAUTHORIZED {
                println!("Unauthorized, token expired (session expired / user logged out / user revoked all tokens)");
            }
            let error = response.json::<UsosError>().await?;
            println!("{}", error);
            return Err(AppError::Http {
                code: status,
                message: error,
            });
        }
        if status.is_server_error() {
            println!("Internal server error");
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

    async fn request_json(mut self) -> Result<Value, AppError> {
        let res = self.request().await?;
        Ok(res.json().await?)
    }
}

#[tokio::test]
async fn test_usos_client() {
    let client = Client::new("https://apps.usos.pw.edu.pl");
    let response = client
        .builder("apiref/method".into())
        .payload([("name", "services/apiref/method"), ("fields", "id|name")])
        .request_json()
        .await
        .unwrap();

    println!("{:?}", response);
}
