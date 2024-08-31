use serde::Deserialize;
use std::fmt::{self, Display, Formatter};
use usos_core::api::types::language::Language;
use usos_core::client::CLIENT;

/// fac/faculty
///
/// Consumer: optional
///
/// Token: optional
///
/// Scopes: n/a
///
/// SSL: not required
pub async fn search_faculties(
    language: Language,
    query: &str,
    visibility: Option<Visibility>,
    num: Option<SearchResults>,
    start: Option<StartIndex>,
) {
}

#[derive(Debug, Deserialize)]
pub struct FacultySearch {
    pub items: Vec<FacultySearchItem>,
    pub next_page: bool,
}

#[derive(Debug, Deserialize)]
pub struct FacultySearchItem {
    pub id: String,
    #[serde(rename = "match")]
    pub match_string: String,
}

pub struct SearchResults(u8);

impl TryFrom<u8> for SearchResults {
    type Error = String;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if value > 20 {
            Err(String::from("Number should be in range <1, 20>"))
        } else {
            Ok(SearchResults(value))
        }
    }
}

pub struct StartIndex(u16);

impl TryFrom<u16> for StartIndex {
    type Error = String;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        if value > 99 {
            Err(String::from("Number should be in range <0, 99>"))
        } else {
            Ok(StartIndex(value))
        }
    }
}

pub enum Visibility {
    Public,
    All,
}

impl Display for Visibility {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Visibility::Public => write!(f, "public"),
            Visibility::All => write!(f, "all"),
        }
    }
}

#[tokio::test]
#[ignore]
async fn test_get_faculty() {
    let faculty = search_faculties(Language::English, "Kwes", None, None, None).await;
    println!("{faculty:?}");
}
