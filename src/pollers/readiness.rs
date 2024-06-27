use super::{dates::TryOuraTimeStringParsing, errors::OuraPollingError};
use crate::oura_api::OuraSleepDocument;
use chrono::{DateTime, Utc};

#[derive(Debug)]
pub struct Contributors {
    pub activity_balance: u8,
    pub body_temperature: u8,
    pub hrv_balance: u8,
    pub previous_day_activity: u8,
    pub previous_night: u8,
    pub recovery_index: u8,
    pub resting_heart_rate: u8,
    pub sleep_balance: u8,
}

#[derive(Debug)]
pub struct Readiness {
    pub score: u8,
    pub temperature_deviation: Option<f32>,
    pub temperature_trend_deviation: Option<f32>,
    pub contributors: Contributors,
    pub timestamp: DateTime<Utc>,
    pub person_name: String,
}

impl OuraSleepDocument {
    pub fn try_to_readiness(&self, person: &str) -> Result<Readiness, OuraPollingError> {
        let timestamp = self.day.try_parse_oura_date()?;

        match &self.readiness {
            None => {
                return Err(OuraPollingError::NoReadinessDataFoundError {
                    sleep_id: self.id.to_string(),
                });
            }
            Some(readiness) => {
                let score = match readiness.score {
                    Some(score) => score,
                    None => {
                        return Err(OuraPollingError::NoReadinessScoreFoundError {
                            sleep_id: self.id.to_string(),
                        })
                    }
                };

                let timestamp = timestamp
                    .and_hms_opt(0, 0, 0)
                    .ok_or(OuraPollingError::UnexpectedError(String::from(
                        "Cannot construct a NaiveDate from parsed oura date",
                    )))?
                    .and_utc();

                Ok(Readiness {
                    score,
                    temperature_deviation: readiness.temperature_deviation,
                    temperature_trend_deviation: readiness.temperature_trend_deviation,
                    contributors: Contributors {
                        activity_balance: readiness.contributors.activity_balance,
                        body_temperature: readiness.contributors.body_temperature,
                        hrv_balance: readiness.contributors.hrv_balance,
                        previous_day_activity: readiness.contributors.previous_day_activity,
                        previous_night: readiness.contributors.previous_night,
                        recovery_index: readiness.contributors.recovery_index,
                        resting_heart_rate: readiness.contributors.resting_heart_rate,
                        sleep_balance: readiness.contributors.sleep_balance,
                    },
                    timestamp,
                    person_name: person.to_string(),
                })
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::oura_api::{OuraContributors, OuraReadiness, OuraSleepDocument};
    use chrono::{DateTime, Utc};

    #[test]
    fn test_try_to_readiness() {
        let sleep_document = OuraSleepDocument {
            id: "test_id".to_owned(),
            day: "2021-01-01".to_owned(),
            readiness: Some(OuraReadiness {
                score: Some(80),
                temperature_deviation: Some(0.5),
                temperature_trend_deviation: Some(0.1),
                contributors: OuraContributors {
                    activity_balance: 1,
                    body_temperature: 2,
                    hrv_balance: 3,
                    previous_day_activity: 4,
                    previous_night: 5,
                    recovery_index: 6,
                    resting_heart_rate: 7,
                    sleep_balance: 8,
                },
            }),
            ..Default::default()
        };

        let readiness = sleep_document.try_to_readiness("test_person").unwrap();

        assert_eq!(readiness.score, 80);
        assert_eq!(readiness.temperature_deviation, Some(0.5));
        assert_eq!(readiness.temperature_trend_deviation, Some(0.1));

        assert_eq!(readiness.contributors.activity_balance, 1);
        assert_eq!(readiness.contributors.body_temperature, 2);
        assert_eq!(readiness.contributors.hrv_balance, 3);
        assert_eq!(readiness.contributors.previous_day_activity, 4);
        assert_eq!(readiness.contributors.previous_night, 5);
        assert_eq!(readiness.contributors.recovery_index, 6);
        assert_eq!(readiness.contributors.resting_heart_rate, 7);
        assert_eq!(readiness.contributors.sleep_balance, 8);

        assert_eq!(
            readiness.timestamp,
            "2021-01-01T00:00:00+00:00"
                .parse::<DateTime<Utc>>()
                .unwrap()
        );
        assert_eq!(readiness.person_name, "test_person");
    }

    #[test]
    fn test_try_to_readiness_with_empty_readiness() {
        let sleep_document = OuraSleepDocument {
            id: "test_id".to_owned(),
            day: "2021-01-01".to_owned(),
            readiness: None,
            ..Default::default()
        };

        let result = sleep_document.try_to_readiness("test_person");

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "No readiness data found for sleep document with id: 'test_id'"
        );
    }

    #[test]
    fn test_try_to_readiness_with_empty_score() {
        let sleep_document = OuraSleepDocument {
            id: "test_id".to_owned(),
            day: "2021-01-01".to_owned(),
            readiness: Some(OuraReadiness {
                score: None,
                temperature_deviation: Some(0.5),
                temperature_trend_deviation: Some(0.1),
                contributors: OuraContributors {
                    activity_balance: 1,
                    body_temperature: 2,
                    hrv_balance: 3,
                    previous_day_activity: 4,
                    previous_night: 5,
                    recovery_index: 6,
                    resting_heart_rate: 7,
                    sleep_balance: 8,
                },
            }),
            ..Default::default()
        };

        let result = sleep_document.try_to_readiness("test_person");

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "No readiness score found for sleep document with id: 'test_id'"
        );
    }
}
