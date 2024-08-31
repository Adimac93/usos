use serde::Deserialize;

use usos_core::{api::types::language::LanguageDictionary, client::CLIENT};

#[derive(Deserialize, Debug)]
pub struct InstallationListItem {
    base_url: String,
    contact_emails: Vec<String>,
    institution_name: LanguageDictionary,
    version: Option<String>,
}

/// services/apisrv/installations
///
/// Consumer: ignored
///
/// Token: ignored
///
/// Scopes: []
///
/// SSL: false
pub async fn get_installations() {}
