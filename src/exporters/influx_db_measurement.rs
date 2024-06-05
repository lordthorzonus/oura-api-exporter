use crate::pollers::HeartRate;
use crate::pollers::HeartRateVariability;
use crate::pollers::Readiness;
use crate::pollers::Sleep;
use crate::pollers::SleepPhase;
use crate::pollers::SleepPhaseType;
use influxdb2::models::WriteDataPoint;
use influxdb2_derive::WriteDataPoint;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MeasurementConvertingError {
    #[error("Error while converting day into datetime for measurement: '{0}'")]
    DayToDateTimeConversionError(String),
}

#[derive(Debug, Default, WriteDataPoint)]
#[measurement = "heart_rate"]
pub struct HeartRateDataPoint {
    #[influxdb(field)]
    bpm: i64,

    #[influxdb(timestamp)]
    timestamp: i64,

    #[influxdb(tag)]
    source: String,

    #[influxdb(tag)]
    person_name: String,
}

#[derive(Debug, Default, WriteDataPoint)]
#[measurement = "sleep_phase"]
pub struct SleepPhaseDataPoint {
    #[influxdb(field)]
    phase: i64,

    #[influxdb(timestamp)]
    timestamp: i64,

    #[influxdb(tag)]
    person_name: String,

    #[influxdb(tag)]
    sleep_id: String,
}

#[derive(Debug, Default, WriteDataPoint)]
#[measurement = "sleep"]
pub struct SleepDataPoint {
    #[influxdb(tag)]
    id: String,

    #[influxdb(field)]
    average_breath: f64,

    #[influxdb(field)]
    average_hrv: i64,

    #[influxdb(field)]
    awake_time: i64,

    #[influxdb(field)]
    bedtime_end: i64,

    #[influxdb(timestamp)]
    bedtime_start: i64,

    #[influxdb(field)]
    day: i64,

    #[influxdb(field)]
    deep_sleep_duration: i64,

    #[influxdb(field)]
    efficiency: i64,

    #[influxdb(field)]
    latency: i64,

    #[influxdb(field)]
    light_sleep_duration: i64,

    #[influxdb(field)]
    low_battery_alert: bool,

    #[influxdb(field)]
    lowest_heart_rate: i64,

    #[influxdb(field)]
    readiness_score_delta: f64,

    #[influxdb(field)]
    rem_sleep_duration: i64,

    #[influxdb(field)]
    restless_periods: i64,

    #[influxdb(field)]
    sleep_score_delta: f64,

    #[influxdb(field)]
    time_in_bed: i64,

    #[influxdb(field)]
    total_sleep_duration: i64,

    #[influxdb(tag)]
    sleep_type: String,

    #[influxdb(tag)]
    person_name: String,
}

#[derive(Debug, Default, WriteDataPoint)]
#[measurement = "heart_rate_variability"]
pub struct HeartRateVariabilityDataPoint {
    #[influxdb(field)]
    ms: i64,

    #[influxdb(timestamp)]
    timestamp: i64,

    #[influxdb(tag)]
    person_name: String,
}

#[derive(Debug, Default, WriteDataPoint)]
pub struct ReadinessDataPoint {
    #[influxdb(field)]
    readiness_score: i64,

    #[influxdb(field)]
    temperature_deviation: Option<f64>,

    #[influxdb(field)]
    temperature_trend_deviation: f64,

    #[influxdb(field)]
    activity_balance_contribution: i64,

    #[influxdb(field)]
    body_temperature_contribution: i64,

    #[influxdb(field)]
    hrv_balance_contribution: i64,

    #[influxdb(field)]
    previous_day_activity_contribution: i64,

    #[influxdb(field)]
    previous_night_contribution: i64,

    #[influxdb(field)]
    recovery_index_contribution: i64,

    #[influxdb(field)]
    resting_heart_rate_contribution: i64,

    #[influxdb(field)]
    sleep_balance_contribution: i64,

    #[influxdb(timestamp)]
    timestamp: i64,

    #[influxdb(tag)]
    person_name: String,
}

#[derive(Debug)]
pub enum InfluxDBMeasurement {
    HeartRate(HeartRateDataPoint),
    SleepPhase(SleepPhaseDataPoint),
    Sleep(SleepDataPoint),
    HeartRateVariability(HeartRateVariabilityDataPoint),
    Readiness(ReadinessDataPoint),
}

impl WriteDataPoint for InfluxDBMeasurement {
    fn write_data_point_to<W>(&self, w: W) -> std::io::Result<()>
    where
        W: std::io::Write,
    {
        match self {
            InfluxDBMeasurement::HeartRate(data) => data.write_data_point_to(w),
            InfluxDBMeasurement::SleepPhase(data) => data.write_data_point_to(w),
            InfluxDBMeasurement::Sleep(data) => data.write_data_point_to(w),
            InfluxDBMeasurement::HeartRateVariability(data) => data.write_data_point_to(w),
            InfluxDBMeasurement::Readiness(data) => data.write_data_point_to(w),
        }
    }
}

