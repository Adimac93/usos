use crate::{
    api::{faculties::faculty::Faculty, params::Params, types::language::LanguageDictionary},
    client::{UsosUri, CLIENT},
    util::{Process, Selector},
};
use serde::Deserialize;
use serde_json::Value;

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
pub async fn get_installation_info(fields: Option<Selector>) -> crate::Result<Value> {
    let url = UsosUri::with_path("services/apisrv/installation");
    let mut params = Params::new();

    if let Some(fields) = fields {
        params = params.add("fields", fields);
    }

    let body = CLIENT.get(&url).query(&params).process_as_json().await?;
    Ok(body)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore]
    async fn health_check() {
        let fields = "base_url|version|machine_version|usos_schema_version|institution_name|institution[id|name|profile_url|homepage_url|phone_numbers|phone_numbers2|postal_address|email|is_public|static_map_urls]|contact_emails|schac_id|mcards_support".to_string();
        let res = get_installation_info(Some(fields)).await.unwrap();
        println!("{res:?}");
    }
}
