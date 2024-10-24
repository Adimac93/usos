//! Utilities for authorizing requests.

use base64::Engine;
use percent_encoding::{AsciiSet, NON_ALPHANUMERIC};
use rand::distributions::{Alphanumeric, Distribution};
use rand::thread_rng;
use ring::hmac::{self, HMAC_SHA1_FOR_LEGACY_USE_ONLY};
use secrecy::{ExposeSecret, SecretString};
use std::collections::{BTreeMap, HashMap};

const NONCE_LENGTH: usize = 32;
const OAUTH_VERSION: &str = "1.0";
use crate::keys::ConsumerKey;

use super::auth::AccessToken;
use super::params::Params;

fn rand_alphanumeric_string(target_len: usize) -> String {
    Alphanumeric
        .sample_iter(thread_rng())
        .take(target_len)
        .map(|a| a as char)
        .collect()
}

/// Generates OAuth 1.0a authorization parameters for a request.
///
/// This function handles the core functionality - it receives request parameters
/// and appends another parameters required for OAuth1.0a authorization.
///
/// # Arguments
///
/// * `method` - The HTTP method of the request (e.g., "GET", "POST").
/// * `uri` - The URI of the request as a string reference.
/// * `consumer` - The `ConsumerKey` containing the key and secret for the application.
/// * `token` - An optional `AccessToken` containing the token and secret for user authentication.
/// * `params` - Additional parameters to include in the OAuth signature, convertible to `Params`.
///
/// # Returns
///
/// Returns a `BTreeMap<String, String>` containing the OAuth parameters, including provided parameters,
/// generated authorization parameters and the signature. These parameters should be included
/// in the form body or query string of the HTTP request.
pub fn authorize<'a>(
    method: &str,
    uri: impl AsRef<str>,
    consumer: &ConsumerKey,
    token: Option<&AccessToken>,
    params: impl Into<Params>,
) -> BTreeMap<String, String> {
    let mut params = params.into();
    let timestamp = time::OffsetDateTime::now_utc().unix_timestamp().to_string();

    let nonce: String = rand_alphanumeric_string(NONCE_LENGTH);

    params.insert("oauth_consumer_key".into(), consumer.key.clone());
    params.insert("oauth_nonce".into(), nonce);
    params.insert("oauth_signature_method".into(), "HMAC-SHA1".into());
    params.insert("oauth_timestamp".into(), timestamp);
    if let Some(tk) = token {
        params.insert("oauth_token".into(), tk.token.as_str().into());
    }
    params.insert("oauth_version".into(), OAUTH_VERSION.into());

    let signature = gen_signature(
        method,
        uri.as_ref(),
        &to_query(&params),
        consumer.secret.expose_secret(),
        token.map(|t| t.secret.expose_secret().as_ref()),
    );

    params.insert("oauth_signature".into(), signature.into());

    params.0
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

fn to_query(params: &BTreeMap<String, String>) -> String {
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
