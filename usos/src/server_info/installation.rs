use serde::Deserialize;
use serde_json::Value;
use usos_core::{
    api::util::Selector,
    api::{params::Params, types::language::LanguageDictionary},
    client::CLIENT,
};

use crate::faculties::faculty::Faculty;

#[derive(Deserialize, Debug)]
pub struct Installation {
    base_url: String,
    version: String,
    #[serde(rename = "machine_version")]
    machine_readable_version: String,
    usos_schema_version: String,
    institution_name: LanguageDictionary,
    #[serde(rename = "institution")]
    primary_faculty: Faculty,
    contact_emails: Vec<String>,
    schac_id: String,
    mcards_support: bool,
}

// Fields: base_url|version|machine_version|usos_schema_version|institution_name|institution[id|name|profile_url|homepage_url|phone_numbers|phone_numbers2|postal_address|email|is_public|static_map_urls]|contact_emails|schac_id|mcards_support
/// services/apisrv/installation
///
/// Consumer: ignored
///
/// Token: ignored
///
/// Scopes: []
///
/// SSL: false
pub async fn get_installation_info(fields: Option<Selector>) {}
