mod dates;
mod errors;
mod heart_rate;
mod hrv;
mod readiness;
mod sleep;
mod sleep_phase;

use crate::config::{OuraApi, OuraPerson};
use crate::oura_api::{OuraApiError, OuraHttpClient};
use crate::pollers::sleep::poll_sleep_data;
use chrono::{DateTime, Utc};
use futures::stream::{select, select_all};
use futures::{stream, FutureExt, Stream, StreamExt};
use heart_rate::poll_heart_rate_data;

pub use heart_rate::HeartRate;
pub use hrv::HeartRateVariability;
pub use readiness::Readiness;
pub use sleep::Sleep;
pub use sleep_phase::{SleepPhase, SleepPhaseType};

use self::errors::OuraPollingError;

#[derive(Debug)]
pub enum OuraData {
    HeartRate(HeartRate),
    HeartRateVariability(HeartRateVariability),
    Sleep(Sleep),
    SleepPhase(SleepPhase),
    Activity,
    Readiness(Readiness),
    Error { message: String },
}

impl OuraData {
    pub fn get_datetime(&self) -> Option<DateTime<Utc>> {
        match self {
            OuraData::HeartRate(heart_rate) => Some(heart_rate.timestamp),
            OuraData::HeartRateVariability(hrv) => Some(hrv.timestamp),
            OuraData::Sleep(sleep) => Some(sleep.bedtime_end),
            OuraData::SleepPhase(sleep_phase) => Some(sleep_phase.timestamp),
            OuraData::Readiness(readiness) => Some(readiness.timestamp),
            OuraData::Activity => None,
            OuraData::Error { .. } => None,
        }
    }
}

impl From<OuraPollingError> for OuraData {
    fn from(error: OuraPollingError) -> OuraData {
        return OuraData::Error {
            message: format!("{}", error),
        };
    }
}

impl From<OuraApiError> for OuraData {
    fn from(error: OuraApiError) -> OuraData {
        return OuraData::Error {
            message: format!("{}", error),
        };
    }
}

struct PollerPerson<'a> {
    person: &'a OuraPerson,
    client: OuraHttpClient<'a>,
}

pub struct Poller<'a> {
    persons: Vec<PollerPerson<'a>>,
}

impl Poller<'_> {
    pub fn initialize_with_persons<'a>(
        persons: &'a Vec<OuraPerson>,
        http_client_config: &'a Option<OuraApi>,
    ) -> Result<Poller<'a>, OuraApiError> {
        let poller_persons: Result<Vec<PollerPerson>, OuraApiError> = persons
            .iter()
            .map(|person| {
                Ok(PollerPerson {
                    person,
                    client: OuraHttpClient::from_config(http_client_config, person)?,
                })
            })
            .collect();

        Ok(Poller {
            persons: poller_persons?,
        })
    }

    pub fn poll_oura_data<'a>(
        &'a self,
        start_time: &'a DateTime<Utc>,
        end_time: &'a DateTime<Utc>,
    ) -> impl Stream<Item = OuraData> + 'a {
        let pollers = select_all(self.persons.iter().map(|person| {
            let sleep_data_stream =
                Box::pin(poll_sleep_data(person, start_time, end_time).into_stream());
            let heart_rate_data_stream =
                Box::pin(poll_heart_rate_data(person, start_time, end_time).into_stream());

            return select(sleep_data_stream, heart_rate_data_stream);
        }));

        return pollers.flat_map(|data| match data {
            Ok(data) => stream::iter(data),
            Err(err) => stream::iter(vec![OuraData::from(err)]),
        });
    }
}
