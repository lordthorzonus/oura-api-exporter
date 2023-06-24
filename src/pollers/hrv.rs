use crate::oura_api::OuraSleepDocument;
use crate::pollers;
use crate::pollers::errors::OuraParsingError;
use chrono::{DateTime, Utc};
use std::ops::Add;

#[derive(Debug)]
pub struct HeartRateVariability {
    pub ms: u16,
    pub timestamp: DateTime<Utc>,
    pub person_name: String,
}

pub trait ToMultipleHeartRateVariability {
    fn to_heart_rate_variability(
        &self,
        person: &str,
    ) -> Result<Vec<HeartRateVariability>, OuraParsingError>;
}

impl ToMultipleHeartRateVariability for OuraSleepDocument {
    fn to_heart_rate_variability(
        &self,
        person: &str,
    ) -> Result<Vec<HeartRateVariability>, OuraParsingError> {
        let timestamp_start = pollers::parse_oura_timestamp(self.hrv.timestamp.as_str())?;
        let hrv_measurement_interval_in_seconds = self.hrv.interval.round() as i64;

        let hrv_data = self
            .hrv
            .items
            .iter()
            .filter_map(|item| item.map(|hrv_item| hrv_item.round() as u16))
            .scan(timestamp_start, |previous_timestamp, hrv| {
                let timestamp = previous_timestamp.add(chrono::Duration::seconds(
                    hrv_measurement_interval_in_seconds,
                ));
                *previous_timestamp = timestamp;

                Some(HeartRateVariability {
                    ms: hrv,
                    timestamp,
                    person_name: String::from(person),
                })
            })
            .collect();

        return Ok(hrv_data);
    }
}