impl TryFrom<&HeartRate> for InfluxDBMeasurement {
    type Error = MeasurementConvertingError;

    fn try_from(
        heart_rate_data: &HeartRate,
    ) -> Result<InfluxDBMeasurement, MeasurementConvertingError> {
        Ok(InfluxDBMeasurement::HeartRate(HeartRateDataPoint {
            bpm: heart_rate_data.bpm.into(),
            source: heart_rate_data.source.to_string(),
            timestamp: heart_rate_data.timestamp.timestamp(),
            person_name: heart_rate_data.person_name.to_string(),
        }))
    }
}

impl TryFrom<&Sleep> for InfluxDBMeasurement {
    type Error = MeasurementConvertingError;

    fn try_from(value: &Sleep) -> Result<InfluxDBMeasurement, MeasurementConvertingError> {
        let readiness_score_delta = value.readiness_score_delta.unwrap_or(0.0);
        let sleep_score_delta = value.sleep_score_delta.unwrap_or(0.0);

        let day = value.day.and_hms_opt(0, 0, 0).ok_or(
            MeasurementConvertingError::DayToDateTimeConversionError(String::from("Sleep")),
        );

        Ok(InfluxDBMeasurement::Sleep(SleepDataPoint {
            id: value.id.to_string(),
            average_breath: value.average_breath.into(),
            average_hrv: value.average_hrv.into(),
            awake_time: value.awake_time.into(),
            bedtime_end: value.bedtime_end.timestamp(),
            bedtime_start: value.bedtime_start.timestamp(),
            day: day?.timestamp(),
            deep_sleep_duration: value.deep_sleep_duration.into(),
            efficiency: value.efficiency.into(),
            latency: value.latency.into(),
            light_sleep_duration: value.light_sleep_duration.into(),
            low_battery_alert: value.low_battery_alert,
            lowest_heart_rate: value.lowest_heart_rate.into(),
            readiness_score_delta: readiness_score_delta.into(),
            rem_sleep_duration: value.rem_sleep_duration.into(),
            restless_periods: value.restless_periods.into(),
            sleep_score_delta: sleep_score_delta.into(),
            time_in_bed: value.time_in_bed.into(),
            total_sleep_duration: value.total_sleep_duration.into(),
            sleep_type: value.sleep_type.to_string(),
            person_name: value.person_name.to_string(),
        }))
    }
}

impl TryFrom<&HeartRateVariability> for InfluxDBMeasurement {
    type Error = MeasurementConvertingError;

    fn try_from(
        value: &HeartRateVariability,
    ) -> Result<InfluxDBMeasurement, MeasurementConvertingError> {
        Ok(InfluxDBMeasurement::HeartRateVariability(
            HeartRateVariabilityDataPoint {
                person_name: value.person_name.to_string(),
                ms: value.ms.into(),
                timestamp: value.timestamp.timestamp(),
            },
        ))
    }
}

impl From<&SleepPhaseType> for i64 {
    fn from(value: &SleepPhaseType) -> i64 {
        match value {
            SleepPhaseType::Awake => 4,
            SleepPhaseType::REMSleep => 3,
            SleepPhaseType::LightSleep => 2,
            SleepPhaseType::DeepSleep => 1,
        }
    }
}

impl TryFrom<&SleepPhase> for InfluxDBMeasurement {
    type Error = MeasurementConvertingError;

    fn try_from(value: &SleepPhase) -> Result<InfluxDBMeasurement, MeasurementConvertingError> {
        let sleep_phase = &value.sleep_phase;

        Ok(InfluxDBMeasurement::SleepPhase(SleepPhaseDataPoint {
            person_name: value.person_name.to_string(),
            phase: sleep_phase.into(),
            timestamp: value.timestamp.timestamp(),
            sleep_id: value.sleep_id.to_string(),
        }))
    }
}

impl TryFrom<&Readiness> for InfluxDBMeasurement {
    type Error = MeasurementConvertingError;

    fn try_from(value: &Readiness) -> Result<InfluxDBMeasurement, MeasurementConvertingError> {
        Ok(InfluxDBMeasurement::Readiness(ReadinessDataPoint {
            readiness_score: value.score.into(),
            temperature_deviation: value.temperature_deviation.map(|v| v.into()),
            temperature_trend_deviation: value.temperature_trend_deviation.into(),
            activity_balance_contribution: value.contributors.activity_balance.into(),
            body_temperature_contribution: value.contributors.body_temperature.into(),
            hrv_balance_contribution: value.contributors.hrv_balance.into(),
            previous_day_activity_contribution: value.contributors.previous_day_activity.into(),
            previous_night_contribution: value.contributors.previous_night.into(),
            recovery_index_contribution: value.contributors.recovery_index.into(),
            resting_heart_rate_contribution: value.contributors.resting_heart_rate.into(),
            sleep_balance_contribution: value.contributors.sleep_balance.into(),
            timestamp: value.timestamp.timestamp(),
            person_name: value.person_name.to_string(),
        }))
    }
}
