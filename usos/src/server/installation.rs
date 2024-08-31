use serde::Deserialize;
use serde_json::Value;

use usos_core::{
    api::{errors::UsosError, types::language::LanguageDictionary},
    client::{UsosDebug, CLIENT},
};

/// apisrv/installation
///
/// Consumer: ignored
///
/// Token: ignored
///
/// Scopes: n/a
///
/// SSL: not required
pub async fn get_installation() -> Result<Installation, UsosError> {
    let response = CLIENT
        .get("https://apps.usos.pwr.edu.pl/services/apisrv/installation")
        .query(&[("fields", "base_url|version|institution_name|contact_emails|machine_version|usos_schema_version|institution|schac_id|mcards_support")])
        .send()
        .await
        .unwrap().debug().await.unwrap();

    // if response.status().is_client_error() {
    //     let error = response.json::<UsosError>().await.unwrap();
    //     return Err(error);
    // }
    let json = response.json::<Installation>().await.unwrap();
    Ok(json)
}

#[derive(Debug, Deserialize)]
pub struct Installation {
    base_url: String,
    version: String,
    institution_name: Option<LanguageDictionary>,
    contact_emails: Vec<String>,
    machine_version: String,
    usos_schema_version: String,
    // supports sub selection of Faculty fields
    institution: PrimaryFaculty,
    schac_id: String,
    mcards_support: bool,
}

#[derive(Debug, Deserialize)]
struct PrimaryFaculty {
    id: String,
    name: LanguageDictionary,
}

#[tokio::test]
async fn test_get_installation() {
    let installations = get_installation().await;
    println!("{installations:#?}");
}
