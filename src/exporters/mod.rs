mod export_item;
mod influx_db_measurement;

use crate::pollers::OuraData;
use export_item::{ ExportItem};
use futures::{stream, Stream, StreamExt};
use influxdb2::api::write::TimestampPrecision;
use influxdb2::models::DataPoint;
use influxdb2::Client;
use itertools::{Either, Itertools};

pub async fn export_oura_data(
    oura_data_stream: impl Stream<Item = OuraData>,
    influxdb_env: &Option<(Client, String)>,
) {
    oura_data_stream
        .flat_map(|oura_data| {
            let data: &OuraData = &oura_data;
            let export_items: Vec<ExportItem> = data.into();

            return stream::iter(export_items);
        })
        .chunks(10)
        .for_each_concurrent(None, |export_items| async move {
            let (influxdb_data_points, mqtt_export_items): (Vec<_>, Vec<_>) = export_items
                .into_iter()
                .partition_map(|export_item| match export_item {
                    ExportItem::MQTT { topic, payload } => Either::Right((topic, payload)),
                    ExportItem::InfluxDB(data_point) => Either::Left(data_point.try_into()),
                });

            if let Some((client, bucket)) = influxdb_env {
                println!("Influx datapoints: {:?}", influxdb_data_points);
                let data_points = influxdb_data_points.filter_map(|data_point| data_point.ok());
                write_to_influxdb(data_points, client, bucket).await
            }

            println!("MQTT export item: {:?}", mqtt_export_items.last());
        })
        .await;
}

async fn write_to_influxdb(influxdb_data_points: Vec<DataPoint>, client: &Client, bucket: &String) {
    let result = client
        .write_with_precision(
            bucket,
            stream::iter(influxdb_data_points),
            TimestampPrecision::Seconds,
        )
        .await;

    if let Err(err) = result {
        eprintln!("Error writing to InfluxDB: {}", err);
    }
}
