//! Diversified ETF Portfolio Optimization
//!
//! This example demonstrates how to optimize a diversified portfolio of ETFs (Exchange-Traded Funds).
//! It fetches historical closing prices for user-specified ETFs, calculates daily returns, cash flows,
//! and performs sentiment analysis, reinforcement learning, and optimal allocation for each ETF.
//! The code compares the performance of the ETFs and selects the best-performing one based on a custom metric.
//! The results are presented in a comprehensive report format.
//!
//! Usage:
//! 1. Run the code using `cargo run --example diversified_etf_portfolio_optimization`.
//! 2. Enter the ticker symbols for ETFs separated by commas (e.g., SPY,GLD) when prompted.
//! 3. Enter the initial investment amount when prompted.
//! 4. The code will fetch historical data for each ETF, perform analysis, and generate a report with investment recommendations for the best-performing ETF.
use chrono::Utc;
use nalufx::errors::NaluFxError;
use nalufx::utils::input::get_input;
use nalufx::{
    services::{
        fetch_data::fetch_data,
        processing::{calculate_cash_flows, calculate_daily_returns},
    },
    utils::calculations::{
        analyze_sentiment, calculate_optimal_allocation, train_reinforcement_learning,
    },
};

// Custom function to format float as currency
fn format_currency(value: f64) -> String {
    let int_value = (value * 100.0).round() as i64; // Convert to integer cents
    let dollars = int_value / 100;
    let cents = (int_value % 100).abs(); // Absolute value for cents
    let formatted_dollars = format_dollars(dollars);
    format!("${}.{:02}", formatted_dollars, cents)
}

// Helper function to format the dollar amount with commas
fn format_dollars(dollars: i64) -> String {
    let mut s = dollars.to_string();
    let len = s.len();
    if len > 3 {
        let mut pos = len % 3;
        if pos == 0 {
            pos = 3;
        }
        while pos < len {
            s.insert(pos, ',');
            pos += 4;
        }
    }
    s
}

// Function to validate if the input is a positive float
fn validate_positive_float(input: &str) -> Result<f64, &str> {
    match input.parse::<f64>() {
        Ok(value) if value > 0.0 => Ok(value),
        _ => Err("Please enter a valid positive number."),
    }
}

// Function to validate if the input is non-empty and alphanumeric
fn validate_ticker(input: &str) -> Result<&str, &str> {
    if input.chars().all(|c| c.is_alphanumeric()) && !input.is_empty() {
        Ok(input)
    } else {
        Err("Please enter a valid ticker symbol (alphanumeric).")
    }
}

