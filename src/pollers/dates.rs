use super::errors::OuraParsingError;
use chrono::{DateTime, NaiveDate, Utc};

const OURA_API_DATE_FORMAT: &'static str = "%Y-%m-%d";

pub trait TryOuraTimeStringParsing {
    fn try_parse_oura_timestamp(&self) -> Result<DateTime<Utc>, OuraParsingError>;
    fn try_parse_oura_date(&self) -> Result<NaiveDate, OuraParsingError>;
}

impl TryOuraTimeStringParsing for String {
    fn try_parse_oura_timestamp(&self) -> Result<DateTime<Utc>, OuraParsingError> {
        match DateTime::parse_from_rfc3339(self) {
            Ok(datetime) => Ok(datetime.with_timezone(&Utc)),
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
