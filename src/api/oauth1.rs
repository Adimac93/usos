use base64::Engine;
use percent_encoding::{AsciiSet, NON_ALPHANUMERIC};
use rand::distributions::{Alphanumeric, Distribution};
use rand::thread_rng;
use ring::hmac::{self, HMAC_SHA1_FOR_LEGACY_USE_ONLY};
use secrecy::ExposeSecret;
use std::collections::HashMap;

const NONCE_LENGTH: usize = 32;
const OAUTH_VERSION: &str = "1.0";
use crate::keys::ConsumerKey;

#[derive(Clone, Debug)]
pub struct KeyPair {
    pub key: String,
    pub secret: String,
}

impl KeyPair {
    pub fn new(key: String, secret: String) -> Self {
        KeyPair { key, secret }
    }
}

fn rand_alphanumeric_string(target_len: usize) -> String {
    Alphanumeric
        .sample_iter(thread_rng())
        .take(target_len)
        .map(|a| a as char)
        .collect()
}

pub fn authorize<'a>(
    method: &str,
    uri: &str,
    consumer: &ConsumerKey,
    token: Option<&KeyPair>,
    params: Option<HashMap<String, String>>,
) -> HashMap<String, String> {
    let mut params = params.unwrap_or_else(HashMap::new);
    let timestamp = time::OffsetDateTime::now_utc().unix_timestamp().to_string();

    let nonce: String = rand_alphanumeric_string(NONCE_LENGTH);

    params.insert("oauth_consumer_key".into(), consumer.key.clone());
    params.insert("oauth_nonce".into(), nonce);
    params.insert("oauth_signature_method".into(), "HMAC-SHA1".into());
    params.insert("oauth_timestamp".into(), timestamp);
    params.insert("oauth_version".into(), OAUTH_VERSION.into());
    if let Some(tk) = token {
        params.insert("oauth_token".into(), tk.key.as_str().into());
    }

    let signature = gen_signature(
        method,
        uri,
        &to_query(&params),
        consumer.secret.expose_secret(),
        token.map(|t| t.secret.as_ref()),
    );

    params.insert("oauth_signature".into(), signature.into());

    params
}

// Encode all but the unreserved characters defined in
// RFC 3986, section 2.3. "Unreserved Characters"
// https://tools.ietf.org/html/rfc3986#page-12
//
// This is required by
// OAuth Core 1.0, section 5.1. "Parameter Encoding"
// https://oauth.net/core/1.0/#encoding_parameters
const STRICT_ENCODE_SET: AsciiSet = NON_ALPHANUMERIC
    .remove(b'-')
    .remove(b'.')
    .remove(b'_')
    .remove(b'~');

fn to_url_encoded(s: &str) -> String {
    percent_encoding::percent_encode(s.as_bytes(), &STRICT_ENCODE_SET).collect()
}

fn to_query(params: &HashMap<String, String>) -> String {
    let mut pairs: Vec<_> = params
        .iter()
        .map(|(k, v)| format!("{}={}", to_url_encoded(k), to_url_encoded(v)))
        .collect();

    pairs.sort();
    pairs.join("&")
}

fn gen_signature(
    method: &str,
    uri: &str,
    query: &str,
    consumer_secret: &str,
    token_secret: Option<&str>,
) -> String {
    let base = format!(
        "{}&{}&{}",
        to_url_encoded(method),
        to_url_encoded(uri),
        to_url_encoded(query)
    );

    let key = format!(
        "{}&{}",
        to_url_encoded(consumer_secret),
        to_url_encoded(token_secret.unwrap_or(""))
    );

    let s_key = hmac::Key::new(HMAC_SHA1_FOR_LEGACY_USE_ONLY, key.as_ref());
    let signature = hmac::sign(&s_key, base.as_bytes());

    base64::engine::general_purpose::STANDARD.encode(signature)
}
