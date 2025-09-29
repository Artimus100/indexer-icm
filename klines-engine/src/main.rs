use std::time::Duration;

use anyhow::{Ok, Result};
use dotenvy::dotenv;
use sqlx::PgPool;
use tracing::{info, error};
use crate::manager::ensure_trading_tables;
use crate::subscriber::RedisStreamSubscriber;

mod subscriber;
mod types;
mod manager;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    dotenv().ok();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = PgPool::connect(&database_url).await?;

    // Ensure trading tables exist on startup
    ensure_trading_tables(&pool).await?;
    info!("Trading tables ensured successfully");

    let refresh_pool = pool.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(10));
        loop {
            interval.tick().await;
            if let Err(e) = refresh_views(&refresh_pool).await {
                error!("Error refreshing views: {:?}", e);
            } else {
                info!("Materialized views refreshed successfully");
            }
        }
    });

    // Start Redis consumer for trade events
    let redis_url = std::env::var("REDIS_URL").unwrap_or_else(|_| "".to_string());
    let redis_client = redis::Client::open(redis_url)?;
    
    let subscriber_pool = pool.clone();
    tokio::spawn(async move {
        let subscriber_result = RedisStreamSubscriber::new(redis_client, subscriber_pool).await;
        let mut subscriber = match subscriber_result {
            std::result::Result::Ok(subscriber) => subscriber,
            std::result::Result::Err(e) => {
                error!("Failed to create Redis subscriber: {:?}", e);
                return;
            }
        };
        
        if let Err(e) = subscriber.run().await {
            error!("Redis subscriber error: {:?}", e);
        }
    });

    // Keep the main thread alive
    info!("Klines-engine started successfully. Press Ctrl+C to stop.");
    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
        info!("Klines-engine is running...");
    }
}


async fn refresh_views(pool: &PgPool) -> Result<()> {
    // Query Postgres for all matview names in public that start with "ohlcv_"
    let views = sqlx::query_as::<_, (Option<String>,)>(
        r#"
        SELECT matviewname
          FROM pg_matviews
         WHERE schemaname = 'public'
           AND matviewname LIKE 'ohlcv_%'
        "#
    )
    .fetch_all(pool)
    .await?;

    for (matviewname,) in views {
        let name = &matviewname.unwrap();
        let sql = format!("REFRESH MATERIALIZED VIEW public.{}", name);
        sqlx::query(&sql).execute(pool).await?;
        info!("Refreshed mat‑view public.{}", name);
    }

    Ok(())
}

// TODO: Implement Redis stream consumer
// This will be added when Redis async streams are properly configured