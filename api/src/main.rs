use dotenvy::dotenv;
use poem::{
    EndpointExt, Route, Server, get, put,
    http::Method,
    listener::TcpListener,
    middleware::{Cors, Tracing},
};

mod handlers;
mod models;
mod utils;

use handlers::*;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    tracing_subscriber::fmt::init();

    dotenv().ok();

    // Configure CORS
    let cors = Cors::new()
        .allow_origin("http://localhost:3000")
        .allow_methods(vec![
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers(vec!["Content-Type", "Authorization"])
        .allow_credentials(true);

    let app = Route::new()
        .at("/v1/health", get(health_check))
        .at("/api/v1/coins/trending", get(get_trending_coins))
        .at("/api/v1/coins/recent", get(get_recent_coins))
        .at("/api/v1/coins/search", get(search_coins))
        .at("/api/v1/coins/creator", get(get_coins_by_creator))
        .at("/api/v1/coins/mint/:mint", get(get_coin_by_mint))
        .at("/api/v1/coins/pool/:pool_id", get(get_coin_by_pool_id))
        .at("/api/v1/coins/:mint/holders", get(get_top_holders))
        .at(
            "/api/v1/coins/:mint/bonding-curve",
            get(get_bonding_curve_stats),
        )
        .at("/api/v1/coins/:mint", put(update_coin))
        .at("/api/v1/coins", get(get_all_coins))
        .at("/api/v1/klines/:mint", get(get_klines))
        .at("/api/v1/trades/:mint", get(get_recent_trades))
        .at("/api/v1/trades/id/:id", get(get_trade_by_id))
        .with(Tracing)
        .with(cors);

    Server::new(TcpListener::bind("0.0.0.0:3001"))
        .name("pumpfun-clone-api")
        .run(app)
        .await
}
