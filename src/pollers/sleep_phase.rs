use crate::oura_api::OuraSleepDocument;
use crate::pollers::dates::TryOuraTimeStringParsing;
use crate::pollers::errors::OuraParsingError;
use chrono::{DateTime, Duration, Utc};
use std::ops::Add;

#[derive(Debug)]
pub struct SleepPhase {
    sleep_id: String,
    sleep_phase: SleepPhaseType,
    timestamp: DateTime<Utc>,
    person_name: String,
}

#[derive(Debug)]
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
