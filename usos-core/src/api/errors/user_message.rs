use crate::api::types::language::LanguageDictionary;
use serde::Deserialize;
use std::{
    collections::HashMap,
    fmt::{self, Display, Formatter},
};

/// Message intended for the user, possibly containing information about validation errors.
#[derive(Debug, Deserialize)]
pub struct UserMessages {
    /// [`LanguageDictionary`] object, with the generic (context-free) message to be displayed for the user.
    generic_message: Option<LanguageDictionary>,
    /// possibly empty - dictionary of parameters which have failed the validation, along with [`LanguageDictionary`] messages for each of them. Usually, the keys of this dictionary will match the named of the method parameters, but it is not a strict rule (e.g. some methods expect the forms to be submitted in a single parameter, as a JSON-encoded string).
    fields: Option<HashMap<String, LanguageDictionary>>,
}

impl Display for UserMessages {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if let Some(generic_message) = &self.generic_message {
            write!(f, "{generic_message}")?;
        }
        if let Some(fields) = &self.fields {
            write!(f, "Field errors:\n")?;
            let message = fields
                .iter()
                .map(|(field_name, field_message)| format!("\t'{field_name}': {field_message}"))
                .collect::<Vec<_>>()
                .join("\n");
            write!(f, "{message}")?;
        }
        Ok(())
    }
}
