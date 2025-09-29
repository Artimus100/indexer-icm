use crate::models::*;
use crate::utils::convert_coin_to_response;
use poem::{
    Result as PoemResult, handler,
    http::StatusCode,
    web::{Path, Query, Json},
};
use rust_decimal::prelude::ToPrimitive;
use store::UpdateLaunchpadCoin;

#[handler]
pub fn get_trending_coins(
    Query(params): Query<CoinsQuery>,
) -> PoemResult<poem::web::Json<Vec<CoinResponse>>> {
    let limit = params.limit.unwrap_or(50);
    let mut store = store::Store::default();

    match store.get_trending_coins(limit) {
        Ok(coins) => {
            let coin_responses: Vec<CoinResponse> =
                coins.into_iter().map(convert_coin_to_response).collect();
            Ok(poem::web::Json(coin_responses))
        }
        Err(e) => {
            tracing::error!("Failed to fetch trending coins: {:?}", e);
            Err(poem::Error::from_string(
                "Failed to fetch trending coins",
                StatusCode::INTERNAL_SERVER_ERROR,
            ))
        }
    }
}

#[handler]
pub fn get_recent_coins(
    Query(params): Query<CoinsQuery>,
) -> PoemResult<poem::web::Json<Vec<CoinResponse>>> {
    let limit = params.limit.unwrap_or(50);
    let mut store = store::Store::default();

    match store.get_recent_coins(limit) {
        Ok(coins) => {
            let coin_responses: Vec<CoinResponse> =
                coins.into_iter().map(convert_coin_to_response).collect();
            Ok(poem::web::Json(coin_responses))
        }
        Err(e) => {
            tracing::error!("Failed to fetch recent coins: {:?}", e);
            Err(poem::Error::from_string(
                "Failed to fetch recent coins",
                StatusCode::INTERNAL_SERVER_ERROR,
            ))
        }
    }
}

#[handler]
pub fn search_coins(
    Query(params): Query<SearchQuery>,
) -> PoemResult<poem::web::Json<Vec<CoinResponse>>> {
    let limit = params.limit.unwrap_or(50);
    let mut store = store::Store::default();

    match store.search_coins(&params.q, limit) {
        Ok(coins) => {
            let coin_responses: Vec<CoinResponse> =
                coins.into_iter().map(convert_coin_to_response).collect();
            Ok(poem::web::Json(coin_responses))
        }
        Err(e) => {
            tracing::error!("Failed to search coins: {:?}", e);
            Err(poem::Error::from_string(
                "Failed to search coins",
                StatusCode::INTERNAL_SERVER_ERROR,
            ))
        }
    }
}

#[handler]
pub fn get_coins_by_creator(
    Query(params): Query<CreatorQuery>,
) -> PoemResult<poem::web::Json<Vec<CoinResponse>>> {
    let limit = params.limit.unwrap_or(50);
    let mut store = store::Store::default();

    match store.get_coins_by_creator(&params.creator, limit) {
        Ok(coins) => {
            let coin_responses: Vec<CoinResponse> =
                coins.into_iter().map(convert_coin_to_response).collect();
            Ok(poem::web::Json(coin_responses))
        }
        Err(e) => {
            tracing::error!("Failed to fetch coins by creator: {:?}", e);
            Err(poem::Error::from_string(
                "Failed to fetch coins by creator",
                StatusCode::INTERNAL_SERVER_ERROR,
            ))
        }
    }
}

#[handler]
pub fn get_coin_by_mint(
    Path(mint): Path<String>,
) -> PoemResult<poem::web::Json<Option<CoinResponse>>> {
    let mut store = store::Store::default();

    match store.get_launchpad_coin_by_mint(&mint) {
        Ok(Some(coin)) => {
            let coin_response = convert_coin_to_response(coin);
            Ok(poem::web::Json(Some(coin_response)))
        }
        Ok(None) => Ok(poem::web::Json(None)),
        Err(e) => {
            tracing::error!("Failed to fetch coin by mint: {:?}", e);
            Err(poem::Error::from_string(
                "Failed to fetch coin by mint",
                StatusCode::INTERNAL_SERVER_ERROR,
            ))
        }
    }
}

