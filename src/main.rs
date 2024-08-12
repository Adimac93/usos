use std::{str::FromStr, time::Duration};

use errors::AppError;
use fantoccini::Locator;
use keys::ConsumerKey;
use secrecy::ExposeSecret;

pub mod api;
pub mod client;
pub mod errors;
pub mod keys;
pub mod webdriver;

type Result<T> = std::result::Result<T, AppError>;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    let consumer_key = ConsumerKey::from_env().unwrap();
    consumer_key.save_to_file().await.unwrap();
}
