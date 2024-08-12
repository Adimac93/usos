use std::{
    collections::{HashMap, HashSet},
    io::Write,
};

use anyhow::Context;

use crate::{
    api::{
        oauth1::{authorize, KeyPair},
        scopes::Scope,
    },
    client::{UsosUri, CLIENT},
    errors::AppError,
    keys::ConsumerKey,
};

use super::scopes::Scopes;

pub struct OAuthToken {
    token: String,
    secret: String,
}

pub async fn acquire_token(
    consumer_key: &ConsumerKey,
    callback: Option<String>,
    scopes: Scopes,
) -> crate::Result<OAuthToken> {
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

    let response = CLIENT.post(&url).form(&authorization).send().await?;

    // TODO: util function to read USOS responses
    let body = response.text().await?;
    println!("{body}");
    let params = body
        .split('&')
        .map(|keyval| {
            Ok(keyval
                .split_once('=')
                .ok_or(AppError::usos("Invalid return params formatting"))?)
        })
        .collect::<crate::Result<HashMap<&str, &str>>>()?;

    let oauth_token = *params
        .get("oauth_token")
        .ok_or(AppError::usos("Invalid return param key"))?;
    let oauth_token_secret = *params
        .get("oauth_token_secret")
        .ok_or(AppError::usos("Invalid return param key"))?;
    let oauth_callback_confirmed = *params
        .get("oauth_callback_confirmed")
        .ok_or(AppError::usos("Invalid return param key"))?;

    if &callback == "oob" {
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

        let url = UsosUri::with_path("/services/oauth/access_token");
        let response = CLIENT
            .post(&url)
            .form(&authorize(
                "POST",
                &url,
                consumer_key,
                Some(&KeyPair::new(oauth_token.into(), oauth_token_secret.into())),
                Some(HashMap::from([("oauth_verifier".into(), pin.into())])),
            ))
            .send()
            .await?;

        let body = response.text().await?;
        let keys = body
            .split('&')
            .map(|keyval| {
                Ok(keyval
                    .split_once('=')
                    .ok_or(AppError::usos("Invalid return params formatting"))?)
            })
            .collect::<crate::Result<HashMap<&str, &str>>>()?;
        let oauth_token = *keys
            .get("oauth_token")
            .ok_or(AppError::usos("Invalid return param key"))?;
        let oauth_token_secret = *keys
            .get("oauth_token_secret")
            .ok_or(AppError::usos("Invalid return param key"))?;

        println!("User OAuth token: {oauth_token}");
        println!("User OAuth token secret: {oauth_token_secret}");

        return Ok(OAuthToken {
            token: oauth_token.into(),
            secret: oauth_token_secret.into(),
        });
    }

    Ok(OAuthToken {
        token: "".into(),
        secret: "".into(),
    })
}

#[tokio::test]
async fn test_acquire_token() {
    dotenvy::dotenv().ok();
    let consumer_key = ConsumerKey::from_env().unwrap();
    acquire_token(
        &consumer_key,
        None,
        Scopes::new(HashSet::from([Scope::Studies])),
    )
    .await
    .unwrap();
}
