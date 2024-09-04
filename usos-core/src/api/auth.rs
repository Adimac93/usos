use std::{
    collections::{HashMap, HashSet},
    io::Write,
    sync::Arc,
};

use anyhow::Context;
use secrecy::SecretString;

use crate::{
    api::{oauth1::authorize, types::scopes::Scope, util::parse_ampersand_params},
    client::{Client, CLIENT},
    errors::AppError,
    keys::ConsumerKey,
};

use super::types::scopes::Scopes;

/// Token used for requesting the OAuth1.0a [`AccessToken`]
///
/// Its sole purpose is to pass it as a parameter to the `services/oauth/access_token` USOS API endpoint (see [`acquire_access_token`])
/// to request an access token. For more details, see [the USOS API reference](https://apps.usos.pw.edu.pl/developers/api/authorization/).
#[derive(Debug, Clone)]
pub struct OAuthRequestToken {
    token: String,
    secret: SecretString,
}

/// Token that identifies an application user (a student).
///
/// This token is required for some USOS API endpoints.
/// It can be obtained by requesting it using a [`OAuthRequestToken`] and calling the `services/oauth/access_token` endpoint (see [`acquire_access_token`]).
/// For more details, see [the USOS API reference](https://apps.usos.pw.edu.pl/developers/api/authorization/).
#[derive(Debug, Clone)]
pub struct AccessToken {
    pub token: String,
    pub secret: SecretString,
}

/// Acquires the request token, calling `services/oauth/request_token`.
///
/// This function initiates the OAuth 1.0a authorization flow by requesting a temporary request token
/// from the `oauth/request_token` endpoint. The request token is required to obtain user authorization
/// and later exchange it for an access token.
///
/// Since this is the first step of the OAuth1.0a flow, after receiving the request token
/// the application should redirect the user to the `services/oauth/authorize` endpoint,
/// where the user can enter its credentials, and then either expect a callback from that
/// endpoint or a code that the user has to enter, depending if the callback url has been provided.
///
/// For more details, see [the USOS API reference](https://apps.usos.pw.edu.pl/developers/api/authorization/).
///
/// # Arguments
///
/// * `client` - The `reqwest::Client` used to make the HTTP request.
/// * `callback` - An optional callback URL where the user will be redirected after authorization.
///   If not provided, the default value `"oob"` (Out-Of-Band) will be used.
/// * `scopes` - A set of scopes that define the access permissions being requested.
pub async fn acquire_request_token(
    client: &Client,
    callback: Option<String>,
    scopes: Scopes,
) -> crate::Result<OAuthRequestToken> {
    let callback = callback.unwrap_or("oob".into());

    let body = CLIENT
        .builder("oauth/request_token")
        .payload([("oauth_callback", callback), ("scopes", scopes.to_string())])
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

/// Generates an access token, calling `services/oauth/access_token`.
///
/// This is the final step of the OAuth1.0a authorization flow.
/// The request token should be obtained from the [`acquire_request_token`] function, which calls `services/oauth/request_token`.
/// The verifier (verification code) should be received from the callback from the USOS API or provided manually by the application user,
/// depending on whether the callback_url has been provided while requesting a request token.
///
/// For more details, see [the USOS API reference](https://apps.usos.pw.edu.pl/developers/api/authorization/).
///
/// # Arguments
///
/// * `client` - The `reqwest::Client` used to make the HTTP request.
/// * `request_token` - The OAuth1.0a request token used to request an access token.
/// * `verifier` - The code received from the USOS API as parameters of a callback
///     request or the code submitted by the user.
pub async fn acquire_access_token(
    client: &Client,
    request_token: OAuthRequestToken,
    verifier: impl Into<String>,
) -> crate::Result<AccessToken> {
    let body = client
        .builder("oauth/access_token")
        .payload([("oauth_verifier", verifier.into())])
        .auth(&AccessToken {
            token: request_token.token,
            secret: request_token.secret,
        })
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
pub async fn get_access_token(client: &Client) -> Result<AccessToken, AppError> {
    let request_token =
        acquire_request_token(client, None, Scopes::new(HashSet::from([Scope::Studies]))).await?;

    let verifier = get_pin(request_token.token.clone()).await;

    Ok(acquire_access_token(client, request_token, verifier).await?)
}

#[cfg(test)]
mod tests {

    use reqwest::Url;
    use secrecy::Secret;

    use super::*;

    #[tokio::test]
    #[ignore]
    async fn acquire_request_token_is_successful() {
        dotenvy::dotenv().ok();
        let client = Client::new(Url::parse("https://apps.usos.pwr.edu.pl").unwrap())
            .authorized_from_env()
            .unwrap();
        acquire_request_token(&client, None, Scopes::new(HashSet::new()))
            .await
            .unwrap();
    }

    #[tokio::test]
    #[ignore]
    async fn acquire_request_token_invalid_consumer_key() {
        dotenvy::dotenv().ok();
        let mut consumer_key =
            ConsumerKey::new("key".into(), Secret::from(String::from("secret")), None);
        let client = Client::new(Url::parse("https://apps.usos.pwr.edu.pl").unwrap())
            .authorized_from_key(consumer_key);
        let res = acquire_request_token(&client, None, Scopes::new(HashSet::new())).await;

        assert!(res.is_err());
    }

    #[tokio::test]
    #[ignore = "requires user interaction"]
    async fn oauth_flow_no_callback_provided() {
        dotenvy::dotenv().ok();
        let client = Client::new(Url::parse("https://apps.usos.pwr.edu.pl").unwrap())
            .authorized_from_env()
            .unwrap();
        get_access_token(&client).await.unwrap();
    }
}
