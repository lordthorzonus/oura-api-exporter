use serde::Deserialize;
use serde_yaml::from_reader;
use std::fs::File;
use thiserror::Error;

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

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Config FileError: '{0}'. Trying to open the file '{1}'")]
    FileError(#[source] std::io::Error, String),

    #[error("Config Serialization Error: {0}")]
    YamlError(#[from] serde_yaml::Error),
}

pub fn get_config() -> Result<Config, ConfigError> {
    let configuration_file_path = match std::env::var("CONFIGURATION_FILE_PATH") {
        Ok(val) => val,
        Err(_) => "configuration.yaml".to_string(),
    };
    let config_file = File::open(&configuration_file_path)
        .map_err(|err| ConfigError::FileError(err, configuration_file_path))?;

    return from_reader(config_file).map_err(ConfigError::YamlError);
}
