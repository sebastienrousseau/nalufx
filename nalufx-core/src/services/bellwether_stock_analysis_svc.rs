use crate::{
    errors::NaluFxError,
    services::{
        fetch_data_svc::fetch_data,
        processing_svc::{calculate_cash_flows, calculate_daily_returns},
    },
    utils::{
        calculations::{
            analyze_sentiment, calculate_optimal_allocation, train_reinforcement_learning,
        },
        date::validate_date,
    },
};
use chrono::Datelike;
use chrono::Utc;
use nalufx_llms::llms::LLM;
use reqwest::Client;

/// Generates an analysis report based on historical stock data, optimal allocation, and LLM analysis.
///
/// # Arguments
///
/// * `llm` - A boxed trait object implementing the LLM trait for language model operations.
/// * `client` - A reference to the reqwest Client for making HTTP requests.
/// * `api_key` - A string reference to the API key for accessing the LLM service.
/// * `ticker` - A string reference to the ticker symbol of the stock to analyze.
/// * `initial_investment` - A f64 representing the initial investment amount.
/// * `start_date` - A string reference to the start date of the analysis period in "YYYY-MM-DD" format.
/// * `end_date` - A string reference to the end date of the analysis period in "YYYY-MM-DD" format.
///
/// # Returns
///
/// * `Result<(), NaluFxError>` - Returns Ok(()) if the analysis is successful, otherwise returns an error.
pub async fn generate_analysis(
    llm: Box<dyn LLM>,
    client: &Client,
    api_key: &str,
    ticker: &str,
    initial_investment: f64,
    start_date: &str,
    end_date: &str,
) -> Result<(), NaluFxError> {
    let start_date = match validate_date(start_date) {
        Ok(date) => date,
        Err(e) => {
            eprintln!("Error: {}", e);
            return Err(NaluFxError::InvalidOption);
        }
    };

    let end_date = match validate_date(end_date) {
        Ok(date) => date,
        Err(e) => {
            eprintln!("Error: {}", e);
            return Err(NaluFxError::InvalidOption);
        }
    };

    match fetch_data(ticker, Some(start_date), Some(end_date)).await {
        Ok(closes) => {
            if closes.is_empty() {
                eprintln!(
                    "No closing prices found for ticker {} in the specified date range",
                    ticker
                );
                return Ok(());
            }

            let daily_returns = calculate_daily_returns(&closes);
            let cash_flows = calculate_cash_flows(&daily_returns, initial_investment);

            let min_length = daily_returns.len().min(cash_flows.len());
            let daily_returns = &daily_returns[..min_length];
            let cash_flows = &cash_flows[..min_length];

            let optimal_allocation_result = calculate_optimal_allocation(
                daily_returns,
                cash_flows,
                &vec![1.0; min_length],
                &vec![1.0; min_length],
                min_length,
            );

            match optimal_allocation_result {
                Ok(mut optimal_allocation) => {
                    optimal_allocation = optimal_allocation
                        .into_iter()
                        .map(|alloc| if alloc < 0.0 { 0.0 } else { alloc })
                        .collect();
                    let total_allocation: f64 = optimal_allocation.iter().sum();
                    if total_allocation == 0.0 {
                        eprintln!("Error: Total allocation is zero for ticker {}", ticker);
                        return Ok(());
                    }
                    optimal_allocation = optimal_allocation
                        .into_iter()
                        .map(|alloc| alloc / total_allocation)
                        .collect();
                    let current_year = Utc::now().year();
                    let prompt = format!(
                        "Analyze the following stock data for {}:\n\n\
                        - Optimal Allocation: {:?}\n\n\
                        Provide a detailed investment recommendation based on this data.\n\
                        Additionally, provide the Current Market Context for {} in {}.\n\
                        This context is essential for understanding the potential drivers behind the stock's performance and the recommendations provided.",
                        ticker, optimal_allocation, ticker, current_year
                    );

                    let response = llm.send_request(client, api_key, &prompt, 1500).await?;
                    let message = response["choices"][0]["message"]["content"]
                        .as_str()
                        .unwrap_or("");

                    // Extract key findings from the message
                    let key_findings = "\n--- Key findings ---\n\n";
                    let mut summary = key_findings.to_string();
                    for line in message.lines() {
                        if line.contains(ticker) {
                            summary.push_str(line);
                            summary.push('\n');
                        }
                    }

                    let sentiment_scores = match analyze_sentiment(min_length) {
                        Ok(scores) => scores,
                        Err(e) => {
                            eprintln!("Error in sentiment analysis for ticker {}: {}", ticker, e);
                            Vec::new()
                        }
                    };

                    let optimal_actions = match train_reinforcement_learning(min_length) {
                        Ok(actions) => actions,
                        Err(e) => {
                            eprintln!(
                                "Error in reinforcement learning for ticker {}: {}",
                                ticker, e
                            );
                            Vec::new()
                        }
                    };

                    // Generate Report
                    println!("\n--- Bellwether Stock Report: {} ---\n", ticker);
                    println!("- **Date Range:** {} - {}", start_date, end_date);
                    println!("- **Initial Investment:** ${:.2}\n", initial_investment);

                    // Explanation of Methodology
                    println!("--- Methodology ---\n");
                    println!("This report combines several analytical techniques to provide a comprehensive view of {}'s potential performance:", ticker);
                    println!("\n- **Optimal Allocation:** Uses historical price data and statistical modelling to suggest a daily allocation of your investment amount to {}. This helps in balancing risk and maximizing returns by identifying optimal investment proportions.", ticker);
                    println!("- **Sentiment Analysis:** Gauges market sentiment towards {} by analysing news articles, social media, and other relevant sources. This helps in understanding the market's perception and potential impact on stock performance.", ticker);
                    println!("- **Reinforcement Learning (RL):** A machine learning model trained on historical data to suggest buy/sell actions based on market conditions. This helps in identifying strategic actions to maximize returns based on learned patterns.\n");

                    // Summary of Key Findings
                    println!("{}", summary);

                    // Current Market Context
                    let current_year = Utc::now().year();
                    println!("\n--- Current Market Context ---\n");
                    println!("As of the analysis period {}, {} has been experiencing the following market conditions:", current_year, ticker);
                    println!("\n- **Technological Innovations:** {} is known for its continuous focus on technological innovations. The market is closely watching for any new product launches or updates that could impact {}'s stock performance.", ticker, ticker);
                    println!("\n- **Competition:** {} faces stiff competition from other tech giants. Any advancements or setbacks from competitors could impact {}'s market position and stock performance.", ticker, ticker);
                    println!("\n- **Macroeconomic Factors:** Economic indicators, inflation rates, interest rates, and government policies can all affect the stock market in general and {} specifically. Monitoring these macroeconomic factors is essential for predicting {}'s stock performance.", ticker, ticker);
                    println!("\n- **Regulatory Environment:** Changes in regulations related to data privacy, antitrust laws, or other regulatory issues can have a significant impact on {}'s business operations and stock performance.", ticker);
                    println!("\n- **Global Events:** Geopolitical events, natural disasters, pandemics, and other global factors can also influence {}'s stock performance. Keeping an eye on such events is essential for understanding the broader market context.\n", ticker);

                    println!("\n--- Key Findings ---\n");
                    println!("- **1. Optimal Allocation:** The model recommends a diversified approach, with daily allocations within a diversified portfolio containing {} ranging from {:.2}% to {:.2}% of your initial investment. This aims to mitigate risk and capture potential gains across different market conditions.\n", ticker, optimal_allocation.iter().cloned().fold(0./0., f64::min) * 100.0, optimal_allocation.iter().cloned().fold(0./0., f64::max) * 100.0);
                    println!("- **2. Sentiment Analysis:** Market sentiment towards {} fluctuates within the specified period, ranging from very positive ({:.2} on Day {}) to somewhat negative ({:.2} on Day {}). This suggests a dynamic market environment.\n", ticker, sentiment_scores.iter().cloned().fold(0./0., f64::max), sentiment_scores.iter().position(|&s| s == sentiment_scores.iter().cloned().fold(0./0., f64::max)).unwrap() + 1, sentiment_scores.iter().cloned().fold(0./0., f64::min), sentiment_scores.iter().position(|&s| s == sentiment_scores.iter().cloned().fold(0./0., f64::min)).unwrap() + 1);
                    println!("- **3. Reinforcement Learning:** The RL model suggests a mix of buy and hold actions, with higher buying recommendations on certain days (e.g., {:.2} on Day {}) and lower on others (e.g., {:.2} on Day {}). This highlights potential opportunities to adjust your position based on the model's predictions.\n", optimal_actions.iter().cloned().fold(0./0., f64::max), optimal_actions.iter().position(|&a| a == optimal_actions.iter().cloned().fold(0./0., f64::max)).unwrap() + 1, optimal_actions.iter().cloned().fold(0./0., f64::min), optimal_actions.iter().position(|&a| a == optimal_actions.iter().cloned().fold(0./0., f64::min)).unwrap() + 1);

                    // Risk Assessment
                    println!("\n--- Risk Assessment ---\n");
                    println!("Investing in {} carries several risks, including market volatility, economic downturns, and company-specific risks such as changes in management or financial performance. It is essential to consider these risks and diversify your investments to mitigate potential losses.", ticker);

                    // Investment Recommendations
                    println!("\n--- Investment Recommendations ---\n");
                    println!("Based on this analysis, here's a possible investment strategy for the specified period, starting with your initial ${:.2}:", initial_investment);
                    println!("\n- 1. Follow the daily optimal allocation percentages for {} as a baseline strategy.", ticker);
                    println!("\n- 2. Consider increasing your {} allocation on days when sentiment is positive and the RL model recommends buying.", ticker);
                    println!("\n- 3. Be cautious about increasing your position on days with negative sentiment or low RL buying recommendations.");
                    println!("\n- 4. Monitor {}'s performance and broader market trends throughout this period.", ticker);
                    println!("\n- 5. Consult with a financial advisor to tailor this strategy to your risk tolerance and investment goals.\n");

                    // Disclaimer
                    println!("\n--- Disclaimer ---\n");
                    println!("This report is intended for informational purposes only and should not be considered financial advice. Investing in the stock market carries risks, and past performance is not indicative of future results. Always conduct thorough research and consult with a financial professional before making any investment decisions.");

                    Ok(())
                }
                Err(e) => {
                    eprintln!(
                        "Error calculating optimal allocation for ticker {}: {}",
                        ticker, e
                    );
                    return Err(NaluFxError::PortfolioOptimizationError(e.to_string()));
                }
            }
        }
        Err(e) => {
            eprintln!(
                "Historical data not available for ticker {} in the specified date range: {}",
                ticker, e
            );
            println!("Please try a different date range or choose another ticker symbol.");
            return Err(NaluFxError::InvalidOption);
        }
    }
}
