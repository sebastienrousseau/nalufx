use std::error::Error;
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

impl Error for NaluFxError {
    fn description(&self) -> &str {
        &self.details
    }
}

#[derive(Debug, PartialEq)]
pub enum AllocationError {
    InputMismatch,
    EmptyInput,
    ClusteringError(String),
    InvalidData,
    OutlierData,
}

impl fmt::Display for AllocationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AllocationError::InputMismatch => write!(f, "Input slices must have the same length"),
            AllocationError::EmptyInput => write!(f, "Input slices cannot be empty"),
            AllocationError::ClusteringError(msg) => write!(f, "Error during clustering: {}", msg),
            AllocationError::InvalidData => {
                write!(f, "Input data contains missing or invalid values")
            }
            AllocationError::OutlierData => write!(f, "Input data contains outliers"),
        }
    }
}

impl Error for AllocationError {}
