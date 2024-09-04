//! The USOS API time types.
//!
//! Note: USOS API provides time based on the Polish time zone, which is UTC+1 normally and UTC+2 in a daylight savings time
//! starting from the last Sunday in March and ending in the last Sunday in October. The types in this module **do not** include any time zone offsets.

use std::fmt::Display;

use serde::{Deserialize, Serialize};
use time::format_description::BorrowedFormatItem;
use time::macros::format_description;

pub const DATE_FORMAT: &[BorrowedFormatItem<'_>] = format_description!("[year]-[month]-[day]");
pub const TIME_FORMAT: &[BorrowedFormatItem<'_>] = format_description!("[hour]:[minute]:[second]");
pub const DATE_TIME_FORMAT: &[BorrowedFormatItem<'_>] =
    format_description!("[year]-[month]-[day] [hour]:[minute]:[second]");
pub const PRECISE_DATE_TIME_FORMAT: &[BorrowedFormatItem<'_>] =
    format_description!("[year]-[month]-[day] [hour]:[minute]:[second].[subsecond digits:6]");

/// Date as provided by the USOS API (yyyy-mm-dd).
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct UsosDate(#[serde(with = "crate::date_string")] pub time::Date);

impl From<UsosDate> for time::Date {
    fn from(date: UsosDate) -> Self {
        date.0
    }
}

impl From<time::Date> for UsosDate {
    fn from(date: time::Date) -> Self {
        UsosDate(date)
    }
}

impl Display for UsosDate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.format(DATE_FORMAT).unwrap())
    }
}

/// Time as provided by the USOS API (hh:mm:ss).
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct UsosTime(#[serde(with = "crate::time_string")] pub time::Time);

impl From<UsosTime> for time::Time {
    fn from(time: UsosTime) -> Self {
        time.0
    }
}

impl From<time::Time> for UsosTime {
    fn from(time: time::Time) -> Self {
        UsosTime(time)
    }
}

impl Display for UsosTime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.format(TIME_FORMAT).unwrap())
    }
}

/// Datetime as provided by the USOS API (yyyy-mm-dd hh:mm:ss).
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct UsosDateTime(#[serde(with = "crate::datetime_string")] pub time::PrimitiveDateTime);

impl From<UsosDateTime> for time::PrimitiveDateTime {
    fn from(datetime: UsosDateTime) -> Self {
        datetime.0
    }
}

impl From<time::PrimitiveDateTime> for UsosDateTime {
    fn from(datetime: time::PrimitiveDateTime) -> Self {
        UsosDateTime(datetime)
    }
}

impl Display for UsosDateTime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.format(DATE_TIME_FORMAT).unwrap())
    }
}

/// Datetime with the microsecond precision as provided by the USOS API (yyyy-mm-dd hh:mm:ss.mmmmmm)
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct UsosPreciseDateTime(
    #[serde(with = "crate::precise_datetime_string")] pub time::PrimitiveDateTime,
);

impl From<UsosPreciseDateTime> for time::PrimitiveDateTime {
    fn from(datetime: UsosPreciseDateTime) -> Self {
        datetime.0
    }
}

impl From<time::PrimitiveDateTime> for UsosPreciseDateTime {
    fn from(datetime: time::PrimitiveDateTime) -> Self {
        UsosPreciseDateTime(datetime)
    }
}

impl Display for UsosPreciseDateTime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.format(PRECISE_DATE_TIME_FORMAT).unwrap())
    }
}

#[cfg(test)]
mod tests {
    use crate::api::types::time::{UsosDate, UsosDateTime, UsosPreciseDateTime, UsosTime};
    use serde::Deserialize;
    use time::{Date, PrimitiveDateTime, Time};

    #[test]
    fn valid_date_string() {
        let (year, month, day) = (2024, 1, 1);
        let string = format!("{year:0>4}-{month:0>2}-{day:0>2}");
        let value = serde_json::to_value(string).unwrap();
        let date = UsosDate::deserialize(value).unwrap();
        assert_eq!(
            date.0,
            Date::from_calendar_date(year, time::Month::try_from(month).unwrap(), day).unwrap()
        );
    }

    #[test]
    fn valid_time_string() {
        let (hour, minute, second) = (12, 34, 56);
        let string = format!("{hour:0>2}:{minute:0>2}:{second:0>2}");
        let value = serde_json::to_value(string).unwrap();
        let time = UsosTime::deserialize(value).unwrap();
        assert_eq!(time.0, Time::from_hms(12, 34, 56).unwrap());
    }

    #[test]
    fn valid_datetime_string() {
        let (year, month, day) = (2024, 1, 1);
        let (hour, minute, second) = (12, 34, 56);
        let string =
            format!("{year:0>4}-{month:0>2}-{day:0>2} {hour:0>2}:{minute:0>2}:{second:0>2}");
        println!("{}", string);
        let value = serde_json::to_value(string).unwrap();
        let datetime = UsosDateTime::deserialize(value).unwrap();
        assert_eq!(
            datetime.0,
            PrimitiveDateTime::new(
                Date::from_calendar_date(year, time::Month::try_from(month).unwrap(), day).unwrap(),
                Time::from_hms(hour, minute, second).unwrap()
            )
        );
    }

    #[test]
    fn valid_precise_datetime_string() {
        let (year, month, day) = (2024, 1, 1);
        let (hour, minute, second, microsecond) = (12, 34, 56, 54321);
        let string = format!(
            "{year:0>4}-{month:0>2}-{day:0>2} {hour:0>2}:{minute:0>2}:{second:0>2}.{microsecond:0>6}"
        );
        println!("{}", string);
        let value = serde_json::to_value(string).unwrap();
        let datetime = UsosPreciseDateTime::deserialize(value).unwrap();
        assert_eq!(
            datetime.0,
            PrimitiveDateTime::new(
                Date::from_calendar_date(year, time::Month::try_from(month).unwrap(), day).unwrap(),
                Time::from_hms_micro(hour, minute, second, microsecond).unwrap()
            )
        );
    }
}
