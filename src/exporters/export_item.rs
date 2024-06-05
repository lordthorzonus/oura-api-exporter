use crate::exporters::influx_db_measurement::{InfluxDBMeasurement, MeasurementConvertingError};
use crate::pollers::{HeartRate, HeartRateVariability, OuraData, Readiness, Sleep, SleepPhase};
use std::fmt;
use thiserror::Error;

#[derive(Debug)]
pub enum MqttTopic {
    HeartRate,
    Sleep,
    Activity,
    Readiness,
}

#[derive(Error, Debug)]
pub enum ExportItemGenerationError {
    #[error("Error while generating InfluxDB export item: {0}")]
    InfluxDBItemGenerationError(#[from] MeasurementConvertingError),

    #[error("Error while serializing MQTT message: {0}")]
    MQTTMessageSerializationError(#[from] serde_json::Error),

    #[error("Invalid Oura data: {0}")]
    InvalidOuraData(String),
}

fn try_into_influx_db_export_item<T>(data: T) -> Result<ExportItem, ExportItemGenerationError>
where
    T: TryInto<InfluxDBMeasurement, Error = MeasurementConvertingError>,
{
    let influx_db_measurement = data
        .try_into()
        .map_err(ExportItemGenerationError::InfluxDBItemGenerationError)?;

    return Ok(ExportItem::InfluxDB(influx_db_measurement));
}

impl TryFrom<&HeartRate> for Vec<ExportItem> {
    type Error = ExportItemGenerationError;

    fn try_from(heart_rate_data: &HeartRate) -> Result<Vec<ExportItem>, ExportItemGenerationError> {
        let mqtt_payload = serde_json::to_string(heart_rate_data)
            .map_err(ExportItemGenerationError::MQTTMessageSerializationError)?;

        return Ok(vec![
            ExportItem::MQTT(MqttMessage {
                topic: MqttTopic::HeartRate,
                payload: mqtt_payload,
            }),
            try_into_influx_db_export_item(heart_rate_data)?,
        ]);
    }
}

impl TryFrom<&HeartRateVariability> for Vec<ExportItem> {
    type Error = ExportItemGenerationError;

    fn try_from(
        hrv_data: &HeartRateVariability,
    ) -> Result<Vec<ExportItem>, ExportItemGenerationError> {
        return Ok(vec![try_into_influx_db_export_item(hrv_data)?]);
    }
}

impl TryFrom<&Sleep> for Vec<ExportItem> {
    type Error = ExportItemGenerationError;

    fn try_from(sleep_data: &Sleep) -> Result<Vec<ExportItem>, ExportItemGenerationError> {
        let export_item = try_into_influx_db_export_item(sleep_data)?;

        return Ok(vec![export_item]);
    }
}

impl TryFrom<&SleepPhase> for Vec<ExportItem> {
    type Error = ExportItemGenerationError;

    fn try_from(sleep_phase: &SleepPhase) -> Result<Vec<ExportItem>, ExportItemGenerationError> {
        return Ok(vec![try_into_influx_db_export_item(sleep_phase)?]);
    }
}

impl TryFrom<&Readiness> for Vec<ExportItem> {
    type Error = ExportItemGenerationError;

    fn try_from(readiness: &Readiness) -> Result<Vec<ExportItem>, ExportItemGenerationError> {
        return Ok(vec![try_into_influx_db_export_item(readiness)?]);
    }
}

impl fmt::Display for MqttTopic {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MqttTopic::HeartRate => write!(f, "oura/heart_rate"),
            MqttTopic::Sleep => write!(f, "oura/sleep"),
            MqttTopic::Activity => write!(f, "oura/activity"),
            MqttTopic::Readiness => write!(f, "oura/readiness"),
        }
    }
}

#[derive(Debug)]
pub struct MqttMessage {
    pub topic: MqttTopic,
    pub payload: String,
}

pub enum ExportItem {
    MQTT(MqttMessage),
    InfluxDB(InfluxDBMeasurement),
}

impl TryFrom<&OuraData> for Vec<ExportItem> {
    type Error = ExportItemGenerationError;

    fn try_from(oura_data: &OuraData) -> Result<Vec<ExportItem>, ExportItemGenerationError> {
        return match oura_data {
            OuraData::HeartRate(heart_rate_data) => Ok(heart_rate_data.try_into()?),
            OuraData::HeartRateVariability(hrv) => Ok(hrv.try_into()?),
            OuraData::Sleep(sleep) => Ok(sleep.try_into()?),
            OuraData::SleepPhase(sleep_phase) => Ok(sleep_phase.try_into()?),
            OuraData::Activity => Ok(vec![]),
            OuraData::Readiness(readiness) => Ok(readiness.try_into()?),
            OuraData::Error { message } => Err(ExportItemGenerationError::InvalidOuraData(
                message.to_string(),
            )),
        };
    }
}
