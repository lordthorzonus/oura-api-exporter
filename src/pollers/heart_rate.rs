use super::OuraData;
use crate::config::OuraPerson;
use crate::oura_api::{get_heart_rate_data, OuraHeartRateData, OuraSleepDocument};
use crate::oura_api::{OuraApiError, OURA_API_DATETIME_FORMAT};
use crate::pollers::errors::OuraParsingError;
use chrono::{DateTime, NaiveDateTime, Utc};
use futures::{stream, TryFutureExt, TryStream, TryStreamExt};
use serde::de::IntoDeserializer;
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
pub struct HeartRateData {
    pub bpm: u8,
    pub source: HeartRateSource,
    pub timestamp: DateTime<Utc>,
    pub person_name: String,
}

pub trait ToSingleHeartRateData {
    fn to_heart_rate_data(&self, person: String) -> Result<HeartRateData, OuraParsingError>;
}

impl ToSingleHeartRateData for OuraHeartRateData {
    fn to_heart_rate_data(&self, person: String) -> Result<HeartRateData, OuraParsingError> {
        let timestamp = DateTime::from_utc(
            NaiveDateTime::parse_from_str(self.timestamp.as_str(), "%Y-%m-%dT%H:%M:%S%.f%z")
                .map_err(|err| OuraParsingError {
                    message: format!("Cannot parse HeartRateData timestamp: {}", err),
                })?,
            Utc,
        );

        Ok(HeartRateData {
            bpm: self.bpm,
            person_name: person,
            source: HeartRateSource::from_str(self.source.as_str())?,
            timestamp,
        })
    }
}

pub trait ToMultipleHeartRateData {
    fn to_heart_rate_data(&self, person: String) -> Result<Vec<HeartRateData>, OuraParsingError>;
}

impl ToMultipleHeartRateData for OuraSleepDocument {
    fn to_heart_rate_data(&self, person: String) -> Result<Vec<HeartRateData>, OuraParsingError> {
        let timestamp_start = DateTime::from_utc(
            NaiveDateTime::parse_from_str(
                self.heart_rate.timestamp.as_str(),
                OURA_API_DATETIME_FORMAT,
            )
            .map_err(|err| OuraParsingError {
                message: format!("Cannot parse HeartRateData timestamp: {}", err),
            })?,
            Utc,
        );
        let heart_rate_measurement_interval_in_seconds = self.heart_rate.interval;

        let (_, heart_rate_data) = self.heart_rate.items.iter().fold(
            (timestamp_start, vec![]),
            |(previous_timestamp, mut result_vec), item| {
                let timestamp = previous_timestamp.add(chrono::Duration::seconds(
                    heart_rate_measurement_interval_in_seconds.round() as i64,
                ));
                match item {
                    Some(bpm) => result_vec.push(HeartRateData {
                        bpm: bpm.clone().round() as u8,
                        person_name: person.clone(),
                        source: HeartRateSource::Sleep,
                        timestamp,
                    }),
                    None => (),
                }

                (timestamp, result_vec)
            },
        );

        Ok(heart_rate_data)
    }
}

pub fn poll_heart_rate_data<'a>(
    person: &'a OuraPerson,
    start_time: &'a DateTime<Utc>,
    end_time: &'a DateTime<Utc>,
) -> impl TryStream<Ok = Vec<OuraData>, Error = OuraApiError> + 'a {
    return stream::once(
        get_heart_rate_data(&person.access_token, start_time, end_time).map_ok(move |data| {
            let heart_rate_data: Vec<OuraData> = data
                .data
                .into_iter()
                .map(|raw| match raw.to_heart_rate_data(person.name.clone()) {
                    Ok(data) => OuraData::HeartRate(data),
                    Err(parsing_error) => OuraData::Error {
                        message: format!("{}", parsing_error),
                    },
                })
                .collect();
            println!("Polled {} heart rate data points", heart_rate_data.len(),);
            return heart_rate_data;
        }),
    )
    .inspect_err(|err| {
        eprintln!("Error polling heart rate data: {:?}", err);
    });
}
