use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct OuraSleepDocument {
    pub id: String,
    pub average_breath: f32,
    pub average_heart_rate: f32,
    pub average_hrv: i16,
    pub awake_time: i16,
    pub bedtime_end: String,
    pub bedtime_start: String,
    pub day: String,
    pub deep_sleep_duration: i16,
    pub efficiency: i16,
    pub heart_rate: OuraSleepHeartRate,
    //pub hrv: OuraSleepHeartRate,
    pub latency: i16,
    pub light_sleep_duration: i16,
    pub low_battery_alert: bool,
    pub lowest_heart_rate: i16,
    pub movement_30_sec: String,
    pub period: i16,
    pub readiness: Option<OuraReadiness>,
    pub readiness_score_delta: Option<f32>,
    pub rem_sleep_duration: i16,
    pub restless_periods: i16,
    pub sleep_phase_5_min: String,
    pub sleep_score_delta: Option<f32>,
    pub time_in_bed: i16,
    pub total_sleep_duration: i16,
    #[serde(rename = "type")]
    pub sleep_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OuraSleepHeartRate {
    pub interval: f32,
    pub items: Vec<Option<f32>>,
    pub timestamp: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OuraReadiness {
    contributors: OuraContributors,
    score: i16,
    temperature_deviation: Option<f32>,
    temperature_trend_deviation: f32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OuraContributors {
    activity_balance: i16,
    body_temperature: i16,
    hrv_balance: i16,
    previous_day_activity: i16,
    previous_night: i16,
    recovery_index: i16,
    resting_heart_rate: i16,
    sleep_balance: i16,
}
