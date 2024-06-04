use crate::oura_api::OuraSleepDocument;
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

#[cfg(test)]
mod test {
    use crate::oura_api::{OuraSleepDocument, OuraSleepMeasurement};
    use chrono::{Utc, DateTime};

    #[test]
    fn test_try_to_heart_rate_variability() {
        let sleep_document = OuraSleepDocument {
            id: "test_id".to_owned(),
            hrv: OuraSleepMeasurement {
                interval: 60.0,
                timestamp: "2021-01-01T00:00:00+00:00".to_owned(),
                items: vec![Some(50.0), Some(60.0), None, Some(70.0)],
            },
            ..Default::default()
        };

        let hrv_data = sleep_document
            .try_to_heart_rate_variability("test_person")
            .unwrap();

        assert_eq!(hrv_data.len(), 3);
        assert_eq!(hrv_data[0].ms, 50);
        assert_eq!(hrv_data[1].ms, 60);
        assert_eq!(hrv_data[2].ms, 70);

        assert_eq!(hrv_data[0].person_name, "test_person");
        assert_eq!(hrv_data[1].person_name, "test_person");
        assert_eq!(hrv_data[2].person_name, "test_person");

        assert_eq!(
            hrv_data[0].timestamp,
            "2021-01-01T00:00:00+00:00".parse::<DateTime<Utc>>().unwrap()
        );
        assert_eq!(
            hrv_data[1].timestamp,
            "2021-01-01T00:01:00+00:00".parse::<DateTime<Utc>>().unwrap()
        );
        assert_eq!(
            hrv_data[2].timestamp,
            "2021-01-01T00:03:00+00:00".parse::<DateTime<Utc>>().unwrap()
        );
    }
}
