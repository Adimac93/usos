use time::format_description::BorrowedFormatItem;
use time::macros::format_description;

pub const DATE_FORMAT: &[BorrowedFormatItem<'_>] = format_description!("[year]-[month]-[day]");
pub const TIME_FORMAT: &[BorrowedFormatItem<'_>] = format_description!("[hour]:[minute]:[second]");
pub const DATE_TIME_FORMAT: &[BorrowedFormatItem<'_>] =
    format_description!("[year]-[month]-[day] [hour]:[minute]:[second]");

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
