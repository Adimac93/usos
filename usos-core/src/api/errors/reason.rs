use serde::Deserialize;
use std::convert::Infallible;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Reason {
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

impl Display for Reason {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
			Reason::ConsumerMissing => write!(f, "consumer signature is missing"),
			Reason::UserMissing => write!(f, "user's access token is required"),
			Reason::SecureRequired => write!(f, "secure connection (SSL) is required"),
			Reason::TrustedRequired => write!(f, "only administrative consumers are allowed"),
			Reason::ScopeMissing => write!(f, "access token does not contain some of the required scopes"),
			Reason::ImpersonateRequired => write!(f, "`as_user_id` parameter was used with a method which you do not have administrative access to"),
			Reason::Custom(s) => write!(f, "{s}"),
		}
    }
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
