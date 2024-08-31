use anyhow::Context;
use serde::Deserialize;
use serde_json::Value;
use usos_core::{api::types::time::UsosPreciseDateTime, client::CLIENT};

/// services/apisrv/now
///
/// Consumer: ignored
///
/// Token: ignored
///
/// Scopes: []
///
/// SSL: false
pub async fn get_usos_server_time() {}
