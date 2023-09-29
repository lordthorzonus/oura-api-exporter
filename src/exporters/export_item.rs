use crate::exporters::influx_db_measurement::InfluxDBMeasurement;
use crate::pollers::{HeartRate, OuraData};
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

impl From<&OuraData> for Vec<ExportItem> {
    fn from(oura_data: &OuraData) -> Self {

        return match oura_data {
            OuraData::HeartRate(heart_rate_data) => {
                println!("DATA: {:?}", heart_rate_data);
                heart_rate_data.into()
            }
            OuraData::HeartRateVariability(hrv) => {
                vec![]
            }
            OuraData::Sleep(sleep) => {
                vec![]
            }
            OuraData::SleepPhase(sleep_phase) => {
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
}

impl From<&HeartRate> for Vec<ExportItem> {
    fn from(heart_rate_data: &HeartRate) -> Self {
        return vec![
            ExportItem::MQTT {
            topic: MqttTopic::HeartRate,
            payload: serde_json::to_string(heart_rate_data).unwrap(),
            },
            ExportItem::InfluxDB(heart_rate_data.into()),
        ];
    }
}

