use chrono::Utc;
use nalufx::{
    services::{
        fetch_data::fetch_data,
        processing::{calculate_cash_flows, calculate_daily_returns},
    },
    utils::calculations::calculate_optimal_allocation,
};
use num_format::{Locale, ToFormattedString};

// Custom function to format float as currency
fn format_currency(value: f64) -> String {
    let int_value = (value * 100.0).round() as i64; // Convert to integer cents
    let dollars = int_value / 100;
    let cents = (int_value % 100).abs(); // Absolute value for cents
    format!("${}.{:02}", dollars.to_formatted_string(&Locale::en), cents)
}

#[tokio::main]
pub(crate) async fn main() {
    // Define the ticker symbol and initial investment amount
    let ticker = "SPY";
    let initial_investment = 100000.0;

    // Fetch historical closing prices for SPY
    match fetch_data(ticker, None, None).await {
        Ok(closes) => {
            // Calculate daily returns from closing prices
            let daily_returns = calculate_daily_returns(&closes);

            // Calculate cash flows based on daily returns and initial investment
            let cash_flows = calculate_cash_flows(&daily_returns, initial_investment);

            // Fetch or generate market indices data
            let market_indices = vec![1000.0, 1010.0, 1005.0, 1015.0]; // Replace with actual data
            println!("\n--- Market Overview ---\n");
            println!(
                "The Market Indices represent key points of market performance during the period:\n"
            );
            for (i, value) in market_indices.iter().enumerate() {
                println!("- Index Point {}: {}", i + 1, format_currency(*value));
            }

            // Fetch or generate fund characteristics data
            let fund_characteristics = vec![0.8, 0.9, 0.85, 0.95]; // Replace with actual data
            println!(
                "\nThe Fund Characteristics represent key attributes of the fund during the period:\n"
            );
            for (i, value) in fund_characteristics.iter().enumerate() {
                println!("- Attribute {}: {:.2}", i + 1, value);
            }

            // Determine the minimum length of all input slices
            let min_length = daily_returns
                .len()
                .min(cash_flows.len())
                .min(market_indices.len())
                .min(fund_characteristics.len());

            // Truncate all slices to the minimum length
            let daily_returns = &daily_returns[..min_length];
            let cash_flows = &cash_flows[..min_length];
            let market_indices = &market_indices[..min_length];
            let fund_characteristics = &fund_characteristics[..min_length];

            // Calculate the optimal allocation based on truncated input slices
            let optimal_allocation_result = calculate_optimal_allocation(
                daily_returns,
                cash_flows,
                market_indices,
                fund_characteristics,
                min_length,
            );

            match optimal_allocation_result {
                Ok(mut optimal_allocation) => {
                    // Filter out negative allocations and normalize the rest
                    optimal_allocation = optimal_allocation
                        .into_iter()
                        .map(|alloc| if alloc < 0.0 { 0.0 } else { alloc })
                        .collect();
                    let total_allocation: f64 = optimal_allocation.iter().sum();
                    optimal_allocation = optimal_allocation
                        .into_iter()
                        .map(|alloc| alloc / total_allocation)
                        .collect();

                    // Print the optimal allocation with descriptive information
                    println!("\n--- Optimal Allocation Report ---\n");
                    println!(
                        "The optimal allocation represents the recommended distribution of funds for the next {} days.",
                        min_length
                    );
                    println!(
                        "Each value in the allocation vector corresponds to the percentage of funds to be allocated to {} for a specific day.",
                        ticker
                    );
                    println!("The sum of all values in the allocation vector should be close to 1.0 (100%).");
                    println!("\n- Optimal Allocation: {:?}", optimal_allocation);

                    // Provide specific recommendations based on the optimal allocation and initial investment
                    println!("\n--- Investment Recommendations ---\n");
                    println!(
                        "Based on the optimal allocation and your initial investment of {}, it is recommended to distribute your funds as follows:\n",
                        format_currency(initial_investment)
                    );

                    let today = Utc::now();
                    for (i, &allocation) in optimal_allocation.iter().enumerate() {
                        let allocation_amount = allocation * initial_investment;
                        let allocation_date = today + chrono::Duration::days(i as i64);
                        let allocation_percentage = allocation * 100.0;
                        println!(
                            "- Day {}: {} - Allocate {} ({:.2}%) to {}",
                            i + 1,
                            allocation_date.format("%Y-%m-%d"),
                            format_currency(allocation_amount),
                            allocation_percentage,
                            ticker
                        );
                    }

                    println!("\n--- Disclaimer ---\n");
                    println!("These recommendations are based on historical data and should be considered as a starting point for your investment strategy.");
                    println!("Market conditions can change rapidly, and past performance is not indicative of future results.");
                    println!("It is always advisable to conduct further research and consult with a financial advisor before making any investment decisions.\n");
                }
                Err(e) => {
                    eprintln!("Error calculating optimal allocation: {}", e);
                    println!("Please check the input data and try again.");
                }
            }
        }
        Err(e) => {
            eprintln!("Error fetching data for ticker {}: {}", ticker, e);
        }
    }
}
