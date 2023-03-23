use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct OuraHeartRateData {
    pub bpm: u8,
    pub source: String,
    pub timestamp: String,
}
