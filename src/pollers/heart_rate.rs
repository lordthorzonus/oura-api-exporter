use super::OuraData;
use crate::config::OuraPerson;
use crate::oura_api::OuraApiError;
use crate::oura_api::{get_heart_rate_data, OuraHeartRateData};
use chrono::{DateTime, NaiveDateTime, Utc};
use futures::{stream, TryFutureExt, TryStream, TryStreamExt};
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Serialize, Deserialize)]
pub enum HeartRateSource {
    Awake,
    Rest,
    Sleep,
    Session,
    Live,
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

impl HeartRateData {
    fn from_oura_data(oura_data: OuraHeartRateData, name: String) -> Self {
        HeartRateData {
            bpm: oura_data.bpm,
            person_name: name,
            source: match oura_data.source.as_str() {
                "awake" => HeartRateSource::Awake,
                "rest" => HeartRateSource::Rest,
                "sleep" => HeartRateSource::Sleep,
                "session" => HeartRateSource::Session,
                "live" => HeartRateSource::Live,
                _ => panic!("Unknown HeartRateSource"),
            },
            timestamp: DateTime::from_utc(
                NaiveDateTime::parse_from_str(
                    oura_data.timestamp.as_str(),
                    "%Y-%m-%dT%H:%M:%S%.f%z",
                )
                .unwrap(),
                Utc,
            ),
        }
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
                .map(|raw| {
                    OuraData::HeartRate(HeartRateData::from_oura_data(
                        raw,
                        String::from(&person.name),
                    ))
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
