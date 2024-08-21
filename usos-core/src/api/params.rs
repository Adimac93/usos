use std::collections::HashMap;

use serde::Serialize;

use crate::keys::ConsumerKey;

use super::{auth::AccessToken, oauth1::authorize};

#[derive(Serialize)]
pub struct Params(HashMap<String, String>);

impl Params {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn add<T, U>(mut self, k: T, v: U) -> Self
    where
        (T, U): Param,
    {
        if let Some((k, v)) = (k, v).into_kv_pair() {
            self.0.insert(k, v);
        }

        self
    }

    pub fn sign(
        mut self,
        method: &str,
        uri: &str,
        consumer: Option<&ConsumerKey>,
        token: Option<&AccessToken>,
    ) -> Self {
        if let Some(consumer) = consumer {
            self.0 = authorize(method, uri, consumer, token, Some(self.0));
        }

        self
    }
}

impl From<HashMap<String, String>> for Params {
    fn from(value: HashMap<String, String>) -> Self {
        Self(value)
    }
}

pub trait Param {
    fn into_kv_pair(self) -> Option<(String, String)>;
}

impl<T> Param for (T, String)
where
    T: Into<String>,
{
    fn into_kv_pair(self) -> Option<(String, String)> {
        Some((self.0.into(), self.1.into()))
    }
}

impl<T> Param for (T, Option<String>)
where
    T: Into<String>,
{
    fn into_kv_pair(self) -> Option<(String, String)> {
        self.1.map(|x| (self.0.into(), x.into()))
    }
}

impl<T> Param for (T, &str)
where
    T: Into<String>,
{
    fn into_kv_pair(self) -> Option<(String, String)> {
        Some((self.0.into(), self.1.into()))
    }
}

impl<T> Param for (T, Option<&str>)
where
    T: Into<String>,
{
    fn into_kv_pair(self) -> Option<(String, String)> {
        self.1.map(|x| (self.0.into(), x.into()))
    }
}
