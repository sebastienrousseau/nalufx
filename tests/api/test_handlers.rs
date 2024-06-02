#[cfg(test)]
mod tests {
    use actix_web::{test, web, App, HttpResponse, Responder};
    use nalufx::{
        api::handlers::{get_openai_api_key, parse_openai_response, send_openai_request},
        models::{CashFlowRequest, CashFlowResponse, ErrorResponse},
    };
    use reqwest::Client;
    use serde_json::json;
    use std::env;
    use wiremock::{
        matchers::{header, method, path},
        Mock, MockServer, ResponseTemplate,
    };

    // Mock predict_cash_flow handler
    async fn mock_predict_cash_flow(
        data: web::Json<CashFlowRequest>,
        _client: web::Data<Client>,
        _api_key: web::Data<String>,
    ) -> impl Responder {
        if data.historical_data.is_empty() {
            return HttpResponse::BadRequest().json(ErrorResponse {
                error: "Invalid historical data".to_string(),
            });
        }

        let predictions = vec![0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
        let optimal_allocation = vec![0.5, 0.3, 0.2];

        HttpResponse::Ok().json(CashFlowResponse {
            predictions,
            optimal_allocation,
        })
    }

    /// Tests the `predict_cash_flow` handler with valid input.
    #[actix_rt::test]
    async fn test_predict_cash_flow_valid() {
        let request = CashFlowRequest {
            historical_data: vec![1.0, 2.0, 3.0],
        };

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(Client::new()))
                .app_data(web::Data::new("test_api_key".to_string()))
                .service(
                    web::scope("/api").route("/predict", web::post().to(mock_predict_cash_flow)),
                ),
        )
        .await;

        let req = test::TestRequest::post()
            .uri("/api/predict")
            .set_json(&request)
            .to_request();
        let resp: CashFlowResponse = test::call_and_read_body_json(&app, req).await;

