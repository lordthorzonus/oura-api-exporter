use crate::config::OuraPerson;
use crate::oura_api::{get_sleep_documents, OuraApiError, OuraSleepDocument};
use crate::pollers;
use crate::pollers::errors::OuraParsingError;
use crate::pollers::hrv::ToMultipleHeartRateVariability;
use crate::pollers::OuraData;
use chrono::{DateTime, Utc};
use pollers::heart_rate::ToMultipleHeartRate;

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

pub async fn poll_sleep_data<'a>(
    person: &'a OuraPerson,
    start_time: &'a DateTime<Utc>,
    end_time: &'a DateTime<Utc>,
) -> Result<Vec<OuraData>, OuraApiError> {
    let person_name = person.name.as_str();
    let response = get_sleep_documents(&person.access_token, start_time, end_time).await;
    let sleep_documents = response?.data;

    let heart_rate_data = parse_heart_rate_data(person_name, &sleep_documents);
    let hrv_data = parse_hrv_data(person_name, &sleep_documents);

    return Ok(heart_rate_data.chain(hrv_data).collect());
}

fn parse_hrv_data<'a>(
    person_name: &'a str,
    sleep_documents: &'a Vec<OuraSleepDocument>,
) -> impl Iterator<Item = OuraData> + 'a {
    sleep_documents.iter().flat_map(|document| {
        let parsing_result = document.to_heart_rate_variability(person_name);
        let oura_data = match parsing_result {
            Ok(hrvs) => hrvs
                .into_iter()
                .map(|data| OuraData::HeartRateVariability(data))
                .collect(),
            Err(err) => vec![OuraData::Error {
                message: format!("{}", err),
            }],
        };

        return oura_data;
    })
}

fn parse_heart_rate_data<'a>(
    person_name: &'a str,
    sleep_documents: &'a Vec<OuraSleepDocument>,
) -> impl Iterator<Item = OuraData> + 'a {
    sleep_documents.iter().flat_map(|document| {
        let parsing_result = document.to_heart_rate_data(person_name);
        let oura_data = match parsing_result {
            Ok(heart_rates) => heart_rates
                .into_iter()
                .map(|data| OuraData::HeartRate(data))
                .collect(),
            Err(err) => vec![OuraData::Error {
                message: format!("{}", err),
            }],
        };
        return oura_data;
    })
}
