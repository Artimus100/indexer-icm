use anyhow::{anyhow, Result};
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use sqlx::{Executor, PgPool};
use tracing::{info, warn};

use crate::types::AddTradePayload;

pub async fn ensure_trading_tables(pool: &PgPool) -> Result<()> {
    // 1. Create raw trades table (one table for all tokens) - only if it doesn't exist
    let create_trades_table_sql = r#"
        CREATE TABLE IF NOT EXISTS public.trades (
            id SERIAL,
            base_mint VARCHAR NOT NULL,
            pool_id VARCHAR,
            timestamp TIMESTAMPTZ NOT NULL,
            price_raw BIGINT NOT NULL,
            quantity_raw BIGINT NOT NULL,
            volume_raw BIGINT NOT NULL,
            price_decimal NUMERIC(20, 9),
            quantity_decimal NUMERIC(20, 9),
            volume_decimal NUMERIC(20, 9)
        );
    "#;
    
    pool.execute(create_trades_table_sql).await?;
    info!("Ensured raw trades table exists.");

    // Note: TimescaleDB hypertable functionality removed for compatibility with standard PostgreSQL
    info!("Raw trades table created (standard PostgreSQL table).");

    // Create indexes on raw trades (include timestamp for TimescaleDB)
    let create_trades_index_sql = "CREATE INDEX IF NOT EXISTS idx_trades_base_mint_timestamp ON public.trades (base_mint, timestamp DESC);";
    pool.execute(create_trades_index_sql).await?;
    
    let create_trades_pool_index_sql = "CREATE INDEX IF NOT EXISTS idx_trades_pool_id_timestamp ON public.trades (pool_id, timestamp DESC);";
    pool.execute(create_trades_pool_index_sql).await?;
    
    info!("Created indexes on raw trades table.");

    // 2. Create OHLCV materialized views for basic time intervals (compatible with standard PostgreSQL)
    let intervals = [
        ("1m", "minute"),
        ("1h", "hour"),
        ("1d", "day"),
    ];

    for (interval_label, date_trunc_unit) in intervals.iter() {
        let mv_name = format!("ohlcv_{}", interval_label);
        
        // Create materialized view that aggregates from raw trades using standard PostgreSQL
        let create_mv_sql = format!(
            r#"
            CREATE MATERIALIZED VIEW IF NOT EXISTS public.{} AS
            SELECT 
                pool_id,
                date_trunc('{}', timestamp) AS bucket,
                (array_agg(price_decimal ORDER BY timestamp))[1] AS open,
                max(price_decimal) AS high,
                min(price_decimal) AS low,
                (array_agg(price_decimal ORDER BY timestamp DESC))[1] AS close,
                sum(volume_decimal) AS volume,
                count(*) AS trade_count
            FROM public.trades
            WHERE pool_id IS NOT NULL
            GROUP BY pool_id, date_trunc('{}', timestamp)
            WITH DATA;
            "#,
            mv_name, date_trunc_unit, date_trunc_unit
        );
        
        pool.execute(create_mv_sql.as_str()).await?;
        info!("Created materialized view public.{}", mv_name);

        // Create index for efficient queries (non-unique to avoid TimescaleDB constraints)
        let create_index_sql = format!(
            "CREATE INDEX IF NOT EXISTS idx_{}_pool_id_bucket ON public.{} (pool_id, bucket DESC);",
            mv_name, mv_name
        );
        pool.execute(create_index_sql.as_str()).await?;
        info!("Created index on public.{}", mv_name);
    }

    Ok(())
}

#[allow(dead_code)]
pub async fn process_trade(
    pool: &PgPool,
    trade_payload: &AddTradePayload,
) -> Result<()> {
    // No need to ensure tables on every trade - they're created at startup

    // Convert Decimal to raw integer values (lamports/token units)
    let price_raw = (trade_payload.price * Decimal::from(1_000_000_000i64))
        .to_i64()
        .unwrap_or(0);
    let quantity_raw = (trade_payload.quantity * Decimal::from(1_000_000_000i64))
        .to_i64()
        .unwrap_or(0);
    let volume_raw = price_raw * quantity_raw / 1_000_000_000i64; // Volume in lamports

    // Keep decimal values for display purposes
    let price_decimal = trade_payload.price;
    let quantity_decimal = trade_payload.quantity;
    let volume_decimal = price_decimal * quantity_decimal;

    // Insert raw trade data (this is the only write operation)
    let insert_trade_sql = r#"
        INSERT INTO public.trades (base_mint, pool_id, timestamp, price_raw, quantity_raw, volume_raw, price_decimal, quantity_decimal, volume_decimal)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
    "#;

    sqlx::query(insert_trade_sql)
        .bind(&trade_payload.ticker) // base_mint
        .bind(&trade_payload.pool_id) // pool_id
        .bind(trade_payload.time)
        .bind(price_raw)
        .bind(quantity_raw)
        .bind(volume_raw)
        .bind(price_decimal)
        .bind(quantity_decimal)
        .bind(volume_decimal)
        .execute(pool)
        .await?;

    info!(
        "Inserted trade for base_mint '{}' at price {} (raw: {}) with volume {} (raw: {})",
        trade_payload.ticker, price_decimal, price_raw, volume_decimal, volume_raw
    );
    Ok(())
}