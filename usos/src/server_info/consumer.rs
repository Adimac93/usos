use serde::Deserialize;
use serde_json::Value;
use usos_core::api::auth::AccessToken;
use usos_core::api::params::Params;
use usos_core::api::types::scopes::Scope;
use usos_core::api::types::time::UsosDateTime;
use usos_core::api::util::Selector;
use usos_core::client::CLIENT;
use usos_core::keys::ConsumerKey;

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
) {
}
