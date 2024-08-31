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
pub async fn get_installations() {}

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
