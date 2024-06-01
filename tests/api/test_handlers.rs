#[cfg(test)]
mod tests {
    use actix_web::{http, test, App};
    use dotenv::dotenv;
    use nalufx::api::handlers::predict_cash_flow;
    use serde_json::json;

    #[actix_rt::test]
    async fn test_predict_cash_flow_empty_historical_data() {
        // Load environment variables from the .env file
        dotenv().ok();
        // Initialize the application
        let mut app = test::init_service(App::new().service(predict_cash_flow)).await;
        // Create a test request with empty historical data
        let req = test::TestRequest::post()
            .uri("/predict")
            .set_json(&json!({
                "historical_data": Vec::<f64>::new(),
                "daily_returns": vec![0.01, 0.02, 0.03, 0.04, 0.05, 0.06],
                "cash_flows": vec![1000.0, 2000.0, 3000.0, 4000.0, 5000.0, 6000.0]
            }))
            .to_request();
        // Send the request and get the response
        let resp = test::call_service(&mut app, req).await;
        let status = resp.status();
        // Check if the response has a bad request status
        assert_eq!(
            status,
            http::StatusCode::BAD_REQUEST,
            "Response status is not 400 Bad Request"
        );
    }
}
