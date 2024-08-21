use std::{collections::HashMap, ops::Deref};

use serde::{Deserialize, Deserializer};
use serde_json::{json, Value};

use crate::{
    api::types::language::LanguageDictionary,
    client::{UsosDebug, CLIENT},
};

/// fac/faculty
///
/// Consumer: optional
///
/// Token: optional
///
/// Scopes: n/a
///
/// SSL: not required
pub async fn get_faculty(faculty_id: &str) -> Value {
    let response = CLIENT
        .get("https://apps.usos.pwr.edu.pl/services/fac/faculty")
        .query(&[("fac_id", faculty_id), ("fields", "id|name|profile_url|homepage_url|phone_numbers|phone_numbers2|postal_address|email|is_public|static_map_urls")])
        .send()
        .await
        .unwrap()
        .debug()
        .await
        .unwrap();

    let mut json = response.json::<Value>().await.unwrap();
    json
}

#[tokio::test]
#[ignore]
async fn test_get_faculty() {
    let faculty = get_faculty("K30W04ND03").await;
    println!("{faculty}");
}

#[derive(Debug, Deserialize)]
pub struct Faculty {
    id: String,
    name: LanguageDictionary,
    profile_url: String,
    homepage_url: Option<String>,
    phone_numbers: Vec<String>,
    phone_numbers2: Vec<PhoneNumber>,
    postal_address: String,
    email: Option<String>,
    is_public: bool,
    // stats: FacultyStats,
    // path: Vec<Faculty>,
    static_map_urls: StaticMapUrls,
}

#[derive(Debug, Deserialize)]
struct PhoneNumber {
    comment: Option<String>,
    number: String,
    #[serde(rename = "type")]
    phone_type: String,
}

#[derive(Debug, Deserialize)]
struct FacultyStats {
    course_count: Option<u32>,
    programme_count: Option<u32>,
    staff_count: Option<u32>,
    subfaculty_count: Option<u32>,
    public_subfaculty_count: Option<u32>,
}

/// Square: 100x100, 200x200, 300x300
///
///	Wide: 400x200, 600x300, 800x400
///
/// Landscape: 1000x250
#[derive(Debug, Hash, PartialEq, Eq)]
enum Resolution {
    /// [`Quality::Low`] 100px x 100px
    ///
    /// [`Quality::Medium`] 200px x 200px
    ///
    /// [`Quality::High`] 300px x 300px
    Square(Quality),
    /// [`Quality::Low`] 400px 200px
    ///
    /// [`Quality::Medium`] 600px x 300px
    ///
    /// [`Quality::High`] 800px x 400px
    Wide(Quality),
    /// 1000px x 250px
    Landscape,
}

#[derive(Debug, Deserialize, Hash, PartialEq, Eq)]
pub enum Quality {
    Low,
    Medium,
    High,
}

#[derive(Debug)]
struct StaticMapUrls(HashMap<Resolution, String>);

impl<'de> Deserialize<'de> for StaticMapUrls {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let map = HashMap::<String, String>::deserialize(deserializer)?;
        let mut out = HashMap::new();
        for (key, value) in map.iter() {
            let resolution = match key.as_str() {
                "100x100" => Resolution::Square(Quality::Low),
                "200x200" => Resolution::Square(Quality::Medium),
                "300x300" => Resolution::Square(Quality::High),
                "400x200" => Resolution::Wide(Quality::Low),
                "600x300" => Resolution::Wide(Quality::Medium),
                "800x400" => Resolution::Wide(Quality::High),
                "1000x250" => Resolution::Landscape,
                other => panic!("Invalid resolution {other}"),
            };
            out.insert(resolution, value.to_string());
        }
        Ok(StaticMapUrls(out))
    }
}
