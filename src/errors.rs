use std::fmt;

#[derive(Debug)]
pub struct NaluFxError {
    details: String,
}

impl NaluFxError {
    pub fn new(msg: &str) -> NaluFxError {
        NaluFxError {
            details: msg.to_string(),
        }
    }
}

impl fmt::Display for NaluFxError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.details)
    }
}

impl std::error::Error for NaluFxError {
    fn description(&self) -> &str {
        &self.details
    }
}
