use crate::api::auth::AccessToken;
use crate::api::params::Params;
use crate::api::scopes::Scope;
use crate::api::types::time::UsosDateTime;
use crate::api::util::Selector;
use crate::keys::ConsumerKey;
use crate::{
    api::util::Process,
    client::{UsosUri, CLIENT},
};
use serde::Deserialize;
use serde_json::Value;

#[derive(Deserialize, Debug)]
pub struct ConsumerInfo {
    name: String,
    url: Option<String>,
    email: String,
    date_registered: UsosDateTime,
    administrative_methods: Vec<String>,
    token_scopes: Option<Vec<Scope>>,
}

// Fields: name|url|email|date_registered|administrative_methods|token_scopes
/// services/apisrv/consumer
///
/// Consumer: required
///
/// Token: optional
///
/// Scopes: []
///
/// SSL: false
pub async fn get_consumer_info(
    consumer_key: &ConsumerKey,
    token: Option<AccessToken>,
    fields: impl Into<Selector>,
) -> crate::Result<Value> {
    let url = UsosUri::with_path("services/apisrv/consumer");
    let mut params = Params::new();

    params = params.add("fields", fields.into());

    params = params.sign("GET", &url, Some(consumer_key), token.as_ref());

    let body = CLIENT.get(&url).query(&params).process_as_json().await?;
    Ok(body)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore]
    async fn health_check() {
        dotenvy::dotenv().ok();
        let consumer_key = ConsumerKey::from_env().unwrap();

        let fields =
            "name|url|email|date_registered|administrative_methods|token_scopes".to_string();
        let res = get_consumer_info(&consumer_key, None, fields)
            .await
            .unwrap();
        println!("{res:?}");
    }
}
