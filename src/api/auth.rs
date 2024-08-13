use std::{
    collections::{HashMap, HashSet},
    io::Write,
};

use anyhow::Context;
use secrecy::SecretString;

use crate::{
    api::{
        oauth1::{authorize, KeyPair},
        scopes::Scope,
    },
    client::{UsosUri, CLIENT},
    errors::AppError,
    keys::ConsumerKey,
    util::{parse_ampersand_params, ToAppResult},
};

use super::scopes::Scopes;

pub struct OAuthRequestToken {
    token: String,
    secret: SecretString,
}

pub struct OAuthAccessToken {
    token: String,
    secret: SecretString,
}

pub async fn acquire_request_token(
    consumer_key: &ConsumerKey,
    callback: Option<String>,
    scopes: Scopes,
) -> crate::Result<OAuthRequestToken> {
    let callback = callback.unwrap_or("oob".into());
    let url = UsosUri::with_path("/services/oauth/request_token");
    let authorization = authorize(
        "POST",
        &url,
        consumer_key,
        None,
        Some(HashMap::from([
            ("oauth_callback".into(), callback.clone()),
            ("scopes".into(), scopes.to_string()),
        ])),
    );

    let body = CLIENT
        .post(&url)
        .form(&authorization)
        .send()
        .await?
        .to_app_result()
        .await?
        .text()
        .await?;

    let mut params = parse_ampersand_params(body)?;

    let oauth_token = params
        .remove("oauth_token")
        .context("Invalid return param key")?;
    let oauth_token_secret = params
        .remove("oauth_token_secret")
        .context("Invalid return param key")?;
    let _oauth_callback_confirmed = params
        .remove("oauth_callback_confirmed")
        .context("Invalid return param key")?;

    Ok(OAuthRequestToken {
        token: oauth_token.into(),
        secret: oauth_token_secret.into(),
    })
}

pub async fn acquire_access_token(
    consumer_key: &ConsumerKey,
    request_token: OAuthRequestToken,
    verifier: impl Into<String>,
) -> crate::Result<OAuthAccessToken> {
    let url = UsosUri::with_path("/services/oauth/access_token");
    let body = CLIENT
        .post(&url)
        .form(&authorize(
            "POST",
            &url,
            consumer_key,
            Some(&KeyPair::new(
                request_token.token.into(),
                request_token.secret,
            )),
            Some(HashMap::from([("oauth_verifier".into(), verifier.into())])),
        ))
        .send()
        .await?
        .to_app_result()
        .await?
        .text()
        .await?;

    let mut params = parse_ampersand_params(body)?;

    let oauth_token = params
        .remove("oauth_token")
        .context("Invalid return param key")?;
    let oauth_token_secret = params
        .remove("oauth_token_secret")
        .context("Invalid return param key")?;

    return Ok(OAuthAccessToken {
        token: oauth_token.into(),
        secret: oauth_token_secret.into(),
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn get_pin(oauth_token: String) -> String {
        println!(
            "Please visit the following URL to authorize the application: {}",
            UsosUri::with_path(&format!(
                "/services/oauth/authorize?oauth_token={oauth_token}"
            ))
        );

        let mut buf = String::new();
        print!("Enter the verifier PIN: ");
        std::io::stdout().flush().unwrap();
        std::io::stdin().read_line(&mut buf).unwrap();
        let pin = buf.trim();

        return pin.into();
    }

    #[tokio::test]
    async fn acquire_request_token_is_successful() {
        dotenvy::dotenv().ok();
        let consumer_key = ConsumerKey::from_env().unwrap();
        acquire_request_token(&consumer_key, None, Scopes::new(HashSet::new()))
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn acquire_request_token_invalid_consumer_key() {
        dotenvy::dotenv().ok();
        let mut consumer_key = ConsumerKey::from_env().unwrap();
        consumer_key.key.push('a');

        let res = acquire_request_token(&consumer_key, None, Scopes::new(HashSet::new())).await;

        assert!(res.is_err());
    }

    #[tokio::test]
    #[ignore = "requires user interaction"]
    async fn oauth_flow_no_callback_provided() {
        dotenvy::dotenv().ok();
        let consumer_key = ConsumerKey::from_env().unwrap();
        let request_token = acquire_request_token(
            &consumer_key,
            None,
            Scopes::new(HashSet::from([Scope::Studies])),
        )
        .await
        .unwrap();

        let verifier = get_pin(request_token.token.clone()).await;

        let _access_token = acquire_access_token(&consumer_key, request_token, verifier)
            .await
            .unwrap();
    }
}
