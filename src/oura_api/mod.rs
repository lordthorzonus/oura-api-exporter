mod heart_rate;
mod sleep;

use crate::oura_api::sleep::OuraSleepDocument;
use chrono::DateTime;
use chrono::Utc;
pub use heart_rate::OuraHeartRateData;
use reqwest::header::AUTHORIZATION;
use reqwest::{Error as ReqwestError, StatusCode};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter, Result as FmtResult};

#[derive(Serialize, Deserialize, Debug)]
pub struct OuraApiResponse<T> {
    pub data: Vec<T>,
    next_token: Option<String>,
}

#[derive(Debug)]
pub enum OuraApiError {
    RequestError(ReqwestError),
    ResponseError {
        status_code: Option<StatusCode>,
        error: String,
        url: String,
    },
}

impl Display for OuraApiError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            OuraApiError::RequestError(e) => {
                write!(f, "Failed to send request to Oura API: {}", e)
            }
            OuraApiError::ResponseError {
                status_code,
                error,
                url,
            } => {
                write!(
                    f,
                    "Received error response from Oura API when requesting url: {}. Error {}, status: {:?}",
                    url, error, status_code
                )
            }
        }
    }
}

pub async fn get_heart_rate_data(
    access_token: &String,
    start_time: &DateTime<Utc>,
    end_time: &DateTime<Utc>,
) -> Result<OuraApiResponse<OuraHeartRateData>, OuraApiError> {
    println!(
        "Getting heart rate data from Oura API, start: {}, end: {}",
        start_time, end_time
    );
    let url = "https://api.ouraring.com/v2/usercollection/heartrate";
    let query = &[
        ("start_datetime", start_time.to_rfc3339()),
        ("end_datetime", end_time.to_rfc3339()),
    ];
    return oura_get_request(&access_token, url, query).await;
}

pub async fn get_sleep_documents(
    access_token: String,
    start_time: DateTime<Utc>,
    end_time: DateTime<Utc>,
) -> Result<OuraApiResponse<OuraSleepDocument>, OuraApiError> {
    let url = "https://api.ouraring.com/v2/usercciwollection/sleep";
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
    TEntity: serde::de::DeserializeOwned,
    TQuery: serde::Serialize,
{
    let client = reqwest::Client::new();
    let response = client
        .get(url)
        .query(query)
        .header(AUTHORIZATION, format!("Bearer {}", access_token))
        .send()
        .await
        .map_err(OuraApiError::RequestError)?;

    if !response.status().is_success() {
        return Err(OuraApiError::ResponseError {
            status_code: Some(response.status()),
            error: response.text().await.unwrap(),
            url: url.to_string(),
        });
    }

    let return_result =
        response
            .json::<TEntity>()
            .await
            .map_err(|e| OuraApiError::ResponseError {
                status_code: e.status(),
                error: format!("{}", e),
                url: url.to_string(),
            })?;

    return Ok(return_result);
}
