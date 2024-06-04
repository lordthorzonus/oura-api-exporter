use crate::oura_api::OuraSleepDocument;
use crate::pollers::dates::TryOuraTimeStringParsing;
use crate::pollers::errors::OuraParsingError;
use chrono::{DateTime, Duration, Utc};
use std::ops::Add;

#[derive(Debug)]
pub struct SleepPhase {
    pub sleep_id: String,
    pub sleep_phase: SleepPhaseType,
    pub timestamp: DateTime<Utc>,
    pub person_name: String,
}

#[derive(Debug, PartialEq)]
pub enum SleepPhaseType {
    DeepSleep,
    LightSleep,
    REMSleep,
    Awake,
}

impl TryFrom<char> for SleepPhaseType {
    type Error = OuraParsingError;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            '1' => Ok(SleepPhaseType::DeepSleep),
            '2' => Ok(SleepPhaseType::LightSleep),
            '3' => Ok(SleepPhaseType::REMSleep),
            '4' => Ok(SleepPhaseType::Awake),
            _ => Err(OuraParsingError {
                message: format!("Unknown SleepStage: {}", value),
            }),
        }
    }
}

impl OuraSleepDocument {
    pub fn try_extract_sleep_phases(
        &self,
        person_name: &str,
    ) -> Result<Vec<SleepPhase>, OuraParsingError> {
        let mut timestamp = self.bedtime_start.try_parse_oura_timestamp()?;
        let mut sleep_phases: Vec<SleepPhase> = Vec::new();

        for sleep_phase_char in self.sleep_phase_5_min.chars() {
            sleep_phases.push(SleepPhase {
                sleep_phase: sleep_phase_char.try_into()?,
                sleep_id: self.id.clone(),
                timestamp,
                person_name: person_name.to_owned(),
            });

            timestamp = timestamp.add(Duration::minutes(5));
        }

        Ok(sleep_phases)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_try_from_char_for_sleep_phase_type() {
        assert_eq!(SleepPhaseType::DeepSleep, '1'.try_into().unwrap());
        assert_eq!(SleepPhaseType::LightSleep, '2'.try_into().unwrap());
        assert_eq!(SleepPhaseType::REMSleep, '3'.try_into().unwrap());
        assert_eq!(SleepPhaseType::Awake, '4'.try_into().unwrap());
    }

    #[test]
    fn test_try_extract_sleep_phases() {
        let oura_sleep_document = OuraSleepDocument {
            id: "id".to_owned(),
            bedtime_start: "2023-06-22T15:00:00+03:00".to_string(),
            sleep_phase_5_min: "1234".to_owned(),
            ..Default::default()
        };

        let sleep_phases = oura_sleep_document
            .try_extract_sleep_phases("person")
            .unwrap();

        assert_eq!(4, sleep_phases.len());

        assert_eq!(SleepPhaseType::DeepSleep, sleep_phases[0].sleep_phase);
        assert_eq!("person", sleep_phases[0].person_name);
        assert_eq!("id", sleep_phases[0].sleep_id);
        assert_eq!(DateTime::parse_from_rfc3339("2023-06-22T15:00:00+03:00").unwrap(), sleep_phases[0].timestamp);

        assert_eq!(SleepPhaseType::LightSleep, sleep_phases[1].sleep_phase);
        assert_eq!("person", sleep_phases[1].person_name);
        assert_eq!("id", sleep_phases[1].sleep_id);
        assert_eq!(DateTime::parse_from_rfc3339("2023-06-22T15:05:00+03:00").unwrap(), sleep_phases[1].timestamp);

        assert_eq!(SleepPhaseType::REMSleep, sleep_phases[2].sleep_phase);
        assert_eq!("person", sleep_phases[2].person_name);
        assert_eq!("id", sleep_phases[2].sleep_id);
        assert_eq!(DateTime::parse_from_rfc3339("2023-06-22T15:10:00+03:00").unwrap(), sleep_phases[2].timestamp);

        assert_eq!(SleepPhaseType::Awake, sleep_phases[3].sleep_phase);
        assert_eq!("person", sleep_phases[3].person_name);
        assert_eq!("id", sleep_phases[3].sleep_id);
        assert_eq!(DateTime::parse_from_rfc3339("2023-06-22T15:15:00+03:00").unwrap(), sleep_phases[3].timestamp);
    }
}
