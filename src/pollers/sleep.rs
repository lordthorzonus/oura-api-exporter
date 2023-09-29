use std::fmt::Display;
use crate::config::OuraPerson;
use crate::oura_api::{get_sleep_documents, OuraApiError, OuraSleepDocument};
use crate::pollers::dates::TryOuraTimeStringParsing;
use crate::pollers::errors::OuraParsingError;
use crate::pollers::OuraData;
use chrono::{DateTime, NaiveDate, Utc};

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

impl Display for SleepType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let sleep_type = match self {
            SleepType::Deleted => "deleted",
            SleepType::Sleep => "sleep",
            SleepType::LongSleep => "long_sleep",
            SleepType::LateNap => "late_nap",
            SleepType::Rest => "rest",
        };

        write!(f, "{}", sleep_type)
    }
}

#[derive(Debug)]
pub struct Sleep {
    pub id: String,
    pub average_breath: f32,
    pub average_hrv: i16,
    pub awake_time: i16,
    pub bedtime_end: DateTime<Utc>,
    pub bedtime_start: DateTime<Utc>,
    pub day: NaiveDate,
    pub deep_sleep_duration: i16,
    pub efficiency: i16,
    pub latency: i16,
    pub light_sleep_duration: i16,
    pub low_battery_alert: bool,
    pub lowest_heart_rate: i16,
    pub readiness_score_delta: Option<f32>,
    pub rem_sleep_duration: i16,
    pub restless_periods: i16,
    pub sleep_score_delta: Option<f32>,
    pub time_in_bed: i16,
    pub total_sleep_duration: i16,
    pub sleep_type: SleepType,
    pub person_name: String,
}

impl OuraSleepDocument {
    pub fn try_to_sleep_data(&self, person_name: &str) -> Result<Sleep, OuraParsingError> {
        Ok(Sleep {
            id: self.id.clone(),
            awake_time: self.awake_time,
            average_breath: self.average_breath,
            average_hrv: self.average_hrv,
            day: self.day.try_parse_oura_date()?,
            bedtime_start: self.bedtime_start.try_parse_oura_timestamp()?,
            bedtime_end: self.bedtime_end.try_parse_oura_timestamp()?,
            deep_sleep_duration: self.deep_sleep_duration,
            efficiency: self.efficiency,
            latency: self.latency,
            light_sleep_duration: self.light_sleep_duration,
            low_battery_alert: self.low_battery_alert,
            lowest_heart_rate: self.lowest_heart_rate,
            readiness_score_delta: self.readiness_score_delta,
            rem_sleep_duration: self.rem_sleep_duration,
            restless_periods: self.restless_periods,
            sleep_score_delta: self.sleep_score_delta,
            time_in_bed: self.time_in_bed,
            total_sleep_duration: self.total_sleep_duration,
            sleep_type: self.sleep_type.parse()?,
            person_name: person_name.to_owned(),
        })
    }
}

fn parse_sleep_data<'a>(
    person_name: &'a str,
    sleep_documents: &'a Vec<OuraSleepDocument>,
) -> impl Iterator<Item=OuraData> + 'a {
    return sleep_documents
        .iter()
        .map(|document| match document.try_to_sleep_data(person_name) {
            Ok(sleep) => OuraData::Sleep(sleep),
            Err(err) => OuraData::from_oura_parsing_error(err),
        });
}

fn parse_hrv_data<'a>(
    person_name: &'a str,
    sleep_documents: &'a Vec<OuraSleepDocument>,
) -> impl Iterator<Item=OuraData> + 'a {
    sleep_documents.iter().flat_map(|document| {
        let parsing_result = document.try_to_heart_rate_variability(person_name);
        let oura_data = match parsing_result {
            Ok(hrvs) => hrvs
                .into_iter()
                .map(|data| OuraData::HeartRateVariability(data))
                .collect(),
            Err(err) => vec![OuraData::from_oura_parsing_error(err)],
        };

        return oura_data;
    })
}

fn parse_heart_rate_data<'a>(
    person_name: &'a str,
    sleep_documents: &'a Vec<OuraSleepDocument>,
) -> impl Iterator<Item=OuraData> + 'a {
    sleep_documents.iter().flat_map(|document| {
        let parsing_result = document.try_to_heart_rate_data(person_name);
        let oura_data = match parsing_result {
            Ok(heart_rates) => heart_rates.into_iter().map(OuraData::HeartRate).collect(),
            Err(err) => vec![OuraData::from_oura_parsing_error(err)],
        };
        return oura_data;
    })
}

fn parse_sleep_phase_data<'a>(
    person_name: &'a str,
    sleep_documents: &'a Vec<OuraSleepDocument>,
) -> impl Iterator<Item=OuraData> + 'a {
    sleep_documents.iter().flat_map(|document| {
        return document.try_extract_sleep_phases(person_name).map_or_else(
            |err| vec![OuraData::from_oura_parsing_error(err)],
            |sleep_phases| sleep_phases.into_iter().map(OuraData::SleepPhase).collect(),
        );
    })
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
    let sleep_data = parse_sleep_data(person_name, &sleep_documents);
    let sleep_phase_data = parse_sleep_phase_data(person_name, &sleep_documents);

    let oura_data = heart_rate_data
        .chain(hrv_data)
        .chain(sleep_data)
        .chain(sleep_phase_data)
        .collect();
    return Ok(oura_data);
}
