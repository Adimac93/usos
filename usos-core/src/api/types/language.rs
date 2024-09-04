//! The language dictionary

use serde::Deserialize;
use std::{
    collections::HashMap,
    fmt::{self, Display, Formatter},
};

/// A type designed to contain human-readable responses in multiple languages.
///
/// See the "Language" section of [the USOS API reference](https://apps.usos.pw.edu.pl/developers/api/definitions/datatypes/).
#[derive(Debug, Deserialize)]
pub struct LanguageDictionary(HashMap<Language, String>);

impl Display for LanguageDictionary {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.english())
    }
}

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

/// All languages supported in user-friendly responses.
#[derive(Debug, Deserialize, Hash, Eq, PartialEq)]
pub enum Language {
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
