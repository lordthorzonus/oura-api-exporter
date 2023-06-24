use super::OuraData;
use crate::config::OuraPerson;
use crate::oura_api::OuraApiError;
use crate::oura_api::{get_heart_rate_data, OuraHeartRateData, OuraSleepDocument};
use crate::pollers;
use crate::pollers::errors::OuraParsingError;
use chrono::{DateTime, Utc};
use futures::{stream, FutureExt, Stream, StreamExt, TryStream, TryStreamExt};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::ops::Add;
use std::str::FromStr;

#[derive(Debug, Serialize, Deserialize)]
pub enum HeartRateSource {
    Awake,
    Rest,
    Sleep,
    Session,
    Live,
}

impl FromStr for HeartRateSource {
    type Err = OuraParsingError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "awake" => Ok(HeartRateSource::Awake),
            "rest" => Ok(HeartRateSource::Rest),
            "sleep" => Ok(HeartRateSource::Sleep),
            "session" => Ok(HeartRateSource::Session),
            "live" => Ok(HeartRateSource::Live),
            _ => Err(OuraParsingError {
                message: format!("Unknown HeartRateSource: {}", s),
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

#[derive(Debug, Serialize, Deserialize)]
pub struct HeartRate {
    pub bpm: u8,
    pub source: HeartRateSource,
    pub timestamp: DateTime<Utc>,
    pub person_name: String,
}

pub trait ToSingleHeartRate {
    fn to_heart_rate_data(&self, person: &str) -> Result<HeartRate, OuraParsingError>;
}

impl ToSingleHeartRate for OuraHeartRateData {
    fn to_heart_rate_data(&self, person: &str) -> Result<HeartRate, OuraParsingError> {
        let timestamp = pollers::parse_oura_timestamp(self.timestamp.as_str())?;

        Ok(HeartRate {
            bpm: self.bpm,
            person_name: String::from(person),
            source: HeartRateSource::from_str(self.source.as_str())?,
            timestamp,
        })
    }
}

pub trait ToMultipleHeartRate {
    fn to_heart_rate_data(&self, person: &str) -> Result<Vec<HeartRate>, OuraParsingError>;
}

impl ToMultipleHeartRate for OuraSleepDocument {
    fn to_heart_rate_data(&self, person: &str) -> Result<Vec<HeartRate>, OuraParsingError> {
        let timestamp_start = pollers::parse_oura_timestamp(self.heart_rate.timestamp.as_str())?;
        let heart_rate_measurement_interval_in_seconds = self.heart_rate.interval.round() as i64;

        let heart_rate_data: Vec<HeartRate> = self
            .heart_rate
            .items
            .iter()
            .filter_map(|item| item.map(|heart_rate_item| heart_rate_item.round() as u8))
            .scan(timestamp_start, |previous_timestamp, heart_rate| {
                let timestamp = previous_timestamp.add(chrono::Duration::seconds(
                    heart_rate_measurement_interval_in_seconds,
                ));
                *previous_timestamp = timestamp;

                Some(HeartRate {
                    bpm: heart_rate,
                    person_name: String::from(person),
                    source: HeartRateSource::Sleep,
                    timestamp,
                })
            })
            .collect();

        Ok(heart_rate_data)
    }
}

pub async fn poll_heart_rate_data(
    person: &OuraPerson,
    start_time: &DateTime<Utc>,
    end_time: &DateTime<Utc>,
) -> Result<Vec<OuraData>, OuraApiError> {
    let response = get_heart_rate_data(&person.access_token, start_time, end_time).await;
    let heart_rate_data: Vec<OuraData> = response?
        .data
        .iter()
        .map(|raw| match raw.to_heart_rate_data(&person.name) {
            Ok(data) => OuraData::HeartRate(data),
            Err(parsing_error) => OuraData::Error {
                message: format!("{}", parsing_error),
            },
        })
        .collect();

    return Ok(heart_rate_data);
}
