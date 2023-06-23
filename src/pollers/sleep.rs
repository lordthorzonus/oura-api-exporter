use crate::config::OuraPerson;
use crate::oura_api::{get_sleep_documents, OuraApiError};
use crate::pollers;
use crate::pollers::errors::OuraParsingError;
use crate::pollers::OuraData;
use chrono::{DateTime, Utc};
use futures::{stream, StreamExt, TryFutureExt, TryStream};
use itertools::Itertools;
use pollers::heart_rate::ToMultipleHeartRateData;

#[derive(Debug)]
pub enum SleepType {
    Deleted,
    Sleep,
    LongSleep,
    LateNap,
    Rest,
}

impl std::str::FromStr for SleepType {
    type Err = OuraParsingError;

    fn from_str(s: &str) -> Result<Self, OuraParsingError> {
        match s {
            "deleted" => Ok(SleepType::Deleted),
            "sleep" => Ok(SleepType::Sleep),
            "long_sleep" => Ok(SleepType::LongSleep),
            "late_nap" => Ok(SleepType::LateNap),
            "rest" => Ok(SleepType::Rest),
            _ => Err(OuraParsingError {
                message: format!("Unknown SleepType: {}", s),
            }),
        }
    }
}
pub struct Sleep {
    id: String,
    average_breath: f32,
    average_hrv: i16,
    awake_time: i16,
    bedtime_end: String,
    bedtime_start: String,
    day: String,
    deep_sleep_duration: i16,
    efficiency: i16,
    latency: i16,
    light_sleep_duration: i16,
    low_battery_alert: bool,
    lowest_heart_rate: i16,
    readiness_score_delta: Option<f32>,
    rem_sleep_duration: i16,
    restless_periods: i16,
    sleep_phase_5_min: String,
    sleep_score_delta: Option<f32>,
    time_in_bed: i16,
    total_sleep_duration: i16,
    sleep_type: SleepType,
    person_name: String,
}

pub fn poll_sleep_data<'a>(
    person: &'a OuraPerson,
    start_time: &'a DateTime<Utc>,
    end_time: &'a DateTime<Utc>,
) -> impl TryStream<Ok = Vec<OuraData>, Error = OuraApiError> + 'a {
    return stream::once(
        get_sleep_documents(&person.access_token, start_time, end_time).map_ok(move |response| {
            let sleep_documents = response.data;
            println!("RESPONSE, {:?}", sleep_documents);
            let heart_rate_data: Vec<OuraData> = sleep_documents
                .into_iter()
                .flat_map(|document| {
                    let parsing_result = document.to_heart_rate_data(person.name.clone());
                    let oura_data = match parsing_result {
                        Ok(heart_rates) => heart_rates
                            .into_iter()
                            .map(|data| OuraData::HeartRate(data))
                            .collect(),
                        Err(err) => vec![OuraData::Error {
                            message: format!("{}", err),
                        }],
                    };
                    println!("DATA: {:?}", oura_data);
                    return oura_data;
                })
                .collect();

            return heart_rate_data;
        }),
    );
}
