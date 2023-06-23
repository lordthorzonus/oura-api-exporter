use crate::pollers::HeartRateData;
use influxdb2::models::DataPoint;

#[derive(Debug)]
pub enum InfluxDBMeasurement {
    HeartRate {
        bpm: i64,
        source: String,
        timestamp: i64,
        person_name: String,
    },
}

impl InfluxDBMeasurement {
    pub fn into_data_point(self) -> DataPoint {
        return match self {
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
                .build()
                .unwrap(),
        };
    }
}

impl From<&HeartRateData> for InfluxDBMeasurement {
    fn from(heart_rate_data: &HeartRateData) -> Self {
        InfluxDBMeasurement::HeartRate {
            bpm: heart_rate_data.bpm.into(),
            source: heart_rate_data.source.to_string(),
            timestamp: heart_rate_data.timestamp.timestamp(),
            person_name: heart_rate_data.person_name.to_string(),
        }
    }
}
