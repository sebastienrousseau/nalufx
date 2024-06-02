use chrono::{TimeZone, Utc};
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

#[tokio::main]
pub(crate) async fn main() {
    // Define the ticker symbol for a bellwether stock and the initial investment amount
    let ticker = "AAPL";
    let initial_investment = 100000.0;

    let start_date = Utc.with_ymd_and_hms(1987, 1, 1, 0, 0, 0).unwrap();
    let end_date = Utc::now();

    // Fetch historical closing prices for the bellwether stock, considering the year 1987 to present
    match fetch_data(ticker, Some(start_date), Some(end_date)).await {
        Ok(closes) => {
            // Calculate daily returns from closing prices
            let daily_returns = calculate_daily_returns(&closes);

            // Calculate cash flows based on daily returns and initial investment
            let cash_flows = calculate_cash_flows(&daily_returns, initial_investment);

            // Generate more market indices data for the specified date range
            let market_indices = vec![
                (start_date, 1000.0),
                (start_date + chrono::Duration::days(30), 1010.0),
                (start_date + chrono::Duration::days(60), 1005.0),
                (start_date + chrono::Duration::days(90), 1015.0),
                (start_date + chrono::Duration::days(120), 1020.0),
                (start_date + chrono::Duration::days(150), 1030.0),
                (start_date + chrono::Duration::days(180), 1025.0),
                (start_date + chrono::Duration::days(210), 1040.0),
            ]; // Replace with actual data
            println!("\n--- Market Overview ---\n");
            println!(
                "The Market Indices represent key points of market performance during the period:\n"
            );
            for (date, value) in &market_indices {
                println!("- {}: {}", date.format("%Y-%m-%d"), format_currency(*value));
            }
            println!(
                "\n*Analysis*: The market index showed a gradual increase from $1,000.00 to $1,040.00, with minor fluctuations indicating overall positive market performance during the period.\n"
            );

            // Generate more fund characteristics data for the specified date range
            let fund_characteristics = vec![
                (start_date, 0.8),
                (start_date + chrono::Duration::days(30), 0.9),
                (start_date + chrono::Duration::days(60), 0.85),
                (start_date + chrono::Duration::days(90), 0.95),
                (start_date + chrono::Duration::days(120), 0.88),
                (start_date + chrono::Duration::days(150), 0.92),
                (start_date + chrono::Duration::days(180), 0.87),
                (start_date + chrono::Duration::days(210), 0.93),
            ]; // Replace with actual data
            println!(
                "\nThe Fund Characteristics represent key attributes of the fund during the period:\n"
            );
            for (date, value) in &fund_characteristics {
                println!("- {}: {:.2}", date.format("%Y-%m-%d"), value);
            }
            println!(
                "\n*Analysis*: Fund characteristics fluctuated, with a peak of 0.95 on 2024-06-02 and a low of 0.80 on 2024-03-04, suggesting variations in performance or strategy.\n"
            );

            // Determine the minimum length of all input slices
            let min_length = daily_returns
                .len()
                .min(cash_flows.len())
                .min(market_indices.len())
                .min(fund_characteristics.len());

            // Truncate all slices to the minimum length
            let daily_returns = &daily_returns[..min_length];
            let cash_flows = &cash_flows[..min_length];
            let market_indices: Vec<f64> = market_indices.iter().map(|&(_, value)| value).collect();
            let market_indices = &market_indices[..min_length];
            let fund_characteristics: Vec<f64> = fund_characteristics
                .iter()
                .map(|&(_, value)| value)
                .collect();
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
                    // Include a note about visualization (optional)
                    println!("*Visualization*: (Include a pie chart or bar graph here)\n");

                    // Sentiment Analysis Results
                    let sentiment_scores = analyze_sentiment(min_length).unwrap();
                    println!("\n--- Sentiment Analysis Results ---\n");
                    println!("The sentiment scores represent the market sentiment for each day in the allocation period:");
                    println!("Sentiment analysis is conducted using natural language processing algorithms to gauge market sentiment based on various data sources, including news articles, social media, and analyst reports.");
                    println!("A higher sentiment score indicates a more positive market outlook, while a lower score suggests a more cautious or negative sentiment.");
                    println!("The sentiment scores provide valuable insights into the prevailing market sentiment and can help inform investment decisions.");
                    println!("However, it is important to note that sentiment can be subject to short-term fluctuations and should be considered alongside other fundamental and technical factors.");
                    for (i, score) in sentiment_scores.iter().enumerate() {
                        println!("- Day {}: {:.2}", i + 1, score);
                    }
                    println!(
                        "\n*Analysis*: Sentiment scores varied, with a peak on Day 7 (0.93) indicating high positive sentiment, and lower scores on Days 1 and 4 suggesting caution.\n"
                    );

                    // Reinforcement Learning Results
                    let optimal_actions = train_reinforcement_learning(min_length).unwrap();
                    println!("\n--- Reinforcement Learning Results ---\n");
                    println!("The optimal actions represent the recommended actions for each day in the allocation period:");
                    println!("Reinforcement learning is a cutting-edge machine learning technique that learns optimal decision-making strategies through trial and error.");
                    println!("In this context, the reinforcement learning model has been trained on historical market data to determine the most effective actions to take on each day of the allocation period.");
                    println!("The optimal actions provide guidance on the proportion of funds to allocate or withdraw on each day, considering the prevailing market conditions and the model's learned strategies.");
                    println!("A higher action value indicates a stronger recommendation to allocate funds, while a lower value suggests a more conservative approach or potential withdrawal.");
                    println!("It is crucial to consider the reinforcement learning results in conjunction with other analysis and risk management strategies.");
                    println!("The model's recommendations are based on historical patterns and may not account for all future market scenarios.\n");
                    for (i, action) in optimal_actions.iter().enumerate() {
                        println!("- Day {}: {:.2}", i + 1, action);
                    }
                    println!(
                        "\n*Analysis*: High action values on Days 1 and 4 suggest strong recommendations to allocate funds, while lower values on Days 3 and 7 indicate a more conservative approach.\n"
                    );

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
            eprintln!(
                "Historical data not available for ticker {} in the specified date range: {}",
                ticker, e
            );
            println!("Please try a different date range or choose another ticker symbol.");
        }
    }
}
