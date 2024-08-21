use crate::{
    api::types::time::UsosPreciseDateTime,
    client::{UsosUri, CLIENT},
    util::Process,
};
use anyhow::Context;
use serde::Deserialize;
use serde_json::Value;

/// services/apisrv/now
///
/// Consumer: ignored
///
/// Token: ignored
///
/// Scopes: []
///
/// SSL: false
pub async fn get_usos_server_time() -> crate::Result<UsosPreciseDateTime> {
    let url = UsosUri::with_path("services/apisrv/now");

    let body = UsosPreciseDateTime::deserialize(CLIENT.get(&url).process_as_json::<Value>().await?)
        .context("Failed to deserialize USOS millisecond precision date time")?;

    Ok(body)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn health_check() {
        let res = get_usos_server_time().await.unwrap();
        println!("{res}");
    }
}
