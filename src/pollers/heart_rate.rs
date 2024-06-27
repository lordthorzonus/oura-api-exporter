use super::{OuraData, PollerPerson};
use crate::oura_api::OuraApiError;
use crate::oura_api::{OuraHeartRateData, OuraSleepDocument};
use crate::pollers::dates::TryOuraTimeStringParsing;
use crate::pollers::errors::OuraPollingError;
use chrono::{DateTime, Utc};
use log::info;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::ops::Add;
use std::str::FromStr;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum HeartRateSource {
    Awake,
    Rest,
    Sleep,
    Session,
    Live,
}

impl FromStr for HeartRateSource {
    type Err = OuraPollingError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "awake" => Ok(HeartRateSource::Awake),
            "rest" => Ok(HeartRateSource::Rest),
            "sleep" => Ok(HeartRateSource::Sleep),
            "session" => Ok(HeartRateSource::Session),
            "live" => Ok(HeartRateSource::Live),
            _ => Err(OuraPollingError::UnknownEnumVariantError {
                enum_name: "HeartRateSource".to_string(),
                variant: s.to_string(),
            }),
        }
    }
}

impl fmt::Display for HeartRateSource {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            HeartRateSource::Awake => write!(f, "awake"),
            HeartRateSource::Rest => write!(f, "rest"),
            HeartRateSource::Sleep => write!(f, "sleep"),
            HeartRateSource::Session => write!(f, "session"),
            HeartRateSource::Live => write!(f, "live"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct HeartRate {
    pub bpm: u8,
    pub source: HeartRateSource,
    pub timestamp: DateTime<Utc>,
    pub person_name: String,
}

impl OuraHeartRateData {
    fn try_to_heart_rate_data(&self, person: &str) -> Result<HeartRate, OuraPollingError> {
        let timestamp = self.timestamp.try_parse_oura_timestamp()?;

        Ok(HeartRate {
            bpm: self.bpm,
            person_name: person.to_owned(),
            source: self.source.parse()?,
            timestamp,
        })
    }
}

impl OuraSleepDocument {
    pub fn try_to_heart_rate_data(&self, person: &str) -> Result<Vec<HeartRate>, OuraPollingError> {
        match &self.heart_rate {
            Some(heart_rate) => {
                let heart_rate_measurement_interval_in_seconds =
                    heart_rate.interval.round() as i64;

                let mut heart_rate_data: Vec<HeartRate> = Vec::new();
                let mut timestamp = heart_rate.timestamp.try_parse_oura_timestamp()?;

                for heart_rate_item in &heart_rate.items {
                    if let Some(heart_rate) = heart_rate_item {
                        heart_rate_data.push(HeartRate {
                            bpm: heart_rate.round() as u8,
                            person_name: person.to_owned(),
                            source: HeartRateSource::Sleep,
                            timestamp,
                        });
                    }

                    timestamp = timestamp.add(chrono::Duration::seconds(
                        heart_rate_measurement_interval_in_seconds,
                    ));
                }

                Ok(heart_rate_data)
            }
            None => Err(OuraPollingError::NoHeartRateDataFoundError { sleep_id: self.id.to_string() })
        }
    }
}

pub async fn poll_heart_rate_data(
    person: &PollerPerson<'_>,
    start_time: &DateTime<Utc>,
    end_time: &DateTime<Utc>,
) -> Result<Vec<OuraData>, OuraApiError> {
    info!(
        "Polling heart rate data for '{}' from {} to {}",
        person.person.name, start_time, end_time
    );
    let response = person
        .client
        .get_heart_rate_data(start_time, end_time)
        .await;
    let heart_rate_data: Vec<OuraData> = response?
        .data
        .iter()
        .map(
            |raw| match raw.try_to_heart_rate_data(&person.person.name) {
                Ok(data) => OuraData::HeartRate(data),
                Err(parsing_error) => OuraData::from(parsing_error),
            },
        )
        .collect();

    return Ok(heart_rate_data);
}

#[cfg(test)]
mod test {
    use crate::{
        oura_api::{OuraSleepDocument, OuraSleepMeasurement},
        pollers::heart_rate::HeartRateSource,
    };

    #[test]
    fn test_heart_rate_source_from_str() {
        assert_eq!(
            "awake".parse::<HeartRateSource>().unwrap(),
            HeartRateSource::Awake
        );
        assert_eq!(
            "rest".parse::<HeartRateSource>().unwrap(),
            HeartRateSource::Rest
        );
        assert_eq!(
            "sleep".parse::<HeartRateSource>().unwrap(),
            HeartRateSource::Sleep
        );
        assert_eq!(
            "session".parse::<HeartRateSource>().unwrap(),
            HeartRateSource::Session
        );
        assert_eq!(
            "live".parse::<HeartRateSource>().unwrap(),
            HeartRateSource::Live
        );

        let error_source = "not-existing".parse::<HeartRateSource>().unwrap_err();
        assert_eq!(
            error_source.to_string(),
            "Unknown HeartRateSource: 'not-existing'"
        );
    }

    #[test]
    fn test_try_oura_heart_rate_data_to_heart_rate_data() {
        use crate::oura_api::OuraHeartRateData;
        use crate::pollers::heart_rate::{HeartRate, HeartRateSource};
        use chrono::{DateTime, Utc};

        let heart_rate_data = OuraHeartRateData {
            bpm: 60,
            source: "rest".to_owned(),
            timestamp: "2021-01-01T00:00:00Z".to_owned(),
        };

        let heart_rate = heart_rate_data.try_to_heart_rate_data("test").unwrap();
        assert_eq!(
            heart_rate,
            HeartRate {
                bpm: 60,
                source: HeartRateSource::Rest,
                timestamp: DateTime::parse_from_rfc3339("2021-01-01T00:00:00Z")
                    .unwrap()
                    .with_timezone(&Utc),
                person_name: "test".to_string(),
            }
        );
    }

    #[test]
    fn test_try_oura_sleep_document_to_heart_rate_data() {
        let oura_sleep_document = OuraSleepDocument {
            id: "id".to_owned(),
            heart_rate: Some(OuraSleepMeasurement {
                interval: 1.0,
                items: vec![Some(60.0), Some(61.0), Some(62.0)],
                timestamp: "2023-06-22T15:00:00+03:00".to_string(),
            }),
            ..Default::default()
        };

        let heart_rate_data = oura_sleep_document
            .try_to_heart_rate_data("person")
            .unwrap();
        assert_eq!(heart_rate_data.len(), 3);

        assert_eq!(heart_rate_data[0].bpm, 60);
        assert_eq!(heart_rate_data[0].person_name, "person");
        assert_eq!(heart_rate_data[0].source, HeartRateSource::Sleep);
        assert_eq!(
            heart_rate_data[0].timestamp,
            chrono::DateTime::parse_from_rfc3339("2023-06-22T15:00:00+03:00").unwrap()
        );

        assert_eq!(heart_rate_data[1].bpm, 61);
        assert_eq!(heart_rate_data[1].person_name, "person");
        assert_eq!(heart_rate_data[1].source, HeartRateSource::Sleep);
        assert_eq!(
            heart_rate_data[1].timestamp,
            chrono::DateTime::parse_from_rfc3339("2023-06-22T15:00:01+03:00").unwrap()
        );

        assert_eq!(heart_rate_data[2].bpm, 62);
        assert_eq!(heart_rate_data[2].person_name, "person");
        assert_eq!(heart_rate_data[2].source, HeartRateSource::Sleep);
        assert_eq!(
            heart_rate_data[2].timestamp,
            chrono::DateTime::parse_from_rfc3339("2023-06-22T15:00:02+03:00").unwrap()
        );
    }
}
