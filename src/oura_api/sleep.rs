use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct OuraSleepDocument {
    id: String,
    average_breath: f32,
    average_heart_rate: f32,
    average_hrv: i16,
    awake_time: i16,
    bedtime_end: String,
    bedtime_start: String,
    day: String,
    deep_sleep_duration: i16,
    efficiency: i16,
    heart_rate: OuraSleepHeartRate,
    hrv: OuraSleepHeartRate,
    latency: i16,
    light_sleep_duration: i16,
    low_battery_alert: bool,
    lowest_heart_rate: i16,
    movement_30_sec: String,
    period: i16,
    readiness: Option<OuraReadiness>,
    readiness_score_delta: Option<f32>,
    rem_sleep_duration: i16,
    restless_periods: i16,
    sleep_phase_5_min: String,
    sleep_score_delta: Option<f32>,
    time_in_bed: i16,
    total_sleep_duration: i16,
    #[serde(rename = "type")]
    sleep_type: String,
}

#[derive(Serialize, Deserialize)]
pub struct OuraSleepHeartRate {
    interval: i16,
    items: Vec<Option<i16>>,
    timestamp: String,
}

#[derive(Serialize, Deserialize)]
pub struct OuraReadiness {
    contributors: OuraContributors,
    score: i16,
    temperature_deviation: f32,
    temperature_trend_deviation: f32,
}

#[derive(Serialize, Deserialize)]
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