        assert_eq!(resp.predictions, vec![0.0, 0.0, 0.0, 0.0, 0.0, 0.0]);
        assert_eq!(resp.optimal_allocation, vec![0.5, 0.3, 0.2]);
    }

    /// Tests the `predict_cash_flow` handler with empty historical data.
    #[actix_rt::test]
    async fn test_predict_cash_flow_empty_data() {
        let request = CashFlowRequest {
            historical_data: vec![],
        };

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(Client::new()))
                .app_data(web::Data::new("test_api_key".to_string()))
                .service(
                    web::scope("/api").route("/predict", web::post().to(mock_predict_cash_flow)),
                ),
        )
        .await;

        let req = test::TestRequest::post()
            .uri("/api/predict")
            .set_json(&request)
            .to_request();
        let resp: ErrorResponse = test::call_and_read_body_json(&app, req).await;

        assert_eq!(resp.error, "Invalid historical data");
    }

    /// Tests the `predict_cash_flow` handler with negative historical data.
    #[actix_rt::test]
    async fn test_predict_cash_flow_negative_data() {
        let request = CashFlowRequest {
            historical_data: vec![-1.0, -2.0, -3.0],
        };

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(Client::new()))
                .app_data(web::Data::new("test_api_key".to_string()))
                .service(
                    web::scope("/api").route("/predict", web::post().to(mock_predict_cash_flow)),
                ),
        )
        .await;

        let req = test::TestRequest::post()
            .uri("/api/predict")
            .set_json(&request)
            .to_request();
        let resp: CashFlowResponse = test::call_and_read_body_json(&app, req).await;

        assert_eq!(resp.predictions, vec![0.0, 0.0, 0.0, 0.0, 0.0, 0.0]);
        assert_eq!(resp.optimal_allocation, vec![0.5, 0.3, 0.2]);
    }

    /// Tests the `predict_cash_flow` handler with zero values in historical data.
    #[actix_rt::test]
    async fn test_predict_cash_flow_zero_data() {
        let request = CashFlowRequest {
            historical_data: vec![0.0, 0.0, 0.0],
        };

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(Client::new()))
                .app_data(web::Data::new("test_api_key".to_string()))
                .service(
                    web::scope("/api").route("/predict", web::post().to(mock_predict_cash_flow)),
                ),
        )
        .await;

        let req = test::TestRequest::post()
            .uri("/api/predict")
            .set_json(&request)
            .to_request();
        let resp: CashFlowResponse = test::call_and_read_body_json(&app, req).await;

        assert_eq!(resp.predictions, vec![0.0, 0.0, 0.0, 0.0, 0.0, 0.0]);
        assert_eq!(resp.optimal_allocation, vec![0.5, 0.3, 0.2]);
    }

    /// Tests fetching the OpenAI API key from the environment.
    #[actix_rt::test]
    async fn test_get_openai_api_key() {
        env::set_var("OPENAI_API_KEY", "test_api_key");
        let api_key = get_openai_api_key().unwrap();
        assert_eq!(api_key, "test_api_key");
        env::remove_var("OPENAI_API_KEY");
    }

    /// Tests sending a request to the OpenAI API.
    #[actix_rt::test]
    async fn test_send_openai_request() {
        let mock_server = MockServer::start().await;

        let mock = Mock::given(method("POST"))
            .and(path("/v1/chat/completions"))
            .and(header("Authorization", "Bearer test_api_key"))
            .and(header("Content-Type", "application/json"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "choices": [
                    {
                        "message": {
                            "content": "1.0 2.0 3.0"
                        }
                    }
                ]
            })));

        mock.mount(&mock_server).await;

        let client = Client::new();
        let api_key = "test_api_key";
        let request_body = json!({
            "model": "gpt-3.5-turbo",
            "messages": [
                {"role": "system", "content": "You are a highly skilled financial assistant."},
                {"role": "user", "content": "Predict the cash flow."}
            ],
            "max_tokens": 100,
        });

        let openai_url = format!("{}/v1/chat/completions", mock_server.uri());
        let response = send_openai_request(&client, &openai_url, api_key, request_body).await;
        assert!(response.is_ok());

        // Verify that the mock received the expected request
        mock_server.verify().await
    }

    /// Tests fetching the OpenAI API key when it is not set.
    #[actix_rt::test]
    async fn test_get_openai_api_key_not_set() {
        env::remove_var("OPENAI_API_KEY");
        let result = get_openai_api_key();
        assert_eq!(result, Err("OPENAI_API_KEY is not set"));
    }

    /// Tests handling a non-successful HTTP response from the OpenAI API.
    #[actix_rt::test]
    async fn test_send_openai_request_http_error() {
        let mock_server = MockServer::start().await;

        let mock = Mock::given(method("POST"))
            .and(path("/v1/chat/completions"))
            .and(header("Authorization", "Bearer test_api_key"))
            .and(header("Content-Type", "application/json"))
            .respond_with(ResponseTemplate::new(500))
            .expect(1);

        mock.mount(&mock_server).await;

        let client = Client::new();
        let api_key = "test_api_key";
        let request_body = json!({
            "model": "gpt-3.5-turbo",
            "messages": [
                {"role": "system", "content": "You are a highly skilled financial assistant."},
                {"role": "user", "content": "Predict the cash flow."}
            ],
            "max_tokens": 100,
        });

        let openai_url = format!("{}/v1/chat/completions", mock_server.uri());
        let response = send_openai_request(&client, &openai_url, api_key, request_body).await;
        assert!(response.is_err());
        assert_eq!(response, Err("OpenAI API call failed"));

        // Verify that the mock received the expected request
        mock_server.verify().await;
    }

    /// Tests parsing the OpenAI API response.
    #[actix_rt::test]
    async fn test_parse_openai_response() {
        let response_body = r#"
        {
            "choices": [
                {
                    "message": {
                        "content": "1.0 2.0 3.0"
                    }
                }
            ]
        }
        "#;

        let predictions = parse_openai_response(response_body).unwrap();
        assert_eq!(predictions, vec![1.0, 2.0, 3.0]);
    }

    /// Tests handling an unexpected response structure from the OpenAI API.
    #[actix_rt::test]
    async fn test_parse_openai_response_unexpected_structure() {
        let response_body = r#"
        {
            "unexpected_field": "unexpected_value"
        }
        "#;

        let result = parse_openai_response(response_body);
        assert!(result.is_err());
    }
}
