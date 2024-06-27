use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct OuraSleepDocument {
    pub id: String,
    pub average_breath: Option<f32>,
    pub average_heart_rate: Option<f32>,
    pub average_hrv: Option<i16>,
    pub awake_time: i16,
    pub bedtime_end: String,
    pub bedtime_start: String,
    pub day: String,
    pub deep_sleep_duration: Option<i16>,
    pub efficiency: Option<i16>,
    pub heart_rate: Option<OuraSleepMeasurement>,
    pub hrv: OuraSleepMeasurement,
    pub latency: Option<i16>,
    pub light_sleep_duration: Option<i16>,
    pub low_battery_alert: bool,
    pub lowest_heart_rate: Option<i16>,
    pub movement_30_sec: String,
    pub period: i16,
    pub readiness: Option<OuraReadiness>,
    pub readiness_score_delta: Option<f32>,
    pub rem_sleep_duration: Option<i16>,
    pub restless_periods: Option<i16>,
    pub sleep_phase_5_min: Option<String>,
    pub sleep_score_delta: Option<f32>,
    pub time_in_bed: i16,
    pub total_sleep_duration: Option<i16>,
    #[serde(rename = "type")]
    pub sleep_type: String,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct OuraSleepMeasurement {
    pub interval: f32,
    pub items: Vec<Option<f32>>,
    pub timestamp: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OuraReadiness {
    pub contributors: OuraContributors,
    pub score: Option<u8>,
    pub temperature_deviation: Option<f32>,
    pub temperature_trend_deviation: Option<f32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OuraContributors {
    pub activity_balance: u8,
    pub body_temperature: u8,
    pub hrv_balance: u8,
    pub previous_day_activity: u8,
    pub previous_night: u8,
    pub recovery_index: u8,
    pub resting_heart_rate: u8,
    pub sleep_balance: u8,
}
