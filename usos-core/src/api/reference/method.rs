use serde::Deserialize;

use crate::{
    api::scopes::Scope,
    client::{UsosUri, CLIENT},
};
/// apiref/method
///
/// Consumer: optional (required only for `admin_access`)
///
/// Token: ignored
///
/// Scopes: n/a
///
/// SSL: not required
pub async fn get_method_info(method_name: &str) -> MethodReference {
    let response = CLIENT
        .get(UsosUri::with_path("services/apiref/method"))
        .query(&[("name", method_name)])
        .send()
        .await
        .unwrap();

    response.json().await.unwrap()
}

#[tokio::test]
#[ignore]
async fn test_get_method_info() {
    let method = get_method_info("services/apiref/method").await;
    println!("{:?}", method);
}

/// # Consumer key signatures
/// - [`Required`] - method requires your application to identify itself
///
/// - [`Optional`] - you may identify yourself, in order to achieve some special behavior
///
/// - [`Ignored`] - method doesn't care if you sign your request or not
///
/// # Access token signatures
/// - [`Required`] - method requires you to include a Token. (Usually an Access Token, needed in order to identify the User in whose name you act upon.)
///
/// Note, that if you use an Administrative Consumer Key, then you may use a special **as_user_id** argument in your request. In this case, you should not include a Token in your signature.
///
/// - [`Optional`] - you may include a Token, to achieve some special behavior (i.e. some methods allow you to pass **user_id** - or include an Access Token - both in order to identify a user),
///
/// - [`Ignored`] - method doesn't care if you include a Token or not
#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
enum SignatureRequirement {
    Required,
    Optional,
    Ignored,
}

// name|short_name|description|brief_description|ref_url|auth_options|arguments|returns|errors|result_fields|beta|deprecated|admin_access|is_internal
#[derive(Debug, Deserialize)]
pub struct MethodReference {
    /// name of the method
    name: String,
    /// name without a path
    short_name: String,
    /// HTML-formatted description of what the method does
    description: String,
    /// brief (max 80 characters), single-line, plain-text description of what the method does
    brief_description: String,
    /// URL of a USOSap Reference webpage with method description
    ref_url: String,
    /// describes authentication requirements for this method
    auth_options: AuthRequirements,
    /// list of dictionaries describing method's parameters
    arguments: Vec<Argument>,
    /// HTML-formatted description method's return value
    returns: String,
    /// HTML-formatted description of possible method exceptions
    errors: String,
    ///  list of method's result fields. Any field can belong to either primary or secondary section. This list serves as a concrete specification and an alternative for the "returns" field in the method description
    result_fields: Vec<Field>,
    /// BETA methods may be altered in a backward-incompatible way
    beta: bool,
    /// in case of non-deprecated methods this will be null
    deprecated: Option<Deprecated>,
    /// true if you have administrative access to this method. You need to sign the request with your Consumer Key in order to access this field.
    /// **Consumer key required!!!
    admin_access: Option<bool>,
    /// true if this method is intended to be used only internally, by USOS API itself. This implies that it is in permanent BETA mode, and it can be altered or removed at any time.
    is_internal: bool,
}

// consumer|token|administrative_only|ssl_required|scopes
#[derive(Debug, Deserialize)]
struct AuthRequirements {
    consumer: SignatureRequirement,
    token: SignatureRequirement,
    administrative_only: bool,
    ssl_required: bool,
    scopes: Vec<Scope>,
}

// name|is_required|is_deprecated|default_value|description
#[derive(Debug, Deserialize)]
struct Argument {
    name: String,
    is_required: bool,
    is_deprecated: bool,
    /// [`None`] if parameter doesn't have a default value
    default_value: Option<String>,
    description: String,
}

// name|description|is_primary|is_secondary
#[derive(Debug, Deserialize)]
struct Field {
    name: Option<String>,
    description: String,
    is_primary: bool,
    is_secondary: bool,
}

// deprecated_by|present_until
#[derive(Debug, Deserialize)]
struct Deprecated {
    deprecated_by: Option<String>,
    present_until: Option<String>,
}
