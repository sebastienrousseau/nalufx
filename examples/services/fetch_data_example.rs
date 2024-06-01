use env_logger;
use nalufx::{
    services::{
        fetch_data::fetch_data,
        processing::{calculate_cash_flows, calculate_daily_returns},
    },
    utils::calculations::calculate_optimal_allocation,
};

/// The main function that demonstrates the usage of the nalufx library for the SPDR S&P 500 ETF Trust (SPY).
///
/// SPY is an exchange-traded fund (ETF) that tracks the performance of the S&P 500 index, which comprises 500 large-cap U.S. stocks.
///
/// This function performs the following steps:
/// 1. Initializes the logger using `env_logger`.
/// 2. Defines the ticker symbol (SPY) and initial investment amount.
/// 3. Fetches historical closing prices for SPY using `fetch_data`.
/// 4. Calculates daily returns from the closing prices using `calculate_daily_returns`.
/// 5. Calculates cash flows based on the daily returns and initial investment using `calculate_cash_flows`.
/// 6. Assumes some predictions for the next 3 days for demonstration purposes.
/// 7. Calculates the optimal allocation based on the predictions using `calculate_optimal_allocation`.
/// 8. Prints the results, including the recommended allocation of funds for each day, or an error message if fetching data fails.
///
/// ```
#[tokio::main]
pub(crate) async fn main() {
    // Initialize the logger
    env_logger::init();

    // Define the ticker symbol and initial investment amount
    let ticker = "SPY";
    let initial_investment = 100000.0;

    // Fetch historical closing prices for SPY
    match fetch_data(ticker).await {
        Ok(closes) => {
            println!(
                "Successfully fetched closing prices for {}: {:?}",
                ticker, closes
            );

            // Calculate daily returns from closing prices
            let daily_returns = calculate_daily_returns(&closes);
            println!("Daily returns for {}: {:?}", ticker, daily_returns);

            // Calculate cash flows based on daily returns and initial investment
            let cash_flows = calculate_cash_flows(&daily_returns, initial_investment);
            println!("Cash Flows for {}: {:?}", ticker, cash_flows);

            // For demonstration, let's assume we have some predictions for the next 3 days
            let predictions = vec![1.0, 2.0, 3.0];

            // Calculate the optimal allocation based on predictions
            let optimal_allocation = calculate_optimal_allocation(&predictions);

            // Print the optimal allocation with descriptive information
            println!("Optimal Allocation:");
            println!("The optimal allocation represents the recommended distribution of funds for the next {} days.", predictions.len());
            println!("Each value in the allocation vector corresponds to the percentage of funds to be allocated to {} for a specific day.", ticker);
            println!(
                "The sum of all values in the allocation vector should be close to 1.0 (100%)."
            );
            println!("Optimal Allocation: {:?}", optimal_allocation);

            // Provide specific recommendations based on the optimal allocation and initial investment
            println!("Recommendation:");
            println!("Based on the optimal allocation and your initial investment of ${}, it is recommended to distribute your funds as follows:", initial_investment);

            for (i, &allocation) in optimal_allocation.iter().enumerate() {
                let allocation_amount = allocation * initial_investment;
                println!(
                    "- Day {}: Allocate ${:.2} to {}",
                    i + 1,
                    allocation_amount,
                    ticker
                );
            }

            println!("Please note that these recommendations are based on the provided predictions and should be considered as a starting point for your investment strategy.");
            println!("It is always advisable to conduct further research and consult with a financial advisor before making any investment decisions.");
        }
        Err(e) => {
            eprintln!("Error fetching data for ticker {}: {}", ticker, e);
        }
    }
}
