use super::OuraData;
use crate::config::OuraPerson;
use crate::oura_api::OuraApiError;
use crate::oura_api::{get_heart_rate_data, OuraHeartRateData, OuraSleepDocument};
use crate::pollers::dates::TryOuraTimeStringParsing;
use crate::pollers::errors::OuraParsingError;
use chrono::{DateTime, Utc};
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

impl OuraHeartRateData {
    fn try_to_heart_rate_data(&self, person: &str) -> Result<HeartRate, OuraParsingError> {
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
    pub fn try_to_heart_rate_data(&self, person: &str) -> Result<Vec<HeartRate>, OuraParsingError> {
        let heart_rate_measurement_interval_in_seconds = self.heart_rate.interval.round() as i64;

        let mut heart_rate_data: Vec<HeartRate> = Vec::new();
        let mut timestamp = self.heart_rate.timestamp.try_parse_oura_timestamp()?;

        for heart_rate_item in &self.heart_rate.items {
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
        .map(|raw| match raw.try_to_heart_rate_data(&person.name) {
            Ok(data) => OuraData::HeartRate(data),
            Err(parsing_error) => OuraData::from_oura_parsing_error(parsing_error),
        })
        .collect();

    return Ok(heart_rate_data);
}
