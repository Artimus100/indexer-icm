use poem::{
    Result as PoemResult, handler,
    http::StatusCode,
    web::{Path, Query},
};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};
use std::env;

#[derive(Serialize, Deserialize)]
pub struct Trade {
    pub id: i32,
    pub pool_id: String,
    pub timestamp: String,
    pub price: String,
    pub quantity: String,
    pub volume: String,
}

#[derive(Deserialize)]
pub struct TradesQuery {
    pub limit: Option<u32>,
    pub offset: Option<u32>,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
}

#[handler]
pub async fn get_recent_trades(
    Path(mint): Path<String>,
    Query(params): Query<TradesQuery>,
) -> PoemResult<poem::web::Json<Vec<Trade>>> {
    let limit = params.limit.unwrap_or(100).min(1000); // Cap at 1000
    let offset = params.offset.unwrap_or(0);
    
    let database_url = env::var("DATABASE_URL")
        .map_err(|_| poem::Error::from_string("DATABASE_URL not set", StatusCode::INTERNAL_SERVER_ERROR))?;
    
    let pool = PgPool::connect(&database_url)
        .await
        .map_err(|e| poem::Error::from_string(format!("Database connection failed: {}", e), StatusCode::INTERNAL_SERVER_ERROR))?;

    // Build the base query
    let mut sql = r#"
        SELECT id, pool_id, timestamp, price_decimal as price, quantity_decimal as quantity, volume_decimal as volume
        FROM public.trades
        WHERE pool_id = $1
    "#.to_string();
    
    let mut param_count = 1;
    let mut params_vec: Vec<Box<dyn sqlx::Encode<'_, sqlx::Postgres> + Send + Sync>> = vec![Box::new(mint.clone())];

    // Add time filters if provided
    if let Some(start_time) = &params.start_time {
        param_count += 1;
        sql.push_str(&format!(" AND timestamp >= to_timestamp(${})", param_count));
        let start_timestamp = start_time.parse::<i64>().map_err(|e| {
            tracing::error!("Failed to parse start_time '{}': {}", start_time, e);
            poem::Error::from_string("Invalid start_time format", StatusCode::BAD_REQUEST)
        })?;
        
        // Convert to seconds if they're in milliseconds
        let start_seconds = if start_timestamp > 1_000_000_000_000 {
            start_timestamp / 1000
        } else {
            start_timestamp
        };
        params_vec.push(Box::new(start_seconds));
    }

    if let Some(end_time) = &params.end_time {
        param_count += 1;
        sql.push_str(&format!(" AND timestamp <= to_timestamp(${})", param_count));
        let end_timestamp = end_time.parse::<i64>().map_err(|e| {
            tracing::error!("Failed to parse end_time '{}': {}", end_time, e);
            poem::Error::from_string("Invalid end_time format", StatusCode::BAD_REQUEST)
        })?;
        
        // Convert to seconds if they're in milliseconds
        let end_seconds = if end_timestamp > 1_000_000_000_000 {
            end_timestamp / 1000
        } else {
            end_timestamp
        };
        params_vec.push(Box::new(end_seconds));
    }

    // Add ordering and pagination
    param_count += 1;
    sql.push_str(&format!(" ORDER BY timestamp DESC LIMIT ${} OFFSET ${}", param_count, param_count + 1));
    params_vec.push(Box::new(limit as i64));
    params_vec.push(Box::new(offset as i64));
    
    tracing::info!("Query params - mint: {}, limit: {}, offset: {}", mint, limit, offset);

    let rows = sqlx::query(&sql)
        .bind(&mint)
        .bind(limit as i64)
        .bind(offset as i64)
        .fetch_all(&pool)
        .await;

    match rows {
        Ok(results) => {
            tracing::info!("Query returned {} trades", results.len());
            
            let trades: Result<Vec<Trade>, _> = results
                .into_iter()
                .map(|row| -> Result<Trade, Box<dyn std::error::Error + Send + Sync>> {
                    Ok(Trade {
                        id: row.try_get::<i32, _>("id")?,
                        pool_id: row.try_get::<String, _>("pool_id")?,
                        timestamp: row.try_get::<chrono::DateTime<chrono::Utc>, _>("timestamp")?
                            .timestamp_millis()
                            .to_string(),
                        price: row.try_get::<Decimal, _>("price")?.to_string(),
                        quantity: row.try_get::<Decimal, _>("quantity")?.to_string(),
                        volume: row.try_get::<Decimal, _>("volume")?.to_string(),
                    })
                })
                .collect();

            match trades {
                Ok(trades) => {
                    tracing::info!("Successfully converted {} trades", trades.len());
                    Ok(poem::web::Json(trades))
                }
                Err(e) => {
                    tracing::error!("Error converting row data: {}", e);
                    Err(poem::Error::from_string(
                        format!("Error processing query results: {}", e),
                        StatusCode::INTERNAL_SERVER_ERROR,
                    ))
                }
            }
        }
        Err(e) => {
            tracing::error!("DB error fetching trades: {:?}", e);
            Err(poem::Error::from_string(
                format!("Database query failed: {}", e),
                StatusCode::INTERNAL_SERVER_ERROR,
            ))
        }
    }
}

