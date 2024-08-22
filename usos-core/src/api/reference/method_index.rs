use serde::Deserialize;

use crate::client::{UsosUri, CLIENT};

/// apiref/method_index
///
/// Consumer: ignored
///
/// Token: ignored
///
/// Scopes: n/a
///
/// SSL: not required
pub async fn get_method_index() -> Vec<MethodBrief> {
    let response = CLIENT
        .get(UsosUri::with_path("services/apiref/method_index"))
        .send()
        .await
        .unwrap();

    let mut json = response.json::<Vec<MethodBrief>>().await.unwrap();
    json.iter_mut().for_each(|method| {
        method.name = method.name.split_once('/').unwrap().1.to_string();
        method.brief_description = method.brief_description.trim().to_string();
    });
    json
}

#[tokio::test]
#[ignore]
async fn test_get_method_index() {
    let methods = get_method_index().await;
    println!("{methods:#?}");
}

#[derive(Debug, Deserialize)]
pub struct MethodBrief {
    /// Method name without "services/" prefix
    pub name: String,
    pub brief_description: String,
}
