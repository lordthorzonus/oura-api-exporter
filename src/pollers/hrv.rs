use crate::oura_api::OuraSleepDocument;
use crate::pollers;
use crate::pollers::dates::TryOuraTimeStringParsing;
use crate::pollers::errors::OuraParsingError;
use chrono::{DateTime, Utc};
use std::ops::Add;

#[derive(Debug)]
pub struct HeartRateVariability {
    pub ms: u16,
    pub timestamp: DateTime<Utc>,
    pub person_name: String,
}

impl OuraSleepDocument {
    pub fn try_to_heart_rate_variability(
        &self,
        person: &str,
    ) -> Result<Vec<HeartRateVariability>, OuraParsingError> {
        let hrv_measurement_interval_in_seconds = self.hrv.interval.round() as i64;

        let mut hrv_data: Vec<HeartRateVariability> = Vec::new();
        let mut timestamp = self.hrv.timestamp.try_parse_oura_timestamp()?;

        for item in &self.hrv.items {
            if let Some(hrv_item) = item {
                let hrv = hrv_item.round() as u16;

                hrv_data.push(HeartRateVariability {
                    ms: hrv,
                    timestamp,
                    person_name: person.to_owned(),
                });
            }

            timestamp = timestamp.add(chrono::Duration::seconds(
                hrv_measurement_interval_in_seconds,
            ));
        }

        Ok(hrv_data)
    }
}
