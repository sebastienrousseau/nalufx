use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct HistoricalData {
    pub ticker: String,
    pub data: Vec<f64>,
}
