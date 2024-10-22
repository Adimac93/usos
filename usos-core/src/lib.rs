//! A crate that provides the lower-level abstractions on USOS API.
//!
//! If you intend to use more sophisticated features, consider using the higher-level `usos` crate.
//!
//! # Feature flags
//! Following optional features are available:
//!
//! Name | Description | Default
//! --- | --- | ---
//! `keygen` | Enables consumer key generation API (see [`client`]) | No

#![cfg_attr(debug_assertions, allow(unused))]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]

pub mod api;
pub mod client;
pub mod errors;
pub mod keys;

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
