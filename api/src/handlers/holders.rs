use crate::models::*;
use poem::{
    Result as PoemResult, handler,
    http::StatusCode,
    web::{Path, Query},
};
use rust_decimal::prelude::ToPrimitive;

#[handler]
pub fn get_top_holders(
    Path(mint): Path<String>,
    Query(params): Query<CoinsQuery>,
) -> PoemResult<poem::web::Json<TopHoldersResponse>> {
    let limit = params.limit.unwrap_or(50).min(100);
    let mut store = store::Store::default();

    match store.get_top_holders(&mint, limit) {
        Ok(holders) => {
            let top_holders: Vec<TopHolder> = holders
                .iter()
                .map(|holder| TopHolder {
                    address: holder.wallet_address.clone(),
                    amount: holder.amount.to_f64().unwrap_or(0.0),
                    sol_balance: holder.sol_balance.to_f64().unwrap_or(0.0),
                })
                .collect();

            // Get all holder addresses for total_holders
            let all_holders = match store.get_all_holders(&mint) {
                Ok(holders) => holders.into_iter().map(|h| h.wallet_address).collect(),
                Err(_) => top_holders.iter().map(|h| h.address.clone()).collect(),
            };

            let response = TopHoldersResponse {
                top_holders,
                total_holders: all_holders,
            };

            Ok(poem::web::Json(response))
        }
        Err(e) => {
            tracing::error!("Failed to fetch top holders: {:?}", e);
            Err(poem::Error::from_string(
                "Failed to fetch top holders",
                StatusCode::INTERNAL_SERVER_ERROR,
            ))
        }
    }
}