#[handler]
pub fn get_coin_by_pool_id(
    Path(pool_id): Path<String>,
) -> PoemResult<poem::web::Json<Option<CoinResponse>>> {
    let mut store = store::Store::default();

    match store.get_launchpad_coin_by_pool(&pool_id) {
        Ok(Some(coin)) => {
            let coin_response = convert_coin_to_response(coin);
            Ok(poem::web::Json(Some(coin_response)))
        }
        Ok(None) => Ok(poem::web::Json(None)),
        Err(e) => {
            tracing::error!("Failed to fetch coin by pool_id: {:?}", e);
            Err(poem::Error::from_string(
                "Failed to fetch coin by pool_id",
                StatusCode::INTERNAL_SERVER_ERROR,
            ))
        }
    }
}


#[handler]
pub fn get_all_coins(
    Query(params): Query<CoinsQuery>,
) -> PoemResult<poem::web::Json<Vec<CoinResponse>>> {
    let limit = params.limit.unwrap_or(50);
    let page = params.page.unwrap_or(1);
    let offset = (page - 1) * limit;
    let mut store = store::Store::default();

    match store.get_all_coins(offset, limit) {
        Ok(coins) => {
            let coin_responses: Vec<CoinResponse> =
                coins.into_iter().map(convert_coin_to_response).collect();
            Ok(poem::web::Json(coin_responses))
        }
        Err(e) => {
            tracing::error!("Failed to fetch all coins: {:?}", e);
            Err(poem::Error::from_string(
                "Failed to fetch all coins",
                StatusCode::INTERNAL_SERVER_ERROR,
            ))
        }
    }
}

#[handler]
pub fn get_bonding_curve_stats(
    Path(mint): Path<String>,
) -> PoemResult<poem::web::Json<BondingCurveStatsResponse>> {
    let mut store = store::Store::default();

    match store.get_bonding_curve_stats(&mint) {
        Ok(stats) => {
            let response = BondingCurveStatsResponse {
                progress_percentage: stats.progress_percentage.to_f64().unwrap_or(0.0),
                sol_in_curve: stats.sol_in_curve.to_f64().unwrap_or(0.0),
                graduation_target: stats.graduation_target.to_f64().unwrap_or(30.0),
                is_graduated: stats.is_graduated,
            };
            Ok(poem::web::Json(response))
        }
        Err(e) => {
            tracing::error!("Failed to get bonding curve stats: {:?}", e);
            Err(poem::Error::from_string(
                "Failed to get bonding curve stats",
                StatusCode::INTERNAL_SERVER_ERROR,
            ))
        }
    }
}

#[handler]
pub fn update_coin(
    Path(mint): Path<String>,
    Json(request): Json<UpdateCoinRequest>,
) -> PoemResult<poem::web::Json<Option<CoinResponse>>> {
    let mut store = store::Store::default();

    // Create UpdateLaunchpadCoin struct from request
    let updates = UpdateLaunchpadCoin {
        name: request.name,
        symbol: request.symbol,
        description: request.description,
        image_uri: request.image_uri,
        metadata_uri: request.metadata_uri,
        twitter: request.twitter,
        telegram: request.telegram,
        website: request.website,
        complete: request.complete,
        virtual_sol_reserves: request.virtual_sol_reserves,
        virtual_token_reserves: request.virtual_token_reserves,
        total_supply: request.total_supply,
        market_cap: None, // Market cap is calculated, not directly updatable
        usd_market_cap: None, // USD market cap is calculated, not directly updatable
        reply_count: request.reply_count,
        is_banned: request.is_banned,
        program: request.program,
    };

    // Update the coin
    match store.update_launchpad_coin(&mint, updates) {
        Ok(()) => {
            // Fetch and return the updated coin
            match store.get_launchpad_coin_by_mint(&mint) {
                Ok(Some(coin)) => {
                    let coin_response = convert_coin_to_response(coin);
                    Ok(poem::web::Json(Some(coin_response)))
                }
                Ok(None) => {
                    tracing::warn!("Coin updated but not found when fetching: {}", mint);
                    Ok(poem::web::Json(None))
                }
                Err(e) => {
                    tracing::error!("Failed to fetch updated coin: {:?}", e);
                    Err(poem::Error::from_string(
                        "Coin updated but failed to fetch updated data",
                        StatusCode::INTERNAL_SERVER_ERROR,
                    ))
                }
            }
        }
        Err(e) => {
            tracing::error!("Failed to update coin {}: {:?}", mint, e);
            Err(poem::Error::from_string(
                "Failed to update coin",
                StatusCode::INTERNAL_SERVER_ERROR,
            ))
        }
    }
}
