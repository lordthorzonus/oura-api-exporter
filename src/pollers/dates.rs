use super::errors::OuraParsingError;
use chrono::{Date, DateTime, NaiveDate, NaiveDateTime, Utc};

const OURA_API_DATETIME_FORMAT: &'static str = "%Y-%m-%dT%H:%M:%S%.f%z";
const OURA_API_DATE_FORMAT: &'static str = "%Y-%m-%d";

pub trait TryOuraTimeStringParsing {
    fn try_parse_oura_timestamp(&self) -> Result<DateTime<Utc>, OuraParsingError>;
    fn try_parse_oura_date(&self) -> Result<NaiveDate, OuraParsingError>;
}

impl TryOuraTimeStringParsing for String {
    fn try_parse_oura_timestamp(&self) -> Result<DateTime<Utc>, OuraParsingError> {
        match NaiveDateTime::parse_from_str(self, OURA_API_DATETIME_FORMAT) {
            Ok(datetime) => Ok(datetime.and_utc()),
            Err(err) => Err(OuraParsingError {
                message: format!("Cannot parse Oura API timestamp: {}", err),
            }),
        }
    }

    fn try_parse_oura_date(&self) -> Result<NaiveDate, OuraParsingError> {
        NaiveDate::parse_from_str(self, OURA_API_DATE_FORMAT).map_err(|err| OuraParsingError {
            message: format!("Cannot parse Oura API date: {}", err),
        })
    }
}
