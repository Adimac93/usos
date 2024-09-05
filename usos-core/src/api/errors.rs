//! USOS API error handling utilities.

use std::{
    error::Error,
    fmt::{self, write, Display, Formatter},
};

use super::types::scopes::Scope;
use reason::Reason;
use serde::Deserialize;
use serde_json::json;
use user_message::UserMessages;
pub mod reason;
pub mod user_message;

/// A representation of standardized error objects sent by USOS API.
/// See [the USOS API reference](https://apps.usos.pw.edu.pl/developers/api/definitions/errors/) for more details.
#[derive(Debug, Deserialize)]
pub struct UsosError {
    /// Error description for the developer
    message: String,
    /// Error code
    #[serde(flatten)]
    kind: Option<UsosErrorKind>,
    /// Error description designed to be user-friendly
    user_messages: Option<UserMessages>,
    /// Required scopes that are missing
    missing_scopes: Option<Vec<Scope>>,
}

impl Error for UsosError {}

impl Display for UsosError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", &self.message)?;
        if let Some(kind) = &self.kind {
            write!(f, "\nKind: {kind}")?;
        }
        if let Some(scopes) = &self.missing_scopes {
            write!(
                f,
                "\nMissing scopes: {}",
                scopes
                    .iter()
                    .map(|scope| format!("'{scope}'"))
                    .collect::<Vec<_>>()
                    .join(", ")
            )?;
        }
        if let Some(user_messages) = &self.user_messages {
            write!(f, "\nUser messages: {}", user_messages)?;
        }
        Ok(())
    }
}

/// Possible error codes that USOS API can send.
#[derive(Debug, Deserialize)]
#[serde(tag = "error")]
#[serde(rename_all = "snake_case")]
pub enum UsosErrorKind {
    /// Access to the method is denied.
    MethodForbidden { reason: Reason },
    /// Required parameter is not provided.
    ParamMissing { param_name: String },
    /// The value of the parameter is invalid.
    ParamInvalid { param_name: String },
    /// You are not allowed to use the specified parameter.
    ParamForbidden { param_name: String, reason: Reason },
    /// Field specified in the `fields` parameter does not exist.
    FieldNotFound {
        field_name: String,
        method_name: String,
    },
    /// The specified field is invalid.
    ///
    /// This can happen for a variety of reasons:
    /// - you have provided subfields for field that does not refer to subobject(s),
    /// for example you gave the input `foo[bar]` and the returned object under the property `foo` does not contain the property `bar`
    /// - you have omitted subfields that were required;
    /// - you have used secondary field, but only primary were allowed.
    FieldInvalid {
        field_name: String,
        method_name: String,
    },
    /// You do not have access to some of the requested fields.
    FieldForbidden {
        field_name: String,
        method_name: String,
        reason: Reason,
    },
    /// Some of the referenced objects do not exist.
    ObjecetNotFound {
        param_name: String,
        method_name: String,
    },
    /// The referenced object is in a state that prevents method execution, see the appropriate USOS API method documentation for more detalis.
    ObjectInvalid,
    /// Access to the referenced object was denied.
    ObjectForbidden,
}

impl Display for UsosErrorKind {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            UsosErrorKind::MethodForbidden { reason } => {
                write!(f, "Method is forbidden - {reason}")
            }
            UsosErrorKind::ParamMissing { param_name } => {
                write!(f, "Parameter is missing: '{param_name}'")
            }
            UsosErrorKind::ParamInvalid { param_name } => {
                write!(f, "Parameter is invalid: '{param_name}'")
            }
            UsosErrorKind::ParamForbidden { param_name, reason } => {
                write!(
                    f,
                    "Parameter is forbidden: '{param_name}' Reason: {reason})"
                )
            }
            UsosErrorKind::FieldNotFound {
                field_name,
                method_name,
            } => write!(f, "Field not found: '{field_name}' Method: '{method_name}'"),
            UsosErrorKind::FieldInvalid {
                field_name,
                method_name,
            } => write!(
                f,
                "Field is invalid: '{field_name}' Method: '{method_name}'"
            ),
            UsosErrorKind::FieldForbidden {
                field_name,
                method_name,
                reason,
            } => write!(
                f,
                "Field is forbidden: '{field_name}' Method: '{method_name}' Reason: {reason})"
            ),
            UsosErrorKind::ObjecetNotFound {
                param_name,
                method_name,
            } => write!(
                f,
                "Object not found: '{param_name}' Method: '{method_name}'"
            ),
            UsosErrorKind::ObjectInvalid => write!(f, "Object is invalid"),
            UsosErrorKind::ObjectForbidden => write!(f, "Object is forbidden"),
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
    let error = UsosError::deserialize(&json).unwrap();
    println!("{error}");
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
    let error = UsosError::deserialize(&json).unwrap();
    println!("{error}");
}

#[test]
fn error_example_2_no_other_fields_on_kind() {
    let json = json!({
        "message": "Required parameter fac_id is missing.",
        "error": "object_forbidden",
        "user_messages": {
            "fields": {
                "fac_id": {
                    "en": "This field is required.",
                    "pl": "To pole jest wymagane."
                }
            }
        },
    });
    let error = UsosError::deserialize(&json).unwrap();
    println!("{error}");
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
    let error = UsosError::deserialize(&json).unwrap();
}
