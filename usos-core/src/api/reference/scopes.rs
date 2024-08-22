use serde::Deserialize;

use crate::{
    api::scopes::Scope,
    client::{UsosUri, CLIENT},
};

/// apiref/scopes
///
/// Consumer: ignored
///
/// Token: ignored
///
/// Scopes: n/a
///
/// SSL: not required
pub async fn get_scopes() -> Vec<ApiScope> {
    let response = CLIENT
        .get(UsosUri::with_path("services/apiref/scopes"))
        .send()
        .await
        .unwrap();

    let mut scopes: Vec<ApiScope> = response.json().await.unwrap();

    scopes.iter_mut().for_each(|scope| {
        let mut formatted_description = String::new();
        let chars = scope.description.trim().chars();
        let mut prev_char = '\0';
        for current_char in chars {
            if !current_char.is_whitespace() {
                if prev_char.is_whitespace() {
                    formatted_description.push(' ');
                }
                formatted_description.push(current_char);
            }

            prev_char = current_char;
        }

        scope.description = formatted_description;
    });

    scopes
}

#[tokio::test]
#[ignore]
async fn test_get_scopes() {
    let scopes = get_scopes().await;
    println!("{scopes:?}");
}

#[derive(Debug, Deserialize)]
pub struct ApiScope {
    #[serde(rename = "key")]
    scope: Scope,
    #[serde(rename = "developers_description")]
    description: String,
}
