# Oura API exporter

This is a simple exporter for the [Oura API V2](https://cloud.ouraring.com/docs/). It exports the data to InfluxDB and MQTT.

Currently supported data:

- Heart rate
- Sleep
- HRV

## Example configuration.yaml

```yaml
persons:
  - name: John Doe
    access_token: oura-personal-access-token
poller_interval: 120 # seconds
influxdb:
  url: https://influxdb.lan.fi
  bucket: oura-data
  organization: homelab
  token: influxdb-access-token
```
