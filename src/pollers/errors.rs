use std::error;
use std::fmt::Display;

#[derive(Debug)]
pub struct OuraParsingError {
    pub message: String,
}

impl Display for OuraParsingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Error while parsing Oura data from API: \"{}\"",
            self.message
        )
    }
}

impl error::Error for OuraParsingError {}

fn test() {}
