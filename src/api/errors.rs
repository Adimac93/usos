use std::{
    collections::HashMap,
    convert::Infallible,
    fmt,
    fmt::{Display, Formatter},
    str::FromStr,
};

use serde::Deserialize;
use serde_json::json;

#[derive(Deserialize)]
struct Error {
    /// Error description for the developer
    message: String,
    #[serde(rename = "error")]
    kind: Option<ErrorKind>,
    reason: Option<Reason>,
    user_messages: Option<UserMessages>,
    param_name: Option<String>,
    field_name: Option<String>,
    method_name: Option<String>,
    missing_scopes: Option<Vec<String>>,
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
enum ErrorKind {
    /// access to the method is denied; in this case **reason** key is always present
    MethodForbidden,
    /// required parameter is not provided; an additional key **param_name** will contain the name of the parameter that caused this error;
    ParamMissing,
    /// value of the parameter is invalid; this type of error also contains **param_name** key;
    ParamInvalid,
    /// you are not allowed to use the parameter; in such case **param_name** and **reason** will be present;
    ParamForbidden,
    /// field specified in **fields** parameter does not exist; the name of the field will be passed in **field_name** key together with the name of the method (**method_name** key);
    FieldNotFound,
    /// specified field is invalid:
    /// - you have provided subfields for field that does not refer to subobject(s);
    /// - you have omitted subfields that were required;
    /// - you have used secondary field, but only primary were allowed.
    /// **field_name** and **method_name** keys will contain the name of the field and its method that caused the error.
    FieldInvalid,
    /// you do not have access to some of the requested fields (**field_name**, **method_name** and **reason** keys will be present);
    FieldForbidden,
    /// some of the referenced objects do not exist; if the object was referenced by one of parameters, the **param_name** and **method_name** keys will be present;
    ObjectNotFound,
    /// the referenced object is in state that prevents method execution; the detailed description of such errors is available in method documentation.
    ObjectInvalid,
    /// access to the referenced object was denied.
    ObjectForbidden,
}

impl FromStr for ErrorKind {
    type Err = (());

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "method_forbidden" => Ok(Self::MethodForbidden),
            "param_missing" => Ok(Self::ParamMissing),
            "param_invalid" => Ok(Self::ParamInvalid),
            "param_forbidden" => Ok(Self::ParamForbidden),
            "field_not_found" => Ok(Self::FieldNotFound),
            "field_invalid" => Ok(Self::FieldInvalid),
            "field_forbidden" => Ok(Self::FieldForbidden),
            "object_not_found" => Ok(Self::ObjectNotFound),
            "object_invalid" => Ok(Self::ObjectInvalid),
            "object_forbidden" => Ok(Self::ObjectForbidden),
            _ => Err(()),
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
enum Reason {
    /// consumer signature is missing;
    ConsumerMissing,
    /// access token is required;
    UserMissing,
    /// secure connection (SSL) is required;
    SecureRequired,
    /// only administrative consumers are allowed;
    TrustedRequired,
    /// access token does not contain some of the required scopes; in this case an additional key **missing_scopes** will be present in the dictionary with the list of missing scopes;
    ScopeMissing,
    /// **as_user_id** parameter was used with a method which you do not have administrative access to;
    ImpersonateRequired,
    /// methods may define their own custom reason codes.
    Custom(String),
}

impl FromStr for Reason {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "consumer_missing" => Ok(Self::ConsumerMissing),
            "user_missing" => Ok(Self::UserMissing),
            "secure_required" => Ok(Self::SecureRequired),
            "trusted_required" => Ok(Self::TrustedRequired),
            "scope_missing" => Ok(Self::ScopeMissing),
            "impersonate_required" => Ok(Self::ImpersonateRequired),
            _ => Ok(Self::Custom(s.to_string())),
        }
    }
}

#[derive(Deserialize)]
struct UserMessages {
    /// [`LanguageDictionary`] object, with the generic (context-free) message to be displayed for the user.
    generic_message: Option<LanguageDictionary>,
    /// possibly empty - dictionary of parameters which have failed the validation, along with LangDict messages for each of them. Usually, the keys of this dictionary will match the named of the method parameters, but it is not a strict rule (e.g. some methods expect the forms to be submitted in a single parameter, as a JSON-encoded string).
    fields: Option<HashMap<String, LanguageDictionary>>,
}

#[derive(Deserialize)]
struct LanguageDictionary(HashMap<Language, String>);

impl LanguageDictionary {
    pub fn get(&self, language: Language) -> &str {
        self.0.get(&language).unwrap()
    }

    pub fn polish(&self) -> &str {
        self.get(Language::Polish)
    }

    pub fn english(&self) -> &str {
        self.get(Language::English)
    }
}

#[derive(Deserialize, Hash, Eq, PartialEq)]
enum Language {
    #[serde(rename = "pl")]
    Polish,
    #[serde(rename = "en")]
    English,
}

impl Display for Language {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Language::Polish => write!(f, "pl"),
            Language::English => write!(f, "en"),
        }
    }
}

#[test]
fn error_example_1() {
    let json = json!({
        "message": "Access denied - spam prevention lock.",
        "user_messages": {
            "generic_message": {
                "en": "You have sent over a 100 messages in the last hour. You must wait before you can send another one.",
                "pl": "W przeciągu ostatniej godziny wysłałeś ponad 100 wiadomości. Musisz poczekać, zanim pozwolimy Ci wysłać kolejną."
            },
        },
    });
    let error = Error::deserialize(&json).unwrap();
}

#[test]
fn error_example_2() {
    let json = json!({
        "message": "Required parameter fac_id is missing.",
        "error": "param_missing",
        "param_name": "fac_id",
        "user_messages": {
            "fields": {
                "fac_id": {
                    "en": "This field is required.",
                    "pl": "To pole jest wymagane."
                }
            }
        },
    });
    let error = Error::deserialize(&json).unwrap();
}

#[test]
fn error_example_3() {
    let json = json!({
        "message": "Multiple errors in the user-supplied form.",
        "user_messages": {
            "fields": {
                "fac_id": {
                    "en": "This field is required.",
                    "pl": "To pole jest wymagane."
                },
                "course_id": {
                    "en": "Course no longer conducted. Select another.",
                    "pl": "Ten przedmiot nie jest już prowadzony. Wybierz inny."
                }
            }
        },
    }
    );
    let error = Error::deserialize(&json).unwrap();
}
