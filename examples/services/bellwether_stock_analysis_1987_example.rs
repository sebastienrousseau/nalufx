use chrono::{TimeZone, Utc};
use nalufx::{
    services::{
        fetch_data::fetch_data,
        processing::{calculate_cash_flows, calculate_daily_returns},
    },
    utils::calculations::calculate_optimal_allocation,
};

#[tokio::main]
pub(crate) async fn main() {
    // Define the ticker symbol for a bellwether stock and the initial investment amount
    let ticker = "AAPL";
    let initial_investment = 100000.0;

    let start_date = Utc.with_ymd_and_hms(1987, 1, 1, 0, 0, 0).unwrap();
    let end_date = Utc.with_ymd_and_hms(1987, 12, 31, 23, 59, 59).unwrap();

    // Fetch historical closing prices for the bellwether stock, considering the year 1987
    match fetch_data(ticker, Some(start_date), Some(end_date)).await {
        Ok(closes) => {
            println!(
                "Successfully fetched closing prices for {} in 1987: {:?}",
                ticker, closes
            );

            // Calculate daily returns from closing prices
            let daily_returns = calculate_daily_returns(&closes);
            println!("Daily returns for {} in 1987: {:?}", ticker, daily_returns);

            // Calculate cash flows based on daily returns and initial investment
            let cash_flows = calculate_cash_flows(&daily_returns, initial_investment);
            println!("Cash Flows for {} in 1987: {:?}", ticker, cash_flows);

            // Fetch or generate market indices data for the specified date range
            let market_indices = vec![1000.0, 1010.0, 1005.0, 1015.0]; // Replace with actual data
            println!("Market Indices in 1987: {:?}", market_indices);

            // Fetch or generate fund characteristics data for the specified date range
            let fund_characteristics = vec![0.8, 0.9, 0.85, 0.95]; // Replace with actual data
            println!("Fund Characteristics in 1987: {:?}", fund_characteristics);

            // Calculate the optimal allocation based on historical data
            let num_days = 3;
            let optimal_allocation_result = calculate_optimal_allocation(
                &daily_returns,
                &cash_flows,
                &market_indices,
                &fund_characteristics,
                num_days,
            );

            match optimal_allocation_result {
                Ok(optimal_allocation) => {
                    // Print the optimal allocation with descriptive information
                    println!("Optimal Allocation:");
                    println!("The optimal allocation represents the recommended distribution of funds for the next {} days.", num_days);
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

                    println!("Please note that these recommendations are based on the historical data from 1987, which includes the significant market event known as 'Black Monday' or 'Armageddon'.");
                    println!("It is always advisable to conduct further research and consult with a financial advisor before making any investment decisions.");
                }
                Err(e) => {
                    eprintln!("Error calculating optimal allocation: {}", e);
                    println!("Please check the input data and try again.");
                }
            }
        }
        Err(e) => {
            eprintln!(
                "Historical data not available for ticker {} in the specified date range: {}",
                ticker, e
            );
            println!("Please try a different date range or choose another ticker symbol.");
        }
    }
}
