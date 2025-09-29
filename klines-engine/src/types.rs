use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum MessageFromEngine {
    #[serde(rename = "TRADE_ADDED")]
    AddTrade { 
        #[allow(dead_code)]
        data: AddTradePayload 
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct AddTradePayload {
    #[allow(dead_code)]
    pub ticker: String,
    #[allow(dead_code)]
    pub pool_id: String,
    #[allow(dead_code)]
    pub time: DateTime<Utc>,
    #[allow(dead_code)]
    pub quantity: Decimal,
    #[allow(dead_code)]
    pub price: Decimal
}