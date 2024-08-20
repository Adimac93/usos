use std::fmt::Display;

use serde::{Deserialize, Serialize};
use time::format_description::BorrowedFormatItem;
use time::macros::format_description;

pub const DATE_FORMAT: &[BorrowedFormatItem<'_>] = format_description!("[year]-[month]-[day]");
pub const TIME_FORMAT: &[BorrowedFormatItem<'_>] = format_description!("[hour]:[minute]:[second]");
pub const DATE_TIME_FORMAT: &[BorrowedFormatItem<'_>] =
    format_description!("[year]-[month]-[day] [hour]:[minute]:[second]");

#[derive(Debug, Serialize, Deserialize)]
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

#[derive(Debug, Serialize, Deserialize)]
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

#[derive(Debug, Serialize, Deserialize)]
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

#[cfg(test)]
mod tests {
    use crate::{date_string, datetime_string, time_string};
    use serde::Deserialize;
    use time::{Date, PrimitiveDateTime, Time};

    #[derive(Deserialize)]
    struct DateString(#[serde(with = "date_string")] Date);

    #[derive(Deserialize)]
    struct TimeString(#[serde(with = "time_string")] Time);

    #[derive(Deserialize)]
    struct DateTimeString(#[serde(with = "datetime_string")] PrimitiveDateTime);

    #[test]
    fn valid_date_string() {
        let (year, month, day) = (2024, 1, 1);
        let string = format!("{year:0>4}-{month:0>2}-{day:0>2}");
        let value = serde_json::to_value(string).unwrap();
        let date = DateString::deserialize(value).unwrap();
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
        let time = TimeString::deserialize(value).unwrap();
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
        let datetime = DateTimeString::deserialize(value).unwrap();
        assert_eq!(
            datetime.0,
            PrimitiveDateTime::new(
                Date::from_calendar_date(year, time::Month::try_from(month).unwrap(), day).unwrap(),
                Time::from_hms(hour, minute, second).unwrap()
            )
        );
    }
}
