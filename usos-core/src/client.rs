use std::{
    cell::LazyCell,
    collections::HashMap,
    fmt::{format, Debug},
    ops::Deref,
};

use reqwest::{RequestBuilder, Response, StatusCode};
use serde::Serialize;
use serde_json::{json, Value};

use crate::{
    api::{auth::AccessToken, errors::UsosError, oauth1::authorize},
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

    fn builder<T: Serialize + Debug>(&self, uri: String) -> UsosRequestBuilder<T> {
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

#[derive(Serialize, Debug)]
struct Form<T>
where
    T: Serialize + Debug,
{
    #[serde(flatten)]
    payload: Option<T>,
    #[serde(flatten)]
    auth: HashMap<String, String>,
}

impl<T> Form<T>
where
    T: Serialize + Debug,
{
    fn new(payload: Option<T>, auth: HashMap<String, String>) -> Self {
        Self { payload, auth }
    }
}

struct UsosRequestBuilder<T>
where
    T: Serialize + Debug,
{
    request_builder: reqwest::RequestBuilder,
    uri: String,
    form: Option<Form<T>>,
}

impl<T> UsosRequestBuilder<T>
where
    T: Serialize + Debug,
{
    fn new(client: &reqwest::Client, uri: String) -> Self {
        Self {
            request_builder: client.post(&uri),
            uri,
            form: None,
        }
    }

    fn payload(mut self, payload: T) -> Self {
        if let Some(form) = &mut self.form {
            form.payload = Some(payload);
        } else {
            self.form = Some(Form::new(Some(payload), HashMap::new()))
        }
        self
    }

    fn auth(mut self, consumer_key: &ConsumerKey, access_token: Option<&AccessToken>) -> Self {
        let auth = authorize("POST", &self.uri, consumer_key, access_token, None);
        if let Some(form) = &mut self.form {
            form.auth.extend(auth);
        } else {
            self.form = Some(Form::new(None, auth))
        }
        self
    }

    fn query<Q: Serialize + ?Sized>(mut self, query: &Q) -> Self {
        self.request_builder = self.request_builder.query(query);
        self
    }

    async fn send(mut self) -> Result<Value, AppError> {
        if let Some(form) = self.form {
            self.request_builder = self.request_builder.form(&form);
        }
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
        if status.is_success() {
            let value = response.json().await?;
            println!("{value:#?}");
            return Ok(value);
        }

        if response.status().is_server_error() {
            println!("Internal server error");
        }
        unimplemented!()
    }
}

#[tokio::test]
async fn test_usos_client() {
    let client = Client::new("https://apps.usos.pw.edu.pl");
    let response = client.builder::<Value>("apiref/scopes".into()).send().await;

    println!("{:?}", response);
}
