use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct CoinResponse {
    pub id: i32,
    pub mint: String,
    pub name: String,
    pub symbol: String,
    pub description: Option<String>,
    pub image_uri: Option<String>,
    pub metadata_uri: Option<String>,
    pub twitter: Option<String>,
    pub telegram: Option<String>,
    pub website: Option<String>,
    pub pool_address: String,
    pub creator: String,
    pub created_timestamp: String,
    pub complete: bool,
    pub virtual_sol_reserves: i64,
    pub virtual_token_reserves: i64,
    pub total_supply: i64,
    pub market_cap: Option<String>,
    pub usd_market_cap: Option<String>,
    pub reply_count: i32,
    pub is_banned: bool,
    pub program: String,
    pub created_at: String,
    pub updated_at: String,
    // Calculated fields
    pub price_per_token: Option<String>,
    pub bonding_curve_progress: Option<String>,
    pub bonding_curve_stats: Option<BondingCurveStatsResponse>,
}

#[derive(Deserialize)]
pub struct CoinsQuery {
    pub limit: Option<i64>,
    pub page: Option<i64>,
}

#[derive(Deserialize)]
pub struct SearchQuery {
    pub q: String,
    pub limit: Option<i64>,
}

#[derive(Serialize)]
pub struct TopHolder {
    pub address: String,
    pub amount: f64,
    pub sol_balance: f64,
}

#[derive(Serialize)]
pub struct TopHoldersResponse {
    pub top_holders: Vec<TopHolder>,
    pub total_holders: Vec<String>,
}

#[derive(Serialize, Deserialize)]
pub struct BondingCurveStatsResponse {
    pub progress_percentage: f64,
    pub sol_in_curve: f64,
    pub graduation_target: f64,
    pub is_graduated: bool,
}

#[derive(Deserialize)]
pub struct CreatorQuery {
    pub creator: String,
    pub limit: Option<i64>,
}

#[derive(Deserialize)]
pub struct UpdateCoinRequest {
    pub name: Option<String>,
    pub symbol: Option<String>,
    pub description: Option<String>,
    pub image_uri: Option<String>,
    pub metadata_uri: Option<String>,
    pub twitter: Option<String>,
    pub telegram: Option<String>,
    pub website: Option<String>,
    pub complete: Option<bool>,
    pub virtual_sol_reserves: Option<i64>,
    pub virtual_token_reserves: Option<i64>,
    pub total_supply: Option<i64>,
    pub reply_count: Option<i32>,
    pub is_banned: Option<bool>,
    pub program: Option<String>,
}
