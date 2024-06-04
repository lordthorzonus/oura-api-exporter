mod export_item;
mod influx_db_measurement;

use crate::pollers::OuraData;
use export_item::ExportItem;
use futures::{stream, Stream, StreamExt};
use influxdb2::api::write::TimestampPrecision;
use influxdb2::Client;
use itertools::{Either, Itertools};

use self::influx_db_measurement::InfluxDBMeasurement;

pub async fn export_oura_data(
    oura_data_stream: impl Stream<Item = OuraData>,
    influxdb_env: &Option<(Client, String)>,
) {
    oura_data_stream
        .flat_map(|oura_data| {
            let data: &OuraData = &oura_data;
            let export_items: Result<Vec<ExportItem>, _> = data.try_into();

            match export_items {
                Ok(export_items) => stream::iter(export_items),
                Err(err) => {
                    eprintln!("Error generating export items: {}", err);
                    stream::iter(vec![])
                }
            }
        })
        .chunks(10)
        .for_each_concurrent(None, |export_items| async move {
            let (influxdb_data_points, mqtt_export_items): (Vec<_>, Vec<_>) = export_items
                .into_iter()
                .partition_map(|export_item| match export_item {
                    ExportItem::MQTT(message) => Either::Right(message),
                    ExportItem::InfluxDB(data_point) => Either::Left(data_point),
                });

            if let Some((client, bucket)) = influxdb_env {
                write_to_influxdb(influxdb_data_points, client, bucket).await
            }

            println!("MQTT export item: {:?}", mqtt_export_items.last());
        })
        .await;
}

async fn write_to_influxdb(influxdb_data_points: Vec<InfluxDBMeasurement>, client: &Client, bucket: &String) {
    println!("Writing to InfluxDB: {:?}", influxdb_data_points);

    // let result = client
    //     .write_with_precision(
    //         bucket,
    //         stream::iter(influxdb_data_points),
    //         TimestampPrecision::Seconds,
    //     )
    //     .await;
    //
    // if let Err(err) = result {
    //     eprintln!("Error writing to InfluxDB: {}", err);
    // }
}
