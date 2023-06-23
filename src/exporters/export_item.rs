use crate::exporters::influx_db_measurement::InfluxDBMeasurement;
use crate::pollers::{HeartRateData, OuraData};
use chrono::{DateTime, Utc};
use futures::{stream, Stream};
use influxdb2::models::{DataPoint, WriteDataPoint};
use std::fmt;

#[derive(Debug)]
pub enum MqttTopic {
    HeartRate,
    Sleep,
    Activity,
    Readiness,
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

pub enum ExportItem {
    MQTT { topic: MqttTopic, payload: String },
    InfluxDB(InfluxDBMeasurement),
}

pub fn map_oura_data_to_export_items(oura_data: OuraData) -> Vec<ExportItem> {
    return match oura_data {
        OuraData::HeartRate(heart_rate_data) => {
            map_heart_rate_data_to_export_items(&heart_rate_data)
        }
        OuraData::Sleep => {
            vec![]
        }
        OuraData::Activity => {
            vec![]
        }
        OuraData::Readiness => {
            vec![]
        }
        OuraData::Error { message } => {
            eprintln!("Cannot map OuraDataError into export item: {}", message);
            vec![]
        }
    };
}

fn map_heart_rate_data_to_export_items(heart_rate_data: &HeartRateData) -> Vec<ExportItem> {
    return vec![
        ExportItem::MQTT {
            topic: MqttTopic::HeartRate,
            payload: serde_json::to_string(heart_rate_data).unwrap(),
        },
        ExportItem::InfluxDB(InfluxDBMeasurement::from(heart_rate_data)),
    ];
}
