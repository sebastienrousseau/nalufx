use nalufx::{
    services::{
        fetch_data::fetch_data,
        processing::{calculate_cash_flows, calculate_daily_returns},
    },
    utils::calculations::calculate_optimal_allocation,
};

#[tokio::main]
pub(crate) async fn main() {
    // Define the ticker symbol and initial investment amount
    let ticker = "SPY";
    let initial_investment = 100000.0;

    // Fetch historical closing prices for SPY
    match fetch_data(ticker, None, None).await {
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

            // Fetch or generate market indices data
            let market_indices = vec![1000.0, 1010.0, 1005.0, 1015.0]; // Replace with actual data
            println!("Market Indices: {:?}", market_indices);

            // Fetch or generate fund characteristics data
            let fund_characteristics = vec![0.8, 0.9, 0.85, 0.95]; // Replace with actual data
            println!("Fund Characteristics: {:?}", fund_characteristics);

            // Ensure all input slices have the same length
            let num_days = daily_returns.len();
            if num_days == cash_flows.len()
                && num_days == market_indices.len()
                && num_days == fund_characteristics.len()
            {
                // Calculate the optimal allocation based on historical data
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

                        println!("Please note that these recommendations are based on the historical data and should be considered as a starting point for your investment strategy.");
                        println!("It is always advisable to conduct further research and consult with a financial advisor before making any investment decisions.");
                    }
                    Err(e) => {
                        eprintln!("Error calculating optimal allocation: {}", e);
                        println!("Please check the input data and try again.");
                    }
                }
            } else {
                eprintln!("Input slices have different lengths.");
                println!("Please ensure that daily returns, cash flows, market indices, and fund characteristics have the same length.");

                // Truncate or pad the input slices to have the same length
                let min_length = num_days
                    .min(cash_flows.len())
                    .min(market_indices.len())
                    .min(fund_characteristics.len());
                let daily_returns = &daily_returns[..min_length];
                let cash_flows = &cash_flows[..min_length];
                let market_indices = &market_indices[..min_length];
                let fund_characteristics = &fund_characteristics[..min_length];

                // Calculate the optimal allocation based on the truncated/padded input slices
                let optimal_allocation_result = calculate_optimal_allocation(
                    daily_returns,
                    cash_flows,
                    market_indices,
                    fund_characteristics,
                    min_length,
                );

                match optimal_allocation_result {
                    Ok(optimal_allocation) => {
                        // Print the optimal allocation with descriptive information
                        println!("Optimal Allocation (based on truncated/padded input slices):");
                        println!("The optimal allocation represents the recommended distribution of funds for the next {} days.", min_length);
                        println!("Each value in the allocation vector corresponds to the percentage of funds to be allocated to {} for a specific day.", ticker);
                        println!(
                            "The sum of all values in the allocation vector should be close to 1.0 (100%)."
                        );
                        println!("Optimal Allocation: {:?}", optimal_allocation);

                        // Provide specific recommendations based on the optimal allocation and initial investment
                        println!("Recommendation (based on truncated/padded input slices):");
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

                        println!("Please note that these recommendations are based on the historical data and should be considered as a starting point for your investment strategy.");
                        println!("It is always advisable to conduct further research and consult with a financial advisor before making any investment decisions.");
                    }
                    Err(e) => {
                        eprintln!("Error calculating optimal allocation: {}", e);
                        println!("Please check the input data and try again.");
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("Error fetching data for ticker {}: {}", ticker, e);
        }
    }
}
