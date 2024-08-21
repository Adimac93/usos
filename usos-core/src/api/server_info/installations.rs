use serde::Deserialize;

use crate::{
    api::types::language::LanguageDictionary,
    client::{UsosUri, CLIENT},
    util::Process,
};

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
pub async fn get_installations() -> crate::Result<Vec<InstallationListItem>> {
    let url = UsosUri::with_path("services/apisrv/installations");

    let body = CLIENT.get(&url).process_as_json().await?;
    Ok(body)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore]
    async fn health_check() {
        let res = get_installations().await.unwrap();
        let res: Vec<InstallationListItem> =
            res.into_iter().filter(|x| x.version.is_none()).collect();
        println!("{res:?}");
    }
}
