use cli::GenerationOptions;
use errors::AppError;
use generation::generate;
use reqwest::Client;
use serde_json::Value;

pub mod cli;
pub mod errors;
pub mod generation;
pub mod module_system;
pub mod reference;

struct UsosUri;

impl UsosUri {
    const ORIGIN: &'static str = "https://apps.usos.pwr.edu.pl/";

    pub fn with_path(path: impl AsRef<str>) -> String {
        format!("{}{}", Self::ORIGIN, path.as_ref())
    }
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let client = Client::new();

    let options = GenerationOptions::prompt_cli(&client).await.unwrap();

    generate(&client, options).await.unwrap();
}
