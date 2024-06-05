use chrono::{Duration, Utc};
use futures::{stream, StreamExt};
use influxdb2::Client;
use log::{error, info};
use std::sync::Arc;

mod config;
mod exporters;
mod oura_api;
mod pollers;

use crate::config::Config;
use config::InfluxDB;

#[tokio::main]
async fn main() {
    env_logger::init();

    let config = match config::get_config() {
        Ok(config) => config,
        Err(e) => {
            error!("Error reading configuration: {}", e);
            return;
        }
    };

    let Config {
        influxdb,
        poller_interval,
        persons,
    } = config;
    let influxdb_env = Arc::new(get_influxdb_env(influxdb));
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();

    tokio::spawn(async move {
        let poller = pollers::Poller::initialize_with_persons(&persons);
        let seconds_in_past: i64 = poller_interval.into();
        let sleep_time: u64 = poller_interval.into();

        loop {
            let start_time = Utc::now() - Duration::seconds(seconds_in_past) - Duration::hours(64);
            let end_time = Utc::now();
            let _ = poller
                .poll_oura_data(&start_time, &end_time)
                .chunks(100)
                .for_each_concurrent(None, |data| async {
                    info!("Sending data for export. Got {} items", data.len());

                    match tx.send(data) {
                        Ok(_) => {}
                        Err(e) => {
                            error!("Error sending data to channel: {}", e);
                        }
                    }
                }).await;

            info!("Polling ended retrying in {} seconds", sleep_time);
            tokio::time::sleep(tokio::time::Duration::from_secs(sleep_time)).await
        }
    });

    while let Some(data) = rx.recv().await {
        exporters::export_oura_data(stream::iter(data), &influxdb_env).await
    }
}

fn get_influxdb_env<'a>(config: Option<InfluxDB>) -> Option<(Client, String)> {
    let influxdb_client = match config {
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
    influxdb_client
}
