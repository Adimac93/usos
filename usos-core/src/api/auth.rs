use std::{
    collections::{HashMap, HashSet},
    io::Write,
    sync::Arc,
};

use anyhow::Context;
use secrecy::SecretString;

use crate::{
    api::util::parse_ampersand_params,
    api::{oauth1::authorize, types::scopes::Scope},
    client::CLIENT,
    errors::AppError,
    keys::ConsumerKey,
};

use super::types::scopes::Scopes;

pub struct OAuthRequestToken {
    token: String,
    secret: SecretString,
}

pub struct AccessToken {
    pub token: String,
    pub secret: SecretString,
}

pub async fn acquire_request_token(
    consumer_key: Arc<ConsumerKey>,
    callback: Option<String>,
    scopes: Scopes,
) -> crate::Result<OAuthRequestToken> {
    let callback = callback.unwrap_or("oob".into());

    let body = CLIENT
        .builder("oauth/request_token")
        .payload([("oauth_callback", callback), ("scopes", scopes.to_string())])
        .auth(Some(&*consumer_key), None)
        .request()
        .await?
        .text()
        .await?;
    println!("{:?}", *CLIENT);

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
    consumer_key: Arc<ConsumerKey>,
    request_token: OAuthRequestToken,
    verifier: impl Into<String>,
) -> crate::Result<AccessToken> {
    let body = CLIENT
        .builder("oauth/access_token")
        .payload([("oauth_verifier", verifier.into())])
        .auth(
            Some(&*consumer_key),
            Some(&AccessToken {
                token: request_token.token,
                secret: request_token.secret,
            }),
        )
        .request()
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

    println!("Access token: {oauth_token}");
    println!("Access token secret: {oauth_token_secret}");

    return Ok(AccessToken {
        token: oauth_token.into(),
        secret: oauth_token_secret.into(),
    });
}

#[cfg(test)]
async fn get_pin(oauth_token: String) -> String {
    println!(
        "Please visit the following URL to authorize the application: {}",
        format!(
            "{}services/oauth/authorize?oauth_token={oauth_token}",
            CLIENT.base_url()
        )
    );

    let mut buf = String::new();
    print!("Enter the verifier PIN: ");
    std::io::stdout().flush().unwrap();
    std::io::stdin().read_line(&mut buf).unwrap();
    let pin = buf.trim();

    return pin.into();
}

#[cfg(test)]
pub async fn get_access_token(consumer_key: ConsumerKey) -> Result<AccessToken, AppError> {
    let consumer_key = Arc::new(consumer_key);
    let request_token = acquire_request_token(
        consumer_key.clone(),
        None,
        Scopes::new(HashSet::from([Scope::Studies])),
    )
    .await?;

    let verifier = get_pin(request_token.token.clone()).await;

    Ok(acquire_access_token(consumer_key, request_token, verifier).await?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore]
    async fn acquire_request_token_is_successful() {
        dotenvy::dotenv().ok();
        let consumer_key = Arc::new(ConsumerKey::from_env().unwrap());
        acquire_request_token(consumer_key, None, Scopes::new(HashSet::new()))
            .await
            .unwrap();
    }

    #[tokio::test]
    #[ignore]
    async fn acquire_request_token_invalid_consumer_key() {
        dotenvy::dotenv().ok();
        let mut consumer_key = ConsumerKey::from_env().unwrap();
        consumer_key.key.push('a');

        let res =
            acquire_request_token(Arc::new(consumer_key), None, Scopes::new(HashSet::new())).await;

        assert!(res.is_err());
    }

    #[tokio::test]
    #[ignore = "requires user interaction"]
    async fn oauth_flow_no_callback_provided() {
        dotenvy::dotenv().ok();
        let consumer_key = ConsumerKey::from_env().unwrap();
        get_access_token(consumer_key).await.unwrap();
    }
}
