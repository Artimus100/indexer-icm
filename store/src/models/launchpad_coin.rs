use chrono::{DateTime, Utc};
use diesel::prelude::{AsChangeset, Insertable, Queryable};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::schema::*;

#[derive(Queryable, Debug, Clone, Serialize, Deserialize)]
#[diesel(table_name = launchpad_coins)]
pub struct LaunchpadCoin {
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
    pub created_timestamp: DateTime<Utc>,
    pub complete: bool,
    pub virtual_sol_reserves: i64,
    pub virtual_token_reserves: i64,
    pub total_supply: i64,
    pub market_cap: Option<Decimal>,
    pub usd_market_cap: Option<Decimal>,
    pub reply_count: i32,
    pub is_banned: bool,
    pub program: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Insertable, Debug, Clone)]
#[diesel(table_name = launchpad_coins)]
pub struct NewLaunchpadCoin {
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
    pub created_timestamp: DateTime<Utc>,
    pub complete: bool,
    pub virtual_sol_reserves: i64,
    pub virtual_token_reserves: i64,
    pub total_supply: i64,
    pub market_cap: Option<Decimal>,
    pub usd_market_cap: Option<Decimal>,
    pub reply_count: i32,
    pub is_banned: bool,
    pub program: String,
}

#[derive(AsChangeset, Debug, Clone)]
#[diesel(table_name = launchpad_coins)]
pub struct UpdateLaunchpadCoin {
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
    pub market_cap: Option<Decimal>,
    pub usd_market_cap: Option<Decimal>,
    pub reply_count: Option<i32>,
    pub is_banned: Option<bool>,
    pub program: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoinMetadata {
    pub name: String,
    pub symbol: String,
    pub description: Option<String>,
    pub image: Option<String>,
    pub twitter: Option<String>,
    pub telegram: Option<String>,
    pub website: Option<String>,
}

impl LaunchpadCoin {
    /// Calculate the current price per token based on reserves
    pub fn calculate_price_per_token(&self) -> Option<Decimal> {
        if self.virtual_token_reserves == 0 {
            return None;
        }

        let sol_reserves = Decimal::from(self.virtual_sol_reserves);
        let token_reserves = Decimal::from(self.virtual_token_reserves);

        // Convert SOL from lamports (divide by 1e9)
        let sol_in_sol = sol_reserves / Decimal::from(1_000_000_000i64);

        Some(sol_in_sol / token_reserves)
    }

    /// Calculate market cap based on current price and total supply
    pub fn calculate_market_cap(&self) -> Option<Decimal> {
        if let Some(price) = self.calculate_price_per_token() {
            let total_supply = Decimal::from(self.total_supply);
            Some(price * total_supply)
        } else {
            None
        }
    }

    /// Calculate USD market cap (assuming SOL price)
    pub fn calculate_usd_market_cap(&self, sol_price_usd: f64) -> Option<Decimal> {
        if let Some(market_cap_sol) = self.calculate_market_cap() {
            let sol_price = Decimal::try_from(sol_price_usd).ok()?;
            Some(market_cap_sol * sol_price)
        } else {
            None
        }
    }

    /// Calculate bonding curve progress (0-100%)
    pub fn calculate_bonding_curve_progress(&self) -> Option<Decimal> {
        if self.total_supply == 0 {
            return None;
        }

        let tokens_sold = self.total_supply - self.virtual_token_reserves;
        let progress =
            (Decimal::from(tokens_sold) / Decimal::from(self.total_supply)) * Decimal::from(100);
        Some(progress)
    }
}
