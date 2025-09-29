use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

// Client request types
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct SubscribePayload {
    pub room: String,
    pub data: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct UnsubscribePayload {
    pub room: String,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "UPPERCASE")]
pub enum ClientRequest {
    #[serde(rename = "SUBSCRIBE")]
    Subscribe { room: String, data: Option<Vec<String>> },
    #[serde(rename = "UNSUBSCRIBE")]
    Unsubscribe { room: String },
}

// Server message types for trading data
#[derive(Debug, Serialize, Clone)]
pub struct TokenTradingData {
    pub token_id: String,
    pub symbol: String,
    pub name: String,
    pub current_price: Option<Decimal>,
    pub market_cap: Option<Decimal>,
    pub volume_24h: Option<Decimal>,
    pub volume_1h: Option<Decimal>,
    pub price_change_24h: Option<Decimal>,
    pub price_change_1h: Option<Decimal>,
    pub holders_count: Option<i32>,
    pub trades_count: Option<i32>,
    pub bonding_curve_progress: Option<Decimal>,
    pub sol_in_bonding_curve: Option<Decimal>,
    pub amount_to_graduate: Option<Decimal>,
    pub last_updated: Option<DateTime<Utc>>,
    pub base_reserve: u64,
    pub quote_reserve: u64,
    pub sqrt_price: u64,
    // Additional pool information
    pub base_mint: Option<String>,
    pub quote_mint: Option<String>,
    pub creator: Option<String>,
    pub uri: Option<String>,
    pub base_vault: Option<String>,
    pub quote_vault: Option<String>,
}

#[derive(Debug, Serialize, Clone)]
pub struct PriceHistoryData {
    pub token_id: String,
    #[serde(serialize_with = "serialize_datetime_as_timestamp")]
    pub time: DateTime<Utc>,
    pub price: Option<Decimal>,
    pub market_cap: Option<Decimal>,
    pub volume: Option<Decimal>,
}

#[derive(Debug, Serialize, Clone)]
pub struct TradeData {
    pub token_id: String,
    #[serde(serialize_with = "serialize_datetime_as_timestamp")]
    pub time: DateTime<Utc>,
    pub user_address: String,
    pub amount_in: i64,
    pub amount_out: i64,
    pub price_at_trade: Option<Decimal>,
    pub trade_type: Option<String>,
    pub tx_signature: Option<String>,
}

fn serialize_datetime_as_timestamp<S>(dt: &DateTime<Utc>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_i64(dt.timestamp())
}

// New message types for different subscription types
#[derive(Debug, Serialize, Clone)]
pub struct LaunchCoinData {
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
    pub market_cap: Option<f64>,
    pub usd_market_cap: Option<f64>,
    pub reply_count: i32,
    pub is_banned: bool,
    pub program: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct KlineData {
    pub close: String,
    pub end: String,
    pub high: String,
    pub low: String,
    pub open: String,
    pub quote_volume: String,
    pub start: String,
    pub trades: String,
    pub volume: String,
}

#[derive(Debug, Serialize, Clone)]
#[serde(tag = "type", rename_all = "UPPERCASE")]
pub enum ServerMessage {
    // Legacy messages (keeping for backward compatibility)
    #[serde(rename = "POOL_UPDATES")]
    PoolUpdates { data: TokenTradingData },
    #[serde(rename = "PRICE_HISTORY")]
    PriceHistory { data: PriceHistoryData },
    #[serde(rename = "TRADE_DATA")]
    TradeData { data: TradeData },
    #[serde(rename = "TOKEN_CREATED")]
    TokenCreated { data: TokenTradingData },
    #[serde(rename = "METADATA_CREATED")]
    MetadataCreated {
        token_id: String,
        pool_creator: String,
        partner: String,
    },
    #[serde(rename = "MIGRATION_METADATA_CREATED")]
    MigrationMetadataCreated {
        token_id: String,
        config: String,
        migration_metadata: String,
        payer: String,
    },
    #[serde(rename = "LOCKER_CREATED")]
    LockerCreated {
        token_id: String,
        config: String,
        base_mint: String,
        creator: String,
        escrow: String,
        escrow_token: String,
    },
    #[serde(rename = "MIGRATION_DAMM_V2")]
    MigrationDammV2 {
        token_id: String,
        config: String,
        pool: String,
        base_mint: String,
        quote_mint: String,
        payer: String,
        first_position_nft_mint: String,
        second_position_nft_mint: Option<String>,
    },
    
    // New room-based messages
    #[serde(rename = "LAUNCH_COIN")]
    LaunchCoin { data: LaunchCoinData },
    #[serde(rename = "TRADE")]
    Trade { data: TradeData },
    #[serde(rename = "KLINE")]
    Kline { data: KlineData },
    
    // System messages
    #[serde(rename = "ERROR")]
    Error { message: String },
    #[serde(rename = "SUBSCRIPTION_CONFIRMED")]
    SubscriptionConfirmed { room: String },
    #[serde(rename = "UNSUBSCRIPTION_CONFIRMED")]
    UnsubscriptionConfirmed { room: String },
}

// MeteoraEvent types (copied from db-processor for Redis stream processing)
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "event_type")]
pub enum MeteoraEvent {
    #[serde(rename = "pool_created")]
    PoolCreated {
        timestamp: u64,
        pool_id: String,
        creator: String,
        base_mint: String,
        quote_mint: String,
        name: String,
        symbol: String,
        uri: String,
        base_vault: String,
        quote_vault: String,
    },
    #[serde(rename = "swap")]
    Swap {
        timestamp: u64,
        pool_id: String,
        user: String,
        amount_in: u64,
        minimum_amount_out: u64,
        input_token_account: String,
        output_token_account: String,
        base_mint: String,
        quote_mint: String,
    },
    #[serde(rename = "metadata_created")]
    MetadataCreated {
        timestamp: u64,
        virtual_pool: String,
        pool_creator: String,
        partner: String,
    },
    #[serde(rename = "migration_metadata_created")]
    MigrationMetadataCreated {
        timestamp: u64,
        virtual_pool: String,
        config: String,
        migration_metadata: String,
        payer: String,
    },
    #[serde(rename = "locker_created")]
    LockerCreated {
        timestamp: u64,
        virtual_pool: String,
        config: String,
        base_mint: String,
        creator: String,
        escrow: String,
        escrow_token: String,
    },
    #[serde(rename = "migration_damm_v2")]
    MigrationDammV2 {
        timestamp: u64,
        virtual_pool: String,
        config: String,
        pool: String,
        base_mint: String,
        quote_mint: String,
        payer: String,
        first_position_nft_mint: String,
        second_position_nft_mint: Option<String>,
    },
}
