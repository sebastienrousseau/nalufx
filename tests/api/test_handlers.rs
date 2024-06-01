#[cfg(test)]
mod tests {
    use actix_web::{http, test, App};
    use dotenv::dotenv;
    use nalufx::api::handlers::predict_cash_flow;
    use nalufx::api::models::CashFlowResponse;
    use serde_json::json;

    #[actix_rt::test]
    async fn test_predict_cash_flow_success() {
        // Load environment variables from the .env file
        dotenv().ok();
        // Initialize the application
        let mut app = test::init_service(App::new().service(predict_cash_flow)).await;
        // Create a test request
        let req = test::TestRequest::post()
            .uri("/predict")
            .set_json(&json!({
                "historical_data": vec![100.0, 101.0, 102.0, 103.0]
            }))
            .to_request();
        // Send the request and get the response
        let resp = test::call_service(&mut app, req).await;
        let status = resp.status(); // Capture the status before moving `resp`
                                    // Check if the response was successful
        assert_eq!(status, http::StatusCode::OK);

        // Parse the response body
        let body = test::read_body(resp).await;
        let response: CashFlowResponse = serde_json::from_slice(&body).unwrap();
        println!("Response body: {:?}", response);

        // Check if the predictions array has the expected length of 6
        assert_eq!(
            response.predictions.len(),
            6,
            "Predictions array length is not 6"
        );
        assert_eq!(
            response.optimal_allocation.len(),
            6,
            "Optimal allocation array length is not 6"
        );

        // Check if the predictions values are as expected (optional)
        assert_eq!(
            response.predictions,
            vec![0.0; 6],
            "Predictions values are incorrect"
        );
        assert_eq!(
            response.optimal_allocation,
            vec![0.0; 6],
            "Optimal allocation values are incorrect"
        );
    }

    #[actix_rt::test]
    async fn test_predict_cash_flow_invalid_request() {
        // Load environment variables from the .env file
        dotenv().ok();
        // Initialize the application
        let mut app = test::init_service(App::new().service(predict_cash_flow)).await;
        // Create a test request with invalid JSON
        let req = test::TestRequest::post()
            .uri("/predict")
            .set_json(&json!({
                "invalid_field": "invalid_value"
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
                "historical_data": Vec::<f64>::new()
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
