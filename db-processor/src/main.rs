use anyhow::{Ok, Result};
use tracing::info;

mod subscriber;
mod types;

use subscriber::RedisStreamSubscriber;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    info!("Starting DB-processor");

    let redis_url =
        std::env::var("REDIS_URL").unwrap_or_else(|_| "".to_string());
    info!("Connecting to Redis at: {}", redis_url);

    let client = redis::Client::open(redis_url)?;
    let mut subscriber = RedisStreamSubscriber::new(client).await?;

    info!("DB-processor initialized successfully");

    // Start consuming messages from the Redis stream
    subscriber.run().await?;

    Ok(())
}
