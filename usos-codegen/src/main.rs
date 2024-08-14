use errors::AppError;
use generation::generate_from_json_docs;
use reqwest::Client;
use serde_json::Value;

pub mod errors;
pub mod generation;

pub async fn get_docs(client: Client, path: impl AsRef<str>) -> Result<Value, AppError> {
    Ok(client
        .get(UsosUri::with_path("/services/apiref/method"))
        .query(&[("name", path.as_ref())])
        .send()
        .await?
        .json()
        .await?)
}

struct UsosUri;

impl UsosUri {
    const ORIGIN: &'static str = "https://apps.usos.pwr.edu.pl";

    pub fn with_path(path: impl AsRef<str>) -> String {
        format!("{}{}", Self::ORIGIN, path.as_ref())
    }
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let client = Client::new();

    let res = get_docs(client, "services/users/user").await.unwrap();

    generate_from_json_docs(res).await.unwrap();
}
