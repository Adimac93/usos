use std::{str::FromStr, time::Duration};

use client::BASE_URL;
use fantoccini::Locator;
use keys::ConsumerKey;
use secrecy::ExposeSecret;

pub mod api;
pub mod client;
pub mod keys;
pub mod webdriver;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    let consumer_key = ConsumerKey::from_env();
    consumer_key.save_to_file().await.unwrap();
}
