mod heart_rate;
mod sleep;

use crate::config::{Config, OuraPerson};
use crate::oura_api::OuraApiError;
use chrono::{DateTime, Utc};
use futures::stream::select_all;
use futures::{stream, Stream, StreamExt, TryStreamExt};
use heart_rate::poll_heart_rate_data;
pub use heart_rate::HeartRateData;

#[derive(Debug)]
pub enum OuraData {
    HeartRate(HeartRateData),
    Sleep,
    Activity,
    Readiness,
    Error(OuraApiError),
}

pub struct Poller {
    persons: Vec<OuraPerson>,
}

impl Poller {
    pub fn initialize_with_persons(persons: Vec<OuraPerson>) -> Self {
        Poller { persons }
    }

    pub fn poll_oura_data<'a>(
        &'a self,
        start_time: &'a DateTime<Utc>,
        end_time: &'a DateTime<Utc>,
    ) -> impl Stream<Item = OuraData> + 'a {
        let pollers = select_all(self.persons.iter().flat_map(|person| {
            return vec![Box::pin(
                poll_heart_rate_data(person, start_time, end_time).into_stream(),
            )];
        }));

        return pollers.flat_map(|data| match data {
            Ok(data) => stream::iter(data),
            Err(err) => stream::iter(vec![OuraData::Error(err)]),
        });
    }
}
