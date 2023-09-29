use crate::pollers::HeartRate;
use crate::pollers::Sleep;
use influxdb2::models::{DataPoint, data_point::DataPointError};

#[derive(Debug)]
pub enum InfluxDBMeasurement {
    HeartRate {
        bpm: i64,
        source: String,
        timestamp: i64,
        person_name: String,
    },
    SleepPhase {
        phase: i64,
        timestamp: i64,
        person_name: String,
        sleep_id: String,
    },
    Sleep {
        id: String,
        average_breath: f64,
        average_hrv: i64,
        awake_time: i64,
        bedtime_end: i64,
        bedtime_start: i64,
        day: i64,
        deep_sleep_duration: i64,
        efficiency: i64,
        latency: i64,
        light_sleep_duration: i64,
        low_battery_alert: bool,
        lowest_heart_rate: i64,
        readiness_score_delta: f64,
        rem_sleep_duration: i64,
        restless_periods: i64,
        sleep_score_delta: f64,
        time_in_bed: i64,
        total_sleep_duration: i64,
        sleep_type: String,
        person_name: String,
    },
    HeartRateVariability {
        ms: i64,
        timestamp: i64,
        source: String,
        person_name: String,
    },
}

impl TryFrom<InfluxDBMeasurement> for DataPoint {
    type Error = DataPointError;

    fn try_from(measurement: InfluxDBMeasurement) -> Result<Self, Self::Error> {
        let data_point = match measurement {
            InfluxDBMeasurement::HeartRate {
                bpm,
                source,
                timestamp,
                person_name,
            } => DataPoint::builder("heart_rate")
                .field("bpm", bpm)
                .tag("source", source)
                .tag("person", person_name)
                .timestamp(timestamp)
                .build()?,
            InfluxDBMeasurement::SleepPhase {
                phase,
                timestamp,
                person_name,
                sleep_id,
            } => DataPoint::builder("sleep_phase")
                .field("phase", phase)
                .tag("person", person_name)
                .tag("sleep_id", sleep_id)
                .timestamp(timestamp)
                .build()?,
            InfluxDBMeasurement::Sleep {
                id,
                average_breath,
                average_hrv,
                awake_time,
                bedtime_end,
                bedtime_start,
                day,
                deep_sleep_duration,
                efficiency,
                latency,
                light_sleep_duration,
                low_battery_alert,
                lowest_heart_rate,
                readiness_score_delta,
                rem_sleep_duration,
                restless_periods,
                sleep_score_delta,
                time_in_bed,
                total_sleep_duration,
                sleep_type,
                person_name,
            } => {
                DataPoint::builder("sleep")
                    .field("id", id)
                    .field("average_breath", average_breath)
                    .field("average_hrv", average_hrv)
                    .field("awake_time", awake_time)
                    .field("bedtime_end", bedtime_end)
                    .field("bedtime_start", bedtime_start)
                    .field("day", day)
                    .field("deep_sleep_duration", deep_sleep_duration)
                    .field("efficiency", efficiency)
                    .field("latency", latency)
                    .field("light_sleep_duration", light_sleep_duration)
                    .field("low_battery_alert", low_battery_alert)
                    .field("lowest_heart_rate", lowest_heart_rate)
                    .field("rem_sleep_duration", rem_sleep_duration)
                    .field("restless_periods", restless_periods)
                    .field("time_in_bed", time_in_bed)
                    .field("total_sleep_duration", total_sleep_duration)
                    .field("readiness_score_delta", readiness_score_delta)
                    .field("sleep_score_delta", sleep_score_delta)
                    .tag("sleep_type", sleep_type)
                    .tag("person", person_name)
                    .timestamp(bedtime_start)
                    .build()?
            },
            InfluxDBMeasurement::HeartRateVariability {
                ms,
                timestamp,
                person_name,
                source,
            } => DataPoint::builder("heart_rate_variability")
                .field("ms", ms)
                .tag("person", person_name)
                .tag("source", source)
                .timestamp(timestamp)
                .build()?,
        };

        Ok(data_point)
    }
}

impl From<&HeartRate> for InfluxDBMeasurement {
    fn from(heart_rate_data: &HeartRate) -> Self {
        InfluxDBMeasurement::HeartRate {
            bpm: heart_rate_data.bpm.into(),
            source: heart_rate_data.source.to_string(),
            timestamp: heart_rate_data.timestamp.timestamp(),
            person_name: heart_rate_data.person_name.to_string(),
        }
    }
}

impl From<&Sleep> for InfluxDBMeasurement {
    fn from(value: &Sleep) -> Self {
        let readiness_score_delta = value.readiness_score_delta.unwrap_or(0.0);
        let sleep_score_delta = value.sleep_score_delta.unwrap_or(0.0);

        InfluxDBMeasurement::Sleep {
            id: value.id.to_string(),
            average_breath: value.average_breath.into(),
            average_hrv: value.average_hrv.into(),
            awake_time: value.awake_time.into(),
            bedtime_end: value.bedtime_end.timestamp(),
            bedtime_start: value.bedtime_start.timestamp(),
            day: value.day.and_hms(0, 0, 0).timestamp(),
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
        }
    }
}