#[handler]
pub async fn get_trade_by_id(
    Path(id): Path<i32>,
) -> PoemResult<poem::web::Json<Option<Trade>>> {
    let database_url = env::var("DATABASE_URL")
        .map_err(|_| poem::Error::from_string("DATABASE_URL not set", StatusCode::INTERNAL_SERVER_ERROR))?;
    
    let pool = PgPool::connect(&database_url)
        .await
        .map_err(|e| poem::Error::from_string(format!("Database connection failed: {}", e), StatusCode::INTERNAL_SERVER_ERROR))?;

    let sql = r#"
        SELECT id, pool_id, timestamp, price_decimal as price, quantity_decimal as quantity, volume_decimal as volume
        FROM public.trades
        WHERE id = $1
    "#;

    let row = sqlx::query(sql)
        .bind(id)
        .fetch_optional(&pool)
        .await
        .map_err(|e| poem::Error::from_string(format!("Database query failed: {}", e), StatusCode::INTERNAL_SERVER_ERROR))?;

    match row {
        Some(row) => {
            let trade = Trade {
                id: row.try_get::<i32, _>("id").map_err(|e| {
                    poem::Error::from_string(format!("Error processing query results: {}", e), StatusCode::INTERNAL_SERVER_ERROR)
                })?,
                pool_id: row.try_get::<String, _>("pool_id").map_err(|e| {
                    poem::Error::from_string(format!("Error processing query results: {}", e), StatusCode::INTERNAL_SERVER_ERROR)
                })?,
                timestamp: row.try_get::<chrono::DateTime<chrono::Utc>, _>("timestamp")
                    .map_err(|e| {
                        poem::Error::from_string(format!("Error processing query results: {}", e), StatusCode::INTERNAL_SERVER_ERROR)
                    })?
                    .timestamp_millis()
                    .to_string(),
                price: row.try_get::<Decimal, _>("price").map_err(|e| {
                    poem::Error::from_string(format!("Error processing query results: {}", e), StatusCode::INTERNAL_SERVER_ERROR)
                })?.to_string(),
                quantity: row.try_get::<Decimal, _>("quantity").map_err(|e| {
                    poem::Error::from_string(format!("Error processing query results: {}", e), StatusCode::INTERNAL_SERVER_ERROR)
                })?.to_string(),
                volume: row.try_get::<Decimal, _>("volume").map_err(|e| {
                    poem::Error::from_string(format!("Error processing query results: {}", e), StatusCode::INTERNAL_SERVER_ERROR)
                })?.to_string(),
            };
            Ok(poem::web::Json(Some(trade)))
        }
        None => Ok(poem::web::Json(None)),
    }
}
