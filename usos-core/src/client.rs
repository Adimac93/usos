use std::{
    cell::LazyCell,
    collections::{BTreeMap, HashMap},
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

#[derive(Debug)]
pub struct Client {
    base_url: &'static str,
    client: reqwest::Client,
}

impl Client {
    pub fn new(base_url: &'static str) -> Self {
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

    pub fn builder(&self, uri: impl AsRef<str>) -> UsosRequestBuilder {
        UsosRequestBuilder::new(
            &self.client,
            format!("{}/services/{}", self.base_url, uri.as_ref()),
        )
    }

    pub fn base_url(&self) -> &str {
        self.base_url
    }
}

pub const CLIENT: LazyCell<Client> = LazyCell::new(|| Client::new("https://apps.usos.pwr.edu.pl"));

#[derive(Default)]
struct Form<'a> {
    payload: Option<BTreeMap<String, String>>,
    auth: Option<(&'a ConsumerKey, Option<&'a AccessToken>)>,
    payload_error: Option<serde_urlencoded::ser::Error>,
}

impl<'a> Form<'a> {
    fn new(
        payload: Option<BTreeMap<String, String>>,
        auth: Option<(&'a ConsumerKey, Option<&'a AccessToken>)>,
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

    pub fn payload<T: IntoParams>(mut self, payload: T) -> Self {
        self.form.payload = Some(payload.into_params());
        self
    }

    pub fn auth(
        mut self,
        consumer_key: &'a ConsumerKey,
        access_token: Option<&'a AccessToken>,
    ) -> Self {
        self.form.auth = Some((consumer_key, access_token));
        self
    }

    pub async fn request(mut self) -> Result<Response, AppError> {
        if let Some(e) = self.form.payload_error {
            // TODO: handle invalid form error
            return Err(AppError::Unexpected(anyhow::anyhow!(e)));
        }

        let signed_form = match self.form.auth {
            Some((consumer_key, token)) => {
                authorize("POST", &self.uri, consumer_key, token, self.form.payload)
            }
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
    dotenvy::dotenv().ok();
    let consumer_key = ConsumerKey::from_env().unwrap();
    let client = Client::new("https://apps.usos.pw.edu.pl");
    let response = client
        .builder("apiref/method")
        .payload([
            ("fields", "name|short_name"),
            ("name", "services/apiref/method"),
        ])
        .auth(&consumer_key, None)
        .request_json()
        .await
        .unwrap();

    println!("{:?}", response);
}
