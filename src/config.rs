use serde::Deserialize;
use serde_yaml::from_reader;
use std::fs::File;

#[derive(Deserialize, Clone, Debug)]
pub struct OuraPerson {
    pub name: String,
    pub access_token: String,
}

#[derive(Deserialize, Debug)]
pub struct InfluxDB {
    pub url: String,
    pub token: String,
    pub organization: String,
    pub bucket: String,
}

#[derive(Deserialize, Debug)]
pub struct Config {
    pub persons: Vec<OuraPerson>,
    pub poller_interval: u16,
    pub influxdb: Option<InfluxDB>,
}

pub fn get_config() -> Config {
    let configuration_file_path = match std::env::var("CONFIGURATION_FILE_PATH") {
        Ok(val) => val,
        Err(_) => "configuration.yaml".to_string(),
    };
    let config_file = File::open(configuration_file_path).unwrap();

    return from_reader(config_file).unwrap();
}
