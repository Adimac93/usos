use std::{str::FromStr, time::Duration};

use errors::AppError;
use fantoccini::Locator;
use keys::ConsumerKey;
use secrecy::ExposeSecret;

pub mod api;
pub mod client;
pub mod errors;
pub mod keys;
pub mod util;
pub mod webdriver;

// should stay in projecet root, see issue https://github.com/time-rs/time/issues/597
time::serde::format_description!(date_string, Date, api::types::time::DATE_FORMAT);
time::serde::format_description!(time_string, Time, api::types::time::TIME_FORMAT);
time::serde::format_description!(
    datetime_string,
    PrimitiveDateTime,
    api::types::time::DATE_TIME_FORMAT
);

type Result<T> = std::result::Result<T, AppError>;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    let consumer_key = ConsumerKey::from_env().unwrap();
    consumer_key.save_to_file().await.unwrap();
}
