use chrono::{Duration, Utc};
use futures::{stream, StreamExt};
use influxdb2::Client;
use log::{error, info};

mod config;
mod exporters;
mod oura_api;
mod pollers;

use crate::config::Config;
use config::InfluxDB;

fn initialize_config_and_logging() -> Config {
    let mut logger_builder = env_logger::Builder::from_env("LOG_LEVEL");

    return match config::get_config() {
        Ok(config) => {
            match &config.log_level {
                Some(level) => {
                    logger_builder.filter_level(level.into());
                    ()
                }
                None => (),
            };

            logger_builder.init();
            config
        }
        Err(e) => {
            logger_builder.init();
            error!("Error reading configuration: {}", e);
            std::process::exit(exitcode::CONFIG);
        }
    };
}

async fn poll(
    poll_interval: u16,
    persons: &Vec<config::OuraPerson>,
    oura_api_config: &Option<config::OuraApi>,
    tx: tokio::sync::mpsc::UnboundedSender<Vec<pollers::OuraData>>,
) {
    match pollers::Poller::initialize_with_persons(persons, oura_api_config) {
        Ok(poller) => {
            let seconds_in_past: i64 = poll_interval.into();
            let sleep_time: u64 = poll_interval.into();
            let mut latest_timestamp =
                Utc::now() - Duration::seconds(seconds_in_past) - Duration::hours(48);

            loop {
                let start_time = latest_timestamp.clone();
                let end_time = Utc::now();
                let mut chunk_stream = poller.poll_oura_data(&start_time, &end_time).chunks(100);

                while let Some(chunk) = chunk_stream.next().await {
                    info!("Sending data for export. Got {} items", chunk.len());
                    let latest_timestamp_in_chunk =
                        chunk.iter().map(|item| item.get_datetime()).max();

                    if let Some(Some(timestamp)) = latest_timestamp_in_chunk {
                        if timestamp > latest_timestamp {
                            latest_timestamp = timestamp + Duration::seconds(1);
                        }
                    }

                    match tx.send(chunk) {
                        Ok(_) => {}
                        Err(e) => {
                            error!("Error sending data to channel: {}", e);
                        }
                    }
                }

                info!(
                    "Polling ended retrying in {} seconds with a new start_time {}",
                    sleep_time, latest_timestamp
                );

                tokio::time::sleep(tokio::time::Duration::from_secs(sleep_time)).await
            }
        }

        Err(e) => {
            error!("Error initializing poller: {}", e);
            std::process::exit(exitcode::CONFIG);
        }
    }
}

fn get_influxdb_env<'a>(config: Option<InfluxDB>) -> Option<(Client, String)> {
    return match config {
        Some(influxdb_config) => {
            let client = Client::new(
                &influxdb_config.url,
                &influxdb_config.organization,
                &influxdb_config.token,
            );
            Some((client, influxdb_config.bucket))
        }
        None => None,
    };
}

#[tokio::main]
async fn main() {
    let config = initialize_config_and_logging();
    let Config {
        influxdb,
        poller_interval,
        persons,
        log_level: _,
        oura_api,
    } = config;
    let influxdb_env = get_influxdb_env(influxdb);
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();

    tokio::spawn(async move {
        poll(poller_interval, &persons, &oura_api, tx).await;
    });

    while let Some(data) = rx.recv().await {
        exporters::export_oura_data(stream::iter(data), &influxdb_env).await
    }
}

