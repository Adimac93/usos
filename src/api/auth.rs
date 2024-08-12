use std::{
    collections::{HashMap, HashSet},
    io::Write,
};

use crate::{
    api::{
        oauth1::{authorize, KeyPair},
        scopes::Scope,
    },
    client::{BASE_URL, CLIENT},
    keys::ConsumerKey,
};

use super::scopes::Scopes;

const CONSUMER_KEY_NAME: &str = "USOS_CONSUMER_KEY";
const CONSUMER_SECRET_NAME: &str = "USOS_CONSUMER_SECRET";

pub async fn acquire_token(consumer_key: &ConsumerKey, callback: Option<String>, scopes: Scopes) {
    let callback = callback.unwrap_or("oob".into());
    let url = format!("{BASE_URL}/services/oauth/request_token");
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

    let response = CLIENT.post(&url).form(&authorization).send().await.unwrap();

    let body = response.text().await.unwrap();
    println!("{body}");
    let mut params = body
        .split('&')
        .map(|keyval| keyval.split_once('=').unwrap().1);

    let oauth_token = params.next().unwrap();
    let oauth_token_secret = params.next().unwrap();
    let oauth_callback_confirmed = params.next().unwrap();

    if &callback == "oob" {
        println!("Please visit the following URL to authorize the application: https://apps.usos.pwr.edu.pl/services/oauth/authorize?oauth_token={oauth_token}");
        let mut buf = String::new();
        print!("Enter the verifier PIN: ");
        std::io::stdout().flush().unwrap();
        std::io::stdin().read_line(&mut buf).unwrap();
        let pin = buf.trim();
        let url = format!("{BASE_URL}/services/oauth/access_token");
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
            .await
            .unwrap();
        let body = response.text().await.unwrap();
        let mut keys = body
            .split('&')
            .map(|keyval| keyval.split_once('=').unwrap().1);
        let oauth_token = keys.next().unwrap();
        let oauth_token_secret = keys.next().unwrap();
        println!("User OAuth token: {oauth_token}");
        println!("User OAuth token secret: {oauth_token_secret}");
    }
}

#[tokio::test]
async fn test_acquire_token() {
    dotenvy::dotenv().ok();
    let consumer_key = ConsumerKey::from_env();
    acquire_token(
        &consumer_key,
        None,
        Scopes::new(HashSet::from([Scope::Studies])),
    )
    .await;
}
