use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct CashFlowRequest {
    pub historical_data: Vec<f64>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct CashFlowResponse {
    pub predictions: Vec<f64>,
    pub optimal_allocation: Vec<f64>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ErrorResponse {
    pub error: String,
}
