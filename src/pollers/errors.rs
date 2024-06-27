use thiserror::Error;

#[derive(Debug, Error)]
pub enum OuraPollingError {
    #[error("Cannot parse Oura API timestamp '{timestamp}': {source}")]
    TimestampParsingError {
        timestamp: String,
        #[source]
        source: chrono::ParseError,
    },

    #[error("Cannot parse Oura API date '{date}': {source}")]
    DateParsingError {
        date: String,
        #[source]
        source: chrono::ParseError,
    },

    #[error("Unknown {enum_name}: '{variant}'")]
    UnknownEnumVariantError { enum_name: String, variant: String },

    #[error("No readiness data found for sleep document with id: '{sleep_id}'")]
    NoReadinessDataFoundError { sleep_id: String },

    #[error("No sleep data found for sleep document with id: '{sleep_id}'")]
    NoSleepDataFoundError { sleep_id: String },

    #[error("No sleep phase data found for sleep document with id: '{sleep_id}'")]
    NoSleepPhaseDataFoundError { sleep_id: String },

    #[error("No heart rate data found for sleep document with id: '{sleep_id}'")]
    NoHeartRateDataFoundError { sleep_id: String },

    #[error("No readiness score found for sleep document with id: '{sleep_id}'")]
    NoReadinessScoreFoundError { sleep_id: String },

    #[error("Something went wrong when polling Oura data: {0}")]
    UnexpectedError(String),
}
