use std::{
    error::Error,
    fmt::{self, write, Display, Formatter},
};

use super::types::scopes::Scope;
use kind::ErrorKind;
use reason::Reason;
use serde::Deserialize;
use serde_json::json;
use user_message::UserMessages;
pub mod kind;
pub mod reason;
pub mod user_message;

#[derive(Debug)]
pub struct UsosError {
    /// Error description for the developer
    message: String,
    kind: Option<UsosErrorKind>,
    user_messages: Option<UserMessages>,
    missing_scopes: Option<Vec<Scope>>,
}

impl<'de> Deserialize<'de> for UsosError {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(Self::from(RawError::deserialize(deserializer)?))
    }
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

#[derive(Debug)]
pub enum UsosErrorKind {
    MethodForbidden {
        reason: Reason,
    },
    ParamMissing {
        param_name: String,
    },
    ParamInvalid {
        param_name: String,
    },
    ParamForbidden {
        param_name: String,
        reason: Reason,
    },
    FieldNotFound {
        field_name: String,
        method_name: String,
    },
    FieldInvalid {
        field_name: String,
        method_name: String,
    },
    FieldForbidden {
        field_name: String,
        method_name: String,
        reason: Reason,
    },
    ObjecetNotFound {
        param_name: String,
        method_name: String,
    },
    ObjectInvalid,
    ObjectForbidden,
}

impl From<RawError> for UsosError {
    fn from(error: RawError) -> Self {
        let kind = error.kind.map(|kind| match kind {
            ErrorKind::MethodForbidden => UsosErrorKind::MethodForbidden {
                reason: error.reason.unwrap(),
            },
            ErrorKind::ParamMissing => UsosErrorKind::ParamMissing {
                param_name: error.param_name.unwrap(),
            },
            ErrorKind::ParamInvalid => UsosErrorKind::ParamInvalid {
                param_name: error.param_name.unwrap(),
            },
            ErrorKind::ParamForbidden => UsosErrorKind::ParamForbidden {
                param_name: error.param_name.unwrap(),
                reason: error.reason.unwrap(),
            },
            ErrorKind::FieldNotFound => UsosErrorKind::FieldNotFound {
                field_name: error.field_name.unwrap(),
                method_name: error.method_name.unwrap(),
            },
            ErrorKind::FieldInvalid => UsosErrorKind::FieldInvalid {
                field_name: error.field_name.unwrap(),
                method_name: error.method_name.unwrap(),
            },
            ErrorKind::FieldForbidden => UsosErrorKind::FieldForbidden {
                field_name: error.field_name.unwrap(),
                method_name: error.method_name.unwrap(),
                reason: error.reason.unwrap(),
            },
            ErrorKind::ObjectNotFound => UsosErrorKind::ObjecetNotFound {
                param_name: error.param_name.unwrap(),
                method_name: error.method_name.unwrap(),
            },
            ErrorKind::ObjectInvalid => UsosErrorKind::ObjectInvalid,
            ErrorKind::ObjectForbidden => UsosErrorKind::ObjectForbidden,
        });
        UsosError {
            message: error.message,
            kind,
            user_messages: error.user_messages,
            missing_scopes: error.missing_scopes,
        }
    }
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

#[derive(Deserialize)]
struct RawError {
    message: String,
    #[serde(rename = "error")]
    kind: Option<ErrorKind>,
    reason: Option<Reason>,
    user_messages: Option<UserMessages>,
    param_name: Option<String>,
    field_name: Option<String>,
    method_name: Option<String>,
    missing_scopes: Option<Vec<Scope>>,
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
    println!("{error}")
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
    println!("{error}")
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
    println!("{error}")
}
