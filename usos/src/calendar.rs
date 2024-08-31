use serde::Deserialize;
use time::PrimitiveDateTime;

use usos_core::api::types::language::LanguageDictionary;

#[derive(Debug, Deserialize)]
enum EventType {
    Rector,
    Dean,
    Holidays,
    PublicHolidays,
    ExamSession,
    Break,
    LinksEdit,
    Undefined,
}

#[derive(Debug, Deserialize)]
struct CalendarEvent {
    #[serde(rename = "id")]
    id: String,
    #[serde(rename = "name")]
    name: LanguageDictionary,
    // #[serde(rename = "start_date", with = "usos_core::datetime_string")] // FIXME
    start_date: PrimitiveDateTime,
    // #[serde(rename = "end_date", with = "usos_core::datetime_string")] // FIXME
    end_date: PrimitiveDateTime,
    // faculty: Faculty
    #[serde(rename = "type")]
    event_type: EventType,
}
