use crate::schema::holders;
use chrono::{DateTime, Utc};
use diesel::prelude::{AsChangeset, Insertable, Queryable};
use rust_decimal::Decimal;

#[derive(Queryable, Insertable, Debug, Clone, serde::Serialize, serde::Deserialize)]
#[diesel(table_name = holders)]
pub struct Holder {
    pub id: i32,
    pub mint: String,
    pub wallet_address: String,
    pub amount: Decimal,
    pub sol_balance: Decimal,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Insertable, Debug, Clone)]
#[diesel(table_name = holders)]
pub struct NewHolder {
    pub mint: String,
    pub wallet_address: String,
    pub amount: Decimal,
    pub sol_balance: Decimal,
}

#[derive(AsChangeset, Debug, Default)]
#[diesel(table_name = holders)]
pub struct UpdateHolder {
    pub amount: Option<Decimal>,
    pub sol_balance: Option<Decimal>,
    pub updated_at: Option<DateTime<Utc>>,
}
