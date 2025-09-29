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
pub struct KLine {
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

#[derive(Deserialize)]
pub struct KLineQuery {
    pub interval: Option<String>,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
}

fn get_interval_seconds(interval: &str) -> i64 {
    match interval {
        "1s" => 1,
        "15s" => 15,
        "30s" => 30,
        "1m" => 60,
        "5m" => 300,
        "15m" => 900,
        "30m" => 1800,
        "1h" => 3600,
        "4h" => 14400,
        "6h" => 21600,
        "12h" => 43200,
        _ => 60, // default to 1 minute
    }
}

#[handler]
pub async fn get_klines(
    Path(mint): Path<String>,
    Query(params): Query<KLineQuery>,
) -> PoemResult<poem::web::Json<Vec<KLine>>> {
    let interval = params.interval.unwrap_or_else(|| "1m".to_string());
    
    // Validate interval
    let valid_intervals = ["1s", "15s", "30s", "1m", "5m", "15m", "30m", "1h", "4h", "6h", "12h"];
    if !valid_intervals.contains(&interval.as_str()) {
        return Err(poem::Error::from_string(
            format!("Invalid interval: {}. Valid intervals are: {}", interval, valid_intervals.join(", ")),
            StatusCode::BAD_REQUEST,
        ));
    }

    let start_time = params.start_time.unwrap_or_else(|| {
        // Default to 24 hours ago
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        (now - 86400).to_string()
    });
    
    let end_time = params.end_time.unwrap_or_else(|| {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        now.to_string()
    });

    let database_url = env::var("DATABASE_URL")
        .map_err(|_| poem::Error::from_string("DATABASE_URL not set", StatusCode::INTERNAL_SERVER_ERROR))?;
    
    let pool = PgPool::connect(&database_url)
        .await
        .map_err(|e| poem::Error::from_string(format!("Database connection failed: {}", e), StatusCode::INTERNAL_SERVER_ERROR))?;

    // Use the materialized view created by klines-engine
    let table_name = format!("ohlcv_{}", interval);
    
    // Alternative: Use PostgreSQL's make_interval function
    let sql = match interval.as_str() {
        "1s" | "15s" | "30s" => format!(
            r#"
            SELECT 
                (EXTRACT(EPOCH FROM bucket) * 1000)::BIGINT as start_timestamp,
                (EXTRACT(EPOCH FROM bucket + make_interval(secs => {})) * 1000)::BIGINT as end_timestamp,
                open, high, low, close, volume,
                volume * close as quote_volume,
                1::BIGINT as trades
            FROM public.{}
            WHERE pool_id = $1 AND bucket >= to_timestamp($2) AND bucket <= to_timestamp($3)
            ORDER BY bucket ASC
            "#, 
            get_interval_seconds(&interval), table_name
        ),
        "1m" | "5m" | "15m" | "30m" => format!(
            r#"
            SELECT 
                (EXTRACT(EPOCH FROM bucket) * 1000)::BIGINT as start_timestamp,
                (EXTRACT(EPOCH FROM bucket + make_interval(mins => {})) * 1000)::BIGINT as end_timestamp,
                open, high, low, close, volume,
                volume * close as quote_volume,
                1::BIGINT as trades
            FROM public.{}
            WHERE pool_id = $1 AND bucket >= to_timestamp($2) AND bucket <= to_timestamp($3)
            ORDER BY bucket ASC
            "#, 
            get_interval_seconds(&interval) / 60, table_name
        ),
        _ => format!(
            r#"
            SELECT 
                (EXTRACT(EPOCH FROM bucket) * 1000)::BIGINT as start_timestamp,
                (EXTRACT(EPOCH FROM bucket + make_interval(hours => {})) * 1000)::BIGINT as end_timestamp,
                open, high, low, close, volume,
                volume * close as quote_volume,
                1::BIGINT as trades
            FROM public.{}
            WHERE pool_id = $1 AND bucket >= to_timestamp($2) AND bucket <= to_timestamp($3)
            ORDER BY bucket ASC
            "#, 
            get_interval_seconds(&interval) / 3600, table_name
        ),
    };

    let start_timestamp = start_time.parse::<i64>().map_err(|e| {
        tracing::error!("Failed to parse start_time '{}': {}", start_time, e);
        poem::Error::from_string("Invalid start_time format", StatusCode::BAD_REQUEST)
    })?;
    
    let end_timestamp = end_time.parse::<i64>().map_err(|e| {
        tracing::error!("Failed to parse end_time '{}': {}", end_time, e);
        poem::Error::from_string("Invalid end_time format", StatusCode::BAD_REQUEST)
    })?;

    // Convert to seconds if they're in milliseconds
    let start_seconds = if start_timestamp > 1_000_000_000_000 {
        start_timestamp / 1000
    } else {
        start_timestamp
    };
    
    let end_seconds = if end_timestamp > 1_000_000_000_000 {
        end_timestamp / 1000
    } else {
        end_timestamp
    };
    
    tracing::info!("Query params - mint: {}, interval: {}, start_seconds: {}, end_seconds: {}", 
                   mint, interval, start_seconds, end_seconds);

    let rows = sqlx::query(&sql)
        .bind(&mint)
        .bind(start_seconds)
        .bind(end_seconds)
        .fetch_all(&pool)
        .await;

    match rows {
        Ok(results) => {
            tracing::info!("Query returned {} rows", results.len());
            
            let klines: Result<Vec<KLine>, _> = results
                .into_iter()
                .map(|row| -> Result<KLine, Box<dyn std::error::Error + Send + Sync>> {
                    Ok(KLine {
                        open: row.try_get::<Decimal, _>("open")?.to_string(),
                        high: row.try_get::<Decimal, _>("high")?.to_string(),
                        low: row.try_get::<Decimal, _>("low")?.to_string(),
                        close: row.try_get::<Decimal, _>("close")?.to_string(),
                        volume: row.try_get::<Decimal, _>("volume")?.to_string(),
                        quote_volume: row.try_get::<Decimal, _>("quote_volume")?.to_string(),
                        trades: row.try_get::<i64, _>("trades")?.to_string(),
                        start: row.try_get::<i64, _>("start_timestamp")?.to_string(),
                        end: row.try_get::<i64, _>("end_timestamp")?.to_string(),
                    })
                })
                .collect();

            match klines {
                Ok(klines) => {
                    tracing::info!("Successfully converted {} klines", klines.len());
                    Ok(poem::web::Json(klines))
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
            tracing::error!("DB error fetching klines: {:?}", e);
            Err(poem::Error::from_string(
                format!("Database query failed for table {}: {}", table_name, e),
                StatusCode::INTERNAL_SERVER_ERROR,
            ))
        }
    }
}