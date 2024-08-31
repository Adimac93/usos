use serde::Deserialize;
use usos_core::client::CLIENT;

/// apiref/method_index
///
/// Consumer: ignored
///
/// Token: ignored
///
/// Scopes: n/a
///
/// SSL: not required
pub async fn get_method_index() {}

#[tokio::test]
#[ignore]
async fn test_get_method_index() {
    let methods = get_method_index().await;
    println!("{methods:#?}");
}

#[derive(Debug, Deserialize)]
pub struct MethodBrief {
    /// Method name without "services/" prefix
    pub name: String,
    pub brief_description: String,
}
