use base64::Engine;
use percent_encoding::{AsciiSet, NON_ALPHANUMERIC};
use rand::distributions::{Alphanumeric, Distribution};
use rand::thread_rng;
use ring::hmac::{self, HMAC_SHA1_FOR_LEGACY_USE_ONLY};
use std::collections::HashMap;

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

struct OAuth {}
pub fn authorize<'a>(
    method: &str,
    uri: &str,
    consumer: &KeyPair,
    token: Option<&KeyPair>,
    params: Option<HashMap<String, String>>,
) -> HashMap<String, String> {
    let mut params = params.unwrap_or_else(HashMap::new);
    let timestamp = time::OffsetDateTime::now_utc().unix_timestamp().to_string();

    let nonce: String = Alphanumeric
        .sample_iter(thread_rng())
        .take(32)
        .map(|a| char::from_u32(a as u32).unwrap())
        .collect();

    params.insert("oauth_consumer_key".into(), consumer.key.clone().into());
    params.insert("oauth_nonce".into(), nonce.into());
    params.insert("oauth_signature_method".into(), "HMAC-SHA1".into());
    params.insert("oauth_timestamp".into(), timestamp.into());
    params.insert("oauth_version".into(), "1.0".into());
    if let Some(tk) = token {
        params.insert("oauth_token".into(), tk.key.as_str().into());
    }

    let signature = gen_signature(
        method,
        uri,
        &to_query(&params),
        &consumer.secret,
        token.map(|t| t.secret.as_ref()),
    );

    params.insert("oauth_signature".into(), signature.into());

    // let mut pairs = params
    //     .iter()
    //     .filter(|&(k, _)| k.starts_with("oauth_"))
    //     .map(|(k, v)| format!("{}=\"{}\"", k, encode(v)))
    //     .collect::<Vec<_>>();

    params
}

#[derive(Copy, Clone)]
struct StrictEncodeSet;

// Encode all but the unreserved characters defined in
// RFC 3986, section 2.3. "Unreserved Characters"
// https://tools.ietf.org/html/rfc3986#page-12
//
// This is required by
// OAuth Core 1.0, section 5.1. "Parameter Encoding"
// https://oauth.net/core/1.0/#encoding_parameters
static STRICT_ENCODE_SET: AsciiSet = NON_ALPHANUMERIC
    .remove(b'-')
    .remove(b'.')
    .remove(b'_')
    .remove(b'~');

fn encode(s: &str) -> String {
    percent_encoding::percent_encode(s.as_bytes(), &STRICT_ENCODE_SET).collect()
}

fn to_query(params: &HashMap<String, String>) -> String {
    let mut pairs: Vec<_> = params
        .iter()
        .map(|(k, v)| format!("{}={}", encode(k), encode(v)))
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
    let base = format!("{}&{}&{}", encode(method), encode(uri), encode(query));

    let key = format!(
        "{}&{}",
        encode(consumer_secret),
        encode(token_secret.unwrap_or(""))
    );

    let s_key = hmac::Key::new(HMAC_SHA1_FOR_LEGACY_USE_ONLY, key.as_ref());
    let signature = hmac::sign(&s_key, base.as_bytes());

    base64::engine::general_purpose::STANDARD.encode(signature)
}
