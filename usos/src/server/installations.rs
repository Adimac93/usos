use serde::Deserialize;
use usos_core::{api::types::language::LanguageDictionary, client::CLIENT};

/// apisrv/installations
/// Consumer: ignored
///
/// Token: ignored
///
/// Scopes: n/a
///
/// SSL: not required
pub async fn get_installations() -> Vec<BulkInstallation> {
    let response = CLIENT
        .get("https://apps.usos.pwr.edu.pl/services/apisrv/installations")
        .send()
        .await
        .unwrap();

    let mut json = response.json::<Vec<BulkInstallation>>().await.unwrap();
    json
}

#[derive(Debug, Deserialize)]
struct BulkInstallation {
    base_url: String,
    version: String,
    institution_name: LanguageDictionary,
    contact_emails: Vec<String>,
}

#[tokio::test]
async fn test_get_installations() {
    let installations = get_installations().await;
    println!("{installations:#?}");
}
