use actix_web::{App, HttpServer};
use dotenvy::dotenv;
use env_logger;
use nalufx::api::handlers::predict_cash_flow;
use nalufx::config::Config;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok(); // Load environment variables from .env file
    env_logger::init();

    let config = Config::from_env().expect("Failed to load configuration");

    HttpServer::new(|| App::new().service(predict_cash_flow))
        .bind(config.server_addr)?
        .run()
        .await
}