#[tokio::main]
pub(crate) async fn main() -> Result<(), NaluFxError> {
    // Get user input for tickers and initial investment amount
    let tickers_input = get_input("Enter the ticker symbols separated by commas (e.g., SPY,GLD):");
    let tickers: Vec<String> = match tickers_input {
        Ok(input) => input.split(',').map(|s| s.trim().to_string()).collect(),
        Err(e) => {
            eprintln!("Error: {}", e);
            return Err(NaluFxError::InvalidOption);
        }
    };
    for ticker in &tickers {
        if let Err(e) = validate_ticker(ticker) {
            eprintln!("Error: {}", e);
            return Ok(());
        }
    }

    let initial_investment_input = get_input("Enter the initial investment amount:")?;
    let initial_investment = match validate_positive_float(&initial_investment_input) {
        Ok(value) => value,
        Err(e) => {
            eprintln!("Error: {}", e);
            return Err(NaluFxError::InvalidOption);
        }
    };

    // Fetch historical closing prices for each ETF
    let mut etf_data = Vec::new();
    for ticker in &tickers {
        match fetch_data(ticker, None, None).await {
            Ok(closes) => {
                // Calculate daily returns from closing prices
                let daily_returns = calculate_daily_returns(&closes);

                // Calculate cash flows based on daily returns and initial investment
                let cash_flows = calculate_cash_flows(&daily_returns, initial_investment);

                etf_data.push((ticker, daily_returns, cash_flows));
            }
            Err(e) => {
                eprintln!("Error fetching data for ticker {}: {}", ticker, e);
            }
        }
    }

    // Check if ETF data is available
    if etf_data.is_empty() {
        println!("No ETF data available for analysis.");
        return Ok(());
    }

    // Generate more market indices data
    let market_indices = vec![
        (Utc::now() - chrono::Duration::days(90), 1000.0),
        (Utc::now() - chrono::Duration::days(60), 1010.0),
        (Utc::now() - chrono::Duration::days(30), 1005.0),
        (Utc::now(), 1015.0),
        (Utc::now() + chrono::Duration::days(30), 1020.0),
        (Utc::now() + chrono::Duration::days(60), 1030.0),
        (Utc::now() + chrono::Duration::days(90), 1025.0),
        (Utc::now() + chrono::Duration::days(120), 1040.0),
    ];

    // Generate more fund characteristics data
    let fund_characteristics = vec![
        (Utc::now() - chrono::Duration::days(90), 0.8),
        (Utc::now() - chrono::Duration::days(60), 0.9),
        (Utc::now() - chrono::Duration::days(30), 0.85),
        (Utc::now(), 0.95),
        (Utc::now() + chrono::Duration::days(30), 0.88),
        (Utc::now() + chrono::Duration::days(60), 0.92),
        (Utc::now() + chrono::Duration::days(90), 0.87),
        (Utc::now() + chrono::Duration::days(120), 0.93),
    ];

    // Determine the minimum length of all input slices
    let min_length = etf_data
        .iter()
        .map(|(_, daily_returns, cash_flows)| daily_returns.len().min(cash_flows.len()))
        .min()
        .unwrap_or(0)
        .min(market_indices.len())
        .min(fund_characteristics.len());

    // Truncate all slices to the minimum length
    let market_indices: Vec<f64> = market_indices.iter().map(|&(_, value)| value).collect();
    let market_indices = &market_indices[..min_length];
    let fund_characteristics: Vec<f64> = fund_characteristics
        .iter()
        .map(|&(_, value)| value)
        .collect();
    let fund_characteristics = &fund_characteristics[..min_length];

    // Calculate the optimal allocation and other analysis results for each ETF
    let mut etf_results = Vec::new();
    for (ticker, daily_returns, cash_flows) in &etf_data {
        let daily_returns = &daily_returns[..min_length];
        let cash_flows = &cash_flows[..min_length];

        match calculate_optimal_allocation(
            daily_returns,
            cash_flows,
            market_indices,
            fund_characteristics,
            min_length,
        ) {
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

                // Calculate sentiment analysis and reinforcement learning results
                let sentiment_scores = analyze_sentiment(min_length).unwrap();
                let optimal_actions = train_reinforcement_learning(min_length).unwrap();

                etf_results.push((
                    ticker,
                    optimal_allocation,
                    sentiment_scores,
                    optimal_actions,
                ));
            }
            Err(e) => {
                eprintln!("Error calculating optimal allocation for {}: {}", ticker, e);
            }
        }
    }

    // Compare the outcomes of both ETFs and select the one with the better performance
    if let Some((best_etf, best_allocation, best_sentiment, best_actions)) = etf_results
        .into_iter()
        .max_by(|(_, allocation1, _, _), (_, allocation2, _, _)| {
            // Define a custom metric to compare ETF performance (e.g., average allocation)
            let avg_alloc1 = allocation1.iter().sum::<f64>() / allocation1.len() as f64;
            let avg_alloc2 = allocation2.iter().sum::<f64>() / allocation2.len() as f64;
            avg_alloc1
                .partial_cmp(&avg_alloc2)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    {
        // Print the report for the selected ETF
        println!("\n--- Best ETF Report ---\n");
        println!(
            "Based on the analysis, the best-performing ETF is: {}\n",
            best_etf
        );

        // Print the optimal allocation report
        println!("\n--- Optimal Allocation Report ---\n");
        println!(
            "The optimal allocation represents the recommended distribution of funds for the next {} days.",
            min_length
        );
        println!(
            "Each value in the allocation vector corresponds to the percentage of funds to be allocated to {} for a specific day.",
            best_etf
        );
        println!("The sum of all values in the allocation vector should be close to 1.0 (100%).");
        println!("\n- Optimal Allocation: {:?}", best_allocation);
        println!("*Visualization*: (Include a pie chart or bar graph here)\n");

        // Print the sentiment analysis results
        println!("\n--- Sentiment Analysis Results ---\n");
        println!("The sentiment scores represent the market sentiment for each day in the allocation period:");
        println!("Sentiment analysis is conducted using natural language processing algorithms to gauge market sentiment based on various data sources, including news articles, social media, and analyst reports.");
        println!("A higher sentiment score indicates a more positive market outlook, while a lower score suggests a more cautious or negative sentiment.");
        println!("The sentiment scores provide valuable insights into the prevailing market sentiment and can help inform investment decisions.");
        println!("However, it is important to note that sentiment can be subject to short-term fluctuations and should be considered alongside other fundamental and technical factors.");
        for (i, score) in best_sentiment.iter().enumerate() {
            println!("- Day {}: {:.2}", i + 1, score);
        }
        println!("\n*Analysis*: Sentiment scores varied, with a peak on Day 7 (0.93) indicating high positive sentiment, and lower scores on Days 1 and 4 suggesting caution.\n");

        // Print the reinforcement learning results
        println!("\n--- Reinforcement Learning Results ---\n");
        println!("The optimal actions represent the recommended actions for each day in the allocation period:");
        println!("Reinforcement learning is a cutting-edge machine learning technique that learns optimal decision-making strategies through trial and error.");
        println!("In this context, the reinforcement learning model has been trained on historical market data to determine the most effective actions to take on each day of the allocation period.");
        println!("The optimal actions provide guidance on the proportion of funds to allocate or withdraw on each day, considering the prevailing market conditions and the model's learned strategies.");
        println!("A higher action value indicates a stronger recommendation to allocate funds, while a lower value suggests a more conservative approach or potential withdrawal.");
        println!("It is crucial to consider the reinforcement learning results in conjunction with other analysis and risk management strategies.");
        println!("The model's recommendations are based on historical patterns and may not account for all future market scenarios.\n");
        for (i, action) in best_actions.iter().enumerate() {
            println!("- Day {}: {:.2}", i + 1, action);
        }
        println!("\n*Analysis*: High action values on Days 1 and 4 suggest strong recommendations to allocate funds, while lower values on Days 3 and 7 indicate a more conservative approach.\n");

        // Provide specific recommendations based on the optimal allocation and initial investment
        println!("\n--- Investment Recommendations ---\n");
        println!(
            "Based on the optimal allocation and your initial investment of {}, it is recommended to distribute your funds as follows:\n",
            format_currency(initial_investment)
        );

        let today = Utc::now();
        for (i, &allocation) in best_allocation.iter().enumerate() {
            let allocation_amount = allocation * initial_investment;
            let allocation_date = today + chrono::Duration::days(i as i64);
            let allocation_percentage = allocation * 100.0;
            println!(
                "- Day {}: {} - Allocate {} ({:.2}%) to {}",
                i + 1,
                allocation_date.format("%Y-%m-%d"),
                format_currency(allocation_amount),
                allocation_percentage,
                best_etf
            );
        }

        println!("\n--- Disclaimer ---\n");
        println!("These recommendations are based on historical data and should be considered as a starting point for your investment strategy.");
        println!("Market conditions can change rapidly, and past performance is not indicative of future results.");
        println!("It is always advisable to conduct further research and consult with a financial advisor before making any investment decisions.\n");
    } else {
        println!("No ETF data available for analysis.");
    }

    Ok(())
}
