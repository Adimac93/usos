use serde::Deserialize;
use serde_json::{json, Value};

use crate::{api::scopes::Scope, client::CLIENT};
/// apiref/method
///
/// Consumer: ignored
///
/// Token: ignored
///
/// Scopes: n/a
///
/// SSL: not required
pub async fn get_scopes(method_name: &str) -> Vec<ApiScope> {
    let response = CLIENT
        .get(format!(
            "https://apps.usos.pwr.edu.pl/services/apiref/scopes"
        ))
        .send()
        .await
        .unwrap();

    let mut json = response.json::<Vec<ApiScope>>().await.unwrap();
    json.iter_mut().for_each(|scope| {
        let mut formatted_descsription = String::new();
        let mut chars = scope.description.trim().chars();
        let mut prev_char = '\0';
        while let Some(current_char) = chars.next() {
            if !current_char.is_whitespace() && current_char != '\n' {
                if (prev_char.is_whitespace() || prev_char == '\n') {
                    formatted_descsription.push(' ');
                }
                formatted_descsription.push(current_char);
            }

            prev_char = current_char;
        }

        scope.description = formatted_descsription.clone();
    });

    json
}

#[tokio::test]
async fn test_get_scopes() {
    let scopes = get_scopes("services/apiref/method").await;
    println!("{scopes:?}");
}

#[derive(Debug, Deserialize)]
struct ApiScope {
    #[serde(rename = "key")]
    scope: Scope,
    #[serde(rename = "developers_description")]
    description: String,
}
