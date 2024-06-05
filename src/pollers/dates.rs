use super::errors::OuraPollingError;
use chrono::{DateTime, NaiveDate, Utc};

const OURA_API_DATE_FORMAT: &'static str = "%Y-%m-%d";

pub trait TryOuraTimeStringParsing {
    fn try_parse_oura_timestamp(&self) -> Result<DateTime<Utc>, OuraPollingError>;
    fn try_parse_oura_date(&self) -> Result<NaiveDate, OuraPollingError>;
}

impl TryOuraTimeStringParsing for String {
    fn try_parse_oura_timestamp(&self) -> Result<DateTime<Utc>, OuraPollingError> {
        match DateTime::parse_from_rfc3339(self) {
            Ok(datetime) => Ok(datetime.with_timezone(&Utc)),
            Err(err) => Err(OuraPollingError::TimestampParsingError {
                timestamp: self.to_string(),
                source: err,
            }),
        }
    }

    fn try_parse_oura_date(&self) -> Result<NaiveDate, OuraPollingError> {
        NaiveDate::parse_from_str(self, OURA_API_DATE_FORMAT).map_err(|err| {
            OuraPollingError::DateParsingError {
                date: self.to_string(),
                source: err,
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;

    use super::*;

    #[test]
    fn test_try_parse_oura_timestamp() {
        let timestamp = "2021-01-01T00:00:00+00:00".to_string();
        let datetime = timestamp.try_parse_oura_timestamp().unwrap();
        assert_eq!(
            datetime,
            Utc.with_ymd_and_hms(2021, 01, 01, 0, 0, 0).unwrap()
        );
    }

    #[test]
    fn test_try_parse_oura_date() {
        let date = "2021-01-01".to_string();
        let naive_date = date.try_parse_oura_date().unwrap();
        assert_eq!(naive_date, NaiveDate::from_ymd_opt(2021, 1, 1).unwrap());
    }

    #[test]
    fn test_try_parse_oura_date_error() {
        let date = "2021-01-01T00:00:00+00:00".to_string();
        let error = date.try_parse_oura_date().unwrap_err();
        assert_eq!(
            error.to_string(),
            "Cannot parse Oura API date '2021-01-01T00:00:00+00:00': trailing input"
        );
    }

    #[test]
    fn test_try_parse_oura_timestamp_error() {
        let timestamp = "2021-01-01".to_string();
        let error = timestamp.try_parse_oura_timestamp().unwrap_err();
        assert_eq!(
            error.to_string(),
            "Cannot parse Oura API timestamp '2021-01-01': premature end of input"
        );
    }
}
