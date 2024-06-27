mod heart_rate;
mod sleep;

use chrono::DateTime;
use chrono::Utc;
use log::debug;
use reqwest::header::AUTHORIZATION;
use reqwest::Response;
use reqwest::{Error as ReqwestError, StatusCode};
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub use heart_rate::OuraHeartRateData;
pub use sleep::OuraSleepDocument;

#[cfg(test)]
pub use sleep::{OuraContributors, OuraReadiness, OuraSleepMeasurement};

use crate::config::OuraApi;
use crate::config::OuraPerson;

#[derive(Serialize, Deserialize, Debug)]
pub struct OuraApiResponse<T> {
    pub data: Vec<T>,
    next_token: Option<String>,
}

#[derive(Debug, Error)]
pub enum OuraApiError {
    #[error("Failed to send request to Oura API: {0}")]
    RequestError(#[from] ReqwestError),

    #[error("Received error response from Oura API when requesting url: {url}. Error: \"{error}\", status: {status_code:?}")]
    ResponseError {
        status_code: Option<StatusCode>,
        error: String,
        url: String,
    },

    #[error("Invalid Oura HTTP client configuration: {0}")]
    InvalidHttpClientConfig(String),
}

pub struct OuraHttpClient<'a> {
    client: reqwest::Client,
    base_url: &'a str,
    port: &'a str,
    access_token: &'a str,
}

const DEFAULT_OURA_API_URL: &str = "https://api.ouraring.com";
const DEFAULT_OURA_API_PORT: &str = "443";

impl OuraHttpClient<'_> {
    pub fn from_config<'a>(
        config: &'a Option<OuraApi>,
        person: &'a OuraPerson,
    ) -> Result<OuraHttpClient<'a>, OuraApiError> {
        let mut client_builder = reqwest::Client::builder();

        match config {
            Some(c) => {
                let url = match &c.url {
                    Some(url) => url,
                    None => DEFAULT_OURA_API_URL,
                };

                let port = match &c.port {
                    Some(port) => port,
                    None => DEFAULT_OURA_API_PORT,
                };

                if let Some(proxy) = &c.proxy {
                    client_builder =
                        client_builder.proxy(reqwest::Proxy::all(proxy).map_err(|err| {
                            OuraApiError::InvalidHttpClientConfig(err.to_string())
                        })?);
                }

                if let Some(verbose_logging) = c.verbose_logging {
                    client_builder = client_builder.connection_verbose(verbose_logging)
                }

                let client = client_builder
                    .build()
                    .map_err(|err| OuraApiError::InvalidHttpClientConfig(err.to_string()))?;

                Ok(OuraHttpClient {
                    client,
                    base_url: url,
                    port,
                    access_token: &person.access_token,
                })
            }
            None => {
                let client = client_builder
                    .build()
                    .map_err(|err| OuraApiError::InvalidHttpClientConfig(err.to_string()))?;

                Ok(OuraHttpClient {
                    client,
                    base_url: DEFAULT_OURA_API_URL,
                    port: DEFAULT_OURA_API_PORT,
                    access_token: &person.access_token,
                })
            }
        }
    }

    fn build_url(&self, path: &str) -> String {
        format!("{}:{}/{}", self.base_url, self.port, path)
    }

    async fn get<TEntity, TQuery>(
        &self,
        path: &str,
        query: &TQuery,
    ) -> Result<TEntity, OuraApiError>
    where
        TEntity: serde::de::DeserializeOwned + std::fmt::Debug,
        TQuery: serde::Serialize + std::fmt::Debug,
    {
        let url = self.build_url(path);
        debug!("Sending request to Oura API: url={} query={:?}", url, query);

        let response = self
            .client
            .get(&url)
            .query(query)
            .header(AUTHORIZATION, format!("Bearer {}", self.access_token))
            .send()
            .await
            .map_err(OuraApiError::RequestError)?;

        let response_status = response.status();

        if !response_status.is_success() {
            return Err(map_response_into_response_error(response).await);
        }

        let response_url = response.url().to_string();
        let return_result =
            response
                .json::<TEntity>()
                .await
                .map_err(|e| OuraApiError::ResponseError {
                    status_code: e.status(),
                    error: format!("{}", e),
                    url: url.to_string(),
                })?;

        debug!(
            "Received response from Oura API: url={} status={:?} body={:#?}",
            response_url, response_status, return_result
        );

        return Ok(return_result);
    }

    pub async fn get_heart_rate_data(
        &self,
        start_time: &DateTime<Utc>,
        end_time: &DateTime<Utc>,
    ) -> Result<OuraApiResponse<OuraHeartRateData>, OuraApiError> {
        let path = "v2/usercollection/heartrate";
        let query = &[
            ("start_datetime", start_time.to_rfc3339()),
            ("end_datetime", end_time.to_rfc3339()),
        ];

        return self.get(path, query).await;
    }

    pub async fn get_sleep_documents(
        &self,
        start_time: &DateTime<Utc>,
        end_time: &DateTime<Utc>,
    ) -> Result<OuraApiResponse<OuraSleepDocument>, OuraApiError> {
        let path = "v2/usercollection/sleep";
        let query = &[
            ("start_date", start_time.format("%Y-%m-%d").to_string()),
            ("end_date", end_time.format("%Y-%m-%d").to_string()),
        ];

        return self.get(path, query).await;
    }
}

async fn map_response_into_response_error(response: Response) -> OuraApiError {
    let status_code = response.status();
    let url = response.url().to_string();
    let error = response
        .text()
        .await
        .unwrap_or(String::from("Unknown error response from Oura API"));

    return OuraApiError::ResponseError {
        status_code: Some(status_code),
        error: error.to_string(),
        url: url.to_string(),
    };
}
