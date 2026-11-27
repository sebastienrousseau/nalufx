use serde::{Deserialize, Serialize};

/// Represents an order to allocate a certain amount of funds to a particular symbol.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllocationOrder {
    /// The symbol of the asset (e.g., stock ticker).
    pub symbol: String,
    /// The name of the asset.
    pub name: String,
    /// The amount to be allocated.
    pub amount: f64,
}

/// Represents an Exchange Traded Fund (ETF) with its details.
#[derive(Debug, Deserialize)]
pub struct Etf {
    /// The symbol of the ETF.
    pub symbol: String,
    /// The name of the ETF.
    pub name: String,
    /// The current price of the ETF.
    pub price: f64,
    /// The total number of shares outstanding.
    pub shares_outstanding: f64,
}

/// Represents a Mutual Fund with its details.
#[derive(Debug, Deserialize)]
pub struct MutualFund {
    /// The symbol of the Mutual Fund.
    pub symbol: String,
    /// The name of the Mutual Fund.
    pub name: String,
    /// The current price of the Mutual Fund.
    pub price: f64,
    /// The net assets of the Mutual Fund.
    pub net_assets: f64,
}

/// Represents the allocation rules specifying the percentages for ETFs and Mutual Funds.
#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct AllocationRules {
    /// The percentage of the total investment to be allocated to ETFs.
    pub etf_percentage: f64,
    /// The percentage of the total investment to be allocated to Mutual Funds.
    pub mutual_fund_percentage: f64,
}

/// Trait defining common data that funds should provide.
pub trait FundData {
    /// Returns the symbol of the fund.
    fn symbol(&self) -> &str;
    /// Returns the name of the fund.
    fn name(&self) -> &str;
    /// Returns the total value of the fund.
    fn value(&self) -> f64;
}

impl FundData for Etf {
    /// Returns the symbol of the ETF.
    fn symbol(&self) -> &str {
        &self.symbol
    }

    /// Returns the name of the ETF.
    fn name(&self) -> &str {
        &self.name
    }

    /// Calculates and returns the total value of the ETF.
    /// This is computed as the price multiplied by the number of shares outstanding.
    fn value(&self) -> f64 {
        self.price * self.shares_outstanding
    }
}

impl FundData for MutualFund {
    /// Returns the symbol of the Mutual Fund.
    fn symbol(&self) -> &str {
        &self.symbol
    }

    /// Returns the name of the Mutual Fund.
    fn name(&self) -> &str {
        &self.name
    }

    /// Calculates and returns the total value of the Mutual Fund.
    /// This is computed as the price multiplied by the net assets.
    fn value(&self) -> f64 {
        self.price * self.net_assets
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_etf_is_valued_at_price_times_shares() {
        let etf: Etf = serde_json::from_str(
            r#"{"symbol":"SPY","name":"S&P 500","price":500.0,"shares_outstanding":2000.0}"#,
        )
        .unwrap();
        assert_eq!((etf.symbol(), etf.name(), etf.value()), ("SPY", "S&P 500", 1_000_000.0));
    }

    #[test]
    fn a_mutual_fund_is_valued_at_price_times_net_assets() {
        let fund: MutualFund = serde_json::from_str(
            r#"{"symbol":"VFIAX","name":"Admiral","price":2.0,"net_assets":300.0}"#,
        )
        .unwrap();
        assert_eq!((fund.symbol(), fund.name(), fund.value()), ("VFIAX", "Admiral", 600.0));
    }

    #[test]
    fn orders_and_rules_round_trip_through_json() {
        let order = AllocationOrder { symbol: "SPY".into(), name: "S&P 500".into(), amount: 12.5 };
        let back: AllocationOrder =
            serde_json::from_str(&serde_json::to_string(&order).unwrap()).unwrap();
        assert_eq!((back.symbol, back.amount), ("SPY".to_string(), 12.5));
        let rules = AllocationRules { etf_percentage: 0.6, mutual_fund_percentage: 0.4 };
        let back: AllocationRules =
            serde_json::from_str(&serde_json::to_string(&rules).unwrap()).unwrap();
        assert_eq!(back.etf_percentage + back.mutual_fund_percentage, 1.0);
    }
}
