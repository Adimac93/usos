#![cfg_attr(debug_assertions, allow(unused))]

pub mod api;
pub mod client;
pub mod errors;
pub mod keys;
pub mod webdriver;

// should stay in projecet root, see issue https://github.com/time-rs/time/issues/597
time::serde::format_description!(date_string, Date, api::types::time::DATE_FORMAT);
time::serde::format_description!(time_string, Time, api::types::time::TIME_FORMAT);
time::serde::format_description!(
    datetime_string,
    PrimitiveDateTime,
    api::types::time::DATE_TIME_FORMAT
);
time::serde::format_description!(
    precise_datetime_string,
    PrimitiveDateTime,
    api::types::time::PRECISE_DATE_TIME_FORMAT
);

pub type Result<T> = std::result::Result<T, errors::AppError>;
