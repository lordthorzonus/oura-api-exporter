mod dates;
mod errors;
mod heart_rate;
mod hrv;
mod readiness;
mod sleep;
mod sleep_phase;

use crate::config::OuraPerson;
use crate::oura_api::OuraApiError;
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

pub struct Poller<'a> {
    persons: &'a Vec<OuraPerson>,
}

impl Poller<'_> {
    pub fn initialize_with_persons(persons: &Vec<OuraPerson>) -> Poller<'_> {
        Poller { persons }
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
