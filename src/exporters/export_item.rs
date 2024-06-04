use crate::exporters::influx_db_measurement::{InfluxDBMeasurement, MeasurementConvertingError};
use crate::pollers::{HeartRate, OuraData};
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
            OuraData::HeartRateVariability(hrv) => Ok(vec![]),
            OuraData::Sleep(sleep) => Ok(vec![]),
            OuraData::SleepPhase(sleep_phase) => Ok(vec![]),
            OuraData::Activity => Ok(vec![]),
            OuraData::Readiness => Ok(vec![]),
            OuraData::Error { message } => Err(ExportItemGenerationError::InvalidOuraData(
                message.to_string(),
            )),
        };
    }
}

impl TryFrom<&HeartRate> for Vec<ExportItem> {
    type Error = ExportItemGenerationError;

    fn try_from(heart_rate_data: &HeartRate) -> Result<Vec<ExportItem>, ExportItemGenerationError> {
        let influx_db_item: InfluxDBMeasurement = heart_rate_data
            .try_into()
            .map_err(ExportItemGenerationError::InfluxDBItemGenerationError)?;

        let mqtt_payload = serde_json::to_string(heart_rate_data)
            .map_err(ExportItemGenerationError::MQTTMessageSerializationError)?;
        return Ok(vec![
            ExportItem::MQTT(MqttMessage {
                topic: MqttTopic::HeartRate,
                payload: mqtt_payload,
            }),
            ExportItem::InfluxDB(influx_db_item),
        ]);
    }
}
