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

pub async fn get_heart_rate_data(
    access_token: &String,
    start_time: &DateTime<Utc>,
    end_time: &DateTime<Utc>,
) -> Result<OuraApiResponse<OuraHeartRateData>, OuraApiError> {
    let url = "https://api.ouraring.com/v2/usercollection/heartrate";
    let query = &[
        ("start_datetime", start_time.to_rfc3339()),
        ("end_datetime", end_time.to_rfc3339()),
    ];

    return oura_get_request(&access_token, url, query).await;
}

pub async fn get_sleep_documents(
    access_token: &String,
    start_time: &DateTime<Utc>,
    end_time: &DateTime<Utc>,
) -> Result<OuraApiResponse<OuraSleepDocument>, OuraApiError> {
    let url = "https://api.ouraring.com/v2/usercollection/sleep";
    let query = &[
        ("start_date", start_time.format("%Y-%m-%d").to_string()),
        ("end_date", end_time.format("%Y-%m-%d").to_string()),
    ];

    return oura_get_request(&access_token, url, query).await;
}

async fn oura_get_request<TEntity, TQuery>(
    access_token: &String,
    url: &str,
    query: &TQuery,
) -> Result<TEntity, OuraApiError>
where
    TEntity: serde::de::DeserializeOwned + std::fmt::Debug,
    TQuery: serde::Serialize + std::fmt::Debug,
{
    debug!("Sending request to Oura API: url={} query={:?}", url, query);

    let client = reqwest::Client::new();
    let response = client
        .get(url)
        .query(query)
        .header(AUTHORIZATION, format!("Bearer {}", access_token))
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
