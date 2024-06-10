use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllocationOrder {
    pub symbol: String,
    pub name: String,
    pub amount: f64,
}

#[derive(Debug, Deserialize)]
pub struct Etf {
    pub symbol: String,
    pub name: String,
    pub price: f64,
    pub shares_outstanding: f64,
}

#[derive(Debug, Deserialize)]
pub struct MutualFund {
    pub symbol: String,
    pub name: String,
    pub price: f64,
    pub net_assets: f64,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AllocationRules {
    pub etf_percentage: f64,
    pub mutual_fund_percentage: f64,
}

pub trait FundData {
    fn symbol(&self) -> &str;
    fn name(&self) -> &str;
    fn value(&self) -> f64;
}

impl FundData for Etf {
    fn symbol(&self) -> &str {
        &self.symbol
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn value(&self) -> f64 {
        self.price * self.shares_outstanding
    }
}

impl FundData for MutualFund {
    fn symbol(&self) -> &str {
        &self.symbol
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn value(&self) -> f64 {
        self.price * self.net_assets
    }
}
