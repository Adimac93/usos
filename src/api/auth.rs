use std::collections::HashMap;

use crate::{
    api::oauth1::{authorize, KeyPair},
    client::{BASE_URL, CLIENT},
};

const CONSUMER_KEY_NAME: &str = "USOS_CONSUMER_KEY";
const CONSUMER_SECRET_NAME: &str = "USOS_CONSUMER_SECRET";

fn from_env() -> (String, String) {
    let key = std::env::var(CONSUMER_KEY_NAME).unwrap();
    let secret = std::env::var(CONSUMER_SECRET_NAME).unwrap();
    (key, secret)
}

pub async fn acquire_token() {
    let (key, secret) = from_env();
    let url = format!("{BASE_URL}/services/oauth/request_token");
    let authorization = authorize(
        "POST",
        &url,
        &KeyPair::new(key, secret),
        None,
        Some(HashMap::from([
            ("oauth_callback".into(), "oob".into()),
            ("scopes".into(), "studies".into()),
        ])),
    );

    let response = CLIENT
        .post(&url)
        // .bearer_auth(authorization)
        .form(&authorization)
        .send()
        .await
        .unwrap();

    let body = response.text().await.unwrap();

    let mut params = body
        .split('&')
        .map(|keyval| keyval.split_once('=').unwrap().1);
    let oauth_token = params.next().unwrap();
    let oauth_token_secret = params.next().unwrap();
    let oauth_callback_confirmed = params.next().unwrap();

    println!("{body}");
}

// async fn acquire_access_token() {
//     let url = format!("{BASE_URL}/services/oauth/access_token");

//     CLIENT.
// }
#[tokio::test]
async fn test_acquire_token() {
    dotenvy::dotenv().ok();
    acquire_token().await;
}
