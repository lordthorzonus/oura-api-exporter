use chrono::{Duration, Utc};
use influxdb2::Client;
use std::sync::Arc;

mod config;
mod exporters;
mod oura_api;
mod pollers;

use crate::config::Config;
use config::InfluxDB;

#[tokio::main]
async fn main() {
    let Config {
        influxdb,
        poller_interval,
        persons,
    } = config::get_config();

    let poller = Arc::new(pollers::Poller::initialize_with_persons(persons));
    let influxdb_env = Arc::new(get_influxdb_env(influxdb));
    let seconds_in_past: i64 = poller_interval.into();
    let sleep_time: u64 = poller_interval.into();

    loop {
        let start_time = Utc::now() - Duration::seconds(seconds_in_past) - Duration::hours(4);
        let end_time = Utc::now();
        let arc_poller = poller.clone();
        let arc_influxdb_env = influxdb_env.clone();

        tokio::spawn(async move {
            exporters::export_oura_data(
                arc_poller.poll_oura_data(&start_time, &end_time),
                &arc_influxdb_env,
            )
            .await;
        })
        .await
        .unwrap();

        tokio::time::sleep(tokio::time::Duration::from_secs(sleep_time)).await;
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
