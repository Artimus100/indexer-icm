use crate::types::MeteoraEvent;
use anyhow::Result;
use chrono::Utc;
use redis::{
    AsyncCommands, Client, RedisResult, Value,
    aio::MultiplexedConnection,
    streams::{StreamReadOptions, StreamReadReply},
};
use rust_decimal::Decimal;
use std::collections::HashMap;
use std::result::Result::Ok;
use tracing::{error, info};

pub struct RedisStreamSubscriber {
    connection: MultiplexedConnection,
    consumer_group: String,
    consumer_name: String,
    // DB-processor needs these specific streams
    pool_created_stream: String,
    swap_stream: String,
    metadata_created_stream: String,
    migration_stream: String,
    store: store::Store,
}

impl RedisStreamSubscriber {
    pub async fn new(client: Client) -> Result<Self> {
        let connection = client.get_multiplexed_async_connection().await?;
        let consumer_group = "db_processor_group".to_string();
        let consumer_name = format!("db-processor-{}", uuid::Uuid::new_v4());
        
        // DB-processor needs these specific streams
        let pool_created_stream = std::env::var("POOL_CREATED_STREAM").unwrap_or_else(|_| "dbc:pool_created".to_string());
        let swap_stream = std::env::var("SWAP_STREAM").unwrap_or_else(|_| "dbc:swap".to_string());
        let metadata_created_stream = std::env::var("METADATA_CREATED_STREAM").unwrap_or_else(|_| "dbc:metadata_created".to_string());
        let migration_stream = std::env::var("MIGRATION_STREAM").unwrap_or_else(|_| "dbc:migration".to_string());

        // Initialize database store
        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "".to_string());
            println!("{}", database_url);
        let store = store::Store::new(&database_url)
            .map_err(|e| anyhow::anyhow!("Failed to connect to database: {}", e))?;

        Ok(Self {
            connection,
            consumer_group,
            consumer_name,
            pool_created_stream,
            swap_stream,
            metadata_created_stream,
            migration_stream,
            store,
        })
    }

    pub async fn run(&mut self) -> Result<()> {
        info!(
            "Starting Redis stream subscriber for DB-processor"
        );
        info!(
            "Consumer: {} in group: {}",
            self.consumer_name, self.consumer_group
        );

        // Create consumer groups for each stream (ignore error if they already exist)
        let _: RedisResult<String> = self
            .connection
            .xgroup_create_mkstream(&self.pool_created_stream, &self.consumer_group, "0")
            .await;

        let _: RedisResult<String> = self
            .connection
            .xgroup_create_mkstream(&self.swap_stream, &self.consumer_group, "0")
            .await;

        let _: RedisResult<String> = self
            .connection
            .xgroup_create_mkstream(&self.metadata_created_stream, &self.consumer_group, "0")
            .await;

        let _: RedisResult<String> = self
            .connection
            .xgroup_create_mkstream(&self.migration_stream, &self.consumer_group, "0")
            .await;

        info!(
            "Consumer group '{}' created/verified for streams: {}, {}, {}, {}",
            self.consumer_group, self.pool_created_stream, self.swap_stream, self.metadata_created_stream, self.migration_stream
        );

        loop {
            if let Err(e) = self.consume_messages().await {
                error!("Error consuming messages: {}", e);
                // Wait before retrying
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            }
        }
    }

    async fn consume_messages(&mut self) -> Result<()> {
        // Read from the stream using consumer group
        let opts = StreamReadOptions::default()
            .group(&self.consumer_group, &self.consumer_name)
            .count(10) // Read up to 10 messages at once
            .block(1000); // Block for 1 second if no messages

        // Read from all DB-processor relevant streams
        let results: RedisResult<StreamReadReply> = self
            .connection
            .xread_options(
                &[&self.pool_created_stream, &self.swap_stream, &self.metadata_created_stream, &self.migration_stream],
                &[">", ">", ">", ">"], // Read only new messages
                &opts,
            )
            .await;

        if let std::result::Result::Ok(stream_reply) = results {
            for stream_key in stream_reply.keys {
                for stream_id in stream_key.ids {
                    let mut fields = HashMap::new();
                    for (key, value) in stream_id.map.iter() {
                        fields.insert(key.clone(), value.clone());
                    }

                    // Determine which stream this message came from
                    let stream_name = &stream_key.key;
                    let ack_stream = if stream_name == &self.pool_created_stream {
                        self.pool_created_stream.clone()
                    } else if stream_name == &self.swap_stream {
                        self.swap_stream.clone()
                    } else if stream_name == &self.metadata_created_stream {
                        self.metadata_created_stream.clone()
                    } else if stream_name == &self.migration_stream {
                        self.migration_stream.clone()
                    } else {
                        // Unknown stream, skip
                        continue;
                    };

                    if let Err(e) = self.process_message(&stream_id.id, &fields, stream_name).await {
                        error!("Failed to process message {}: {}", stream_id.id, e);
                    // Note: We don't acknowledge failed messages so they can be retried
                    } else {
                        // Acknowledge the message
                        let _: RedisResult<i32> = self
                            .connection
                            .xack(&ack_stream, &self.consumer_group, &[&stream_id.id])
                            .await;
                    }
                }
            }
        } else if let std::result::Result::Err(_e) = results {
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }

        Ok(())
    }

    async fn process_message(
        &mut self,
        _message_id: &str,
        fields: &HashMap<String, Value>,
        stream_name: &str,
    ) -> Result<()> {
        // Extract the event data from the message
        let data = self.extract_field_as_string(fields, "data")?;

        info!("Raw stream data from {}: {}", stream_name, data);

        // Parse the JSON data into MeteoraEvent
        let event = serde_json::from_str::<MeteoraEvent>(&data)
            .map_err(|e| anyhow::anyhow!("JSON parsing error: {}", e))?;

        // Handle events based on which stream they came from
        if stream_name == &self.pool_created_stream {
            self.handle_pool_creation_for_coin(&event).await?;
        } else if stream_name == &self.swap_stream {
            self.handle_swap_for_coin(&event).await?;
        } else if stream_name == &self.metadata_created_stream {
            info!("Received metadata event: {:?}", event);
        } else if stream_name == &self.migration_stream {
            self.handle_migration_for_coin(&event).await?;
        } else {
            info!("Received event from unknown stream: {}", stream_name);
        }

        Ok(())
    }


    fn extract_field_as_string(
        &self,
        fields: &HashMap<String, Value>,
        field_name: &str,
    ) -> Result<String> {
        match fields.get(field_name) {
            Some(Value::BulkString(data)) => String::from_utf8(data.clone())
                .map_err(|e| anyhow::anyhow!("Failed to convert {} to string: {}", field_name, e)),
            Some(value) => {
                // Try to convert other value types to string
                match value {
                    Value::SimpleString(s) => Ok(s.clone()),
                    Value::Int(i) => Ok(i.to_string()),
                    _ => Err(anyhow::anyhow!(
                        "Field {} has unexpected type: {:?}",
                        field_name,
                        value
                    )),
                }
            }
            None => Err(anyhow::anyhow!("Missing required field: {}", field_name)),
        }
    }

    // ===== LAUNCHPAD COIN OPERATIONS =====

    /// Handle pool creation by creating a launchpad coin
    async fn handle_pool_creation_for_coin(&mut self, event: &MeteoraEvent) -> Result<()> {
        if let MeteoraEvent::PoolCreated {
            pool_id,
            creator,
            base_mint,
            name,
            symbol,
            uri,
            ..
        } = event
        {
            // Check if coin already exists
            match self.store.get_launchpad_coin_by_mint(base_mint) {
                Ok(Some(_)) => {
                    info!(
                        "Launchpad coin for mint {} already exists, skipping creation",
                        base_mint
                    );
                    return Ok(());
                }
                Ok(None) => {} // Continue with creation
                Err(e) => {
                    error!("Error checking if coin exists: {}", e);
                    return Err(anyhow::anyhow!("Failed to check if coin exists: {}", e));
                }
            }

            // Create new launchpad coin
            let new_coin = store::NewLaunchpadCoin {
                mint: base_mint.clone(),
                name: name.clone(),
                symbol: symbol.clone(),
                description: None, // Will be filled from metadata if available
                image_uri: Some(uri.clone()),
                metadata_uri: Some(uri.clone()),
                twitter: None,
                telegram: None,
                website: None,
                pool_address: pool_id.clone(),
                creator: creator.clone(),
                created_timestamp: Utc::now(),
                complete: false,
                virtual_sol_reserves: 0, // Will be updated when pool updates come
                virtual_token_reserves: 0,
                total_supply: 1_000_000_000, // Default for pump.fun tokens
                market_cap: None,
                usd_market_cap: None,
                reply_count: 0,
                is_banned: false,
                program: "dbc".to_string(),
            };

            self.store
                .create_launchpad_coin(new_coin)
                .map_err(|e| anyhow::anyhow!("Failed to create launchpad coin: {}", e))?;

            info!("Created launchpad coin for mint: {}", base_mint);
        }
        Ok(())
    }


    /// Handle migration events by marking coins as complete
    async fn handle_migration_for_coin(&mut self, event: &MeteoraEvent) -> Result<()> {
        if let MeteoraEvent::MigrationDammV2 { virtual_pool, .. } = event {
            // Find the coin by pool address
            match self.store.get_launchpad_coin_by_pool(virtual_pool) {
                Ok(Some(coin)) => {
                    // Mark as complete
                    self.store
                        .mark_coin_complete(&coin.mint)
                        .map_err(|e| anyhow::anyhow!("Failed to mark coin as complete: {}", e))?;

                    info!("Marked coin {} as complete (migrated)", coin.mint);
                }
                Ok(None) => {
                    info!(
                        "No launchpad coin found for pool {}, skipping migration",
                        virtual_pool
                    );
                }
                Err(e) => {
                    error!("Error finding coin for pool {}: {}", virtual_pool, e);
                }
            }
        }
        Ok(())
    }

    /// Handle swap events for launchpad coins and update holder balances
    async fn handle_swap_for_coin(&mut self, swap: &MeteoraEvent) -> Result<()> {
        if let MeteoraEvent::Swap {
            pool_id,
            user,
            amount_in,
            minimum_amount_out,
            ..
        } = swap
        {
            // Get the coin by pool address
            if let Ok(Some(coin)) = self.store.get_launchpad_coin_by_pool(pool_id) {
                // Update holder balance
                self.update_holder_balance(&coin.mint, user, *minimum_amount_out, *amount_in)
                    .await?;

                info!(
                    "Swap detected for coin {}: {} SOL -> {} tokens by user {}",
                    coin.mint, amount_in, minimum_amount_out, user
                );
            }
        }
        Ok(())
    }

    /// Update holder balance after a swap
    async fn update_holder_balance(
        &mut self,
        mint: &str,
        wallet_address: &str,
        token_amount: u64,
        sol_amount: u64,
    ) -> Result<()> {
        // Convert amounts to Decimal with proper decimal places
        // Both tokens have 9 decimals, so we need to divide by 10^9
        let token_amount_decimal = Decimal::from(token_amount) / Decimal::from(1_000_000_000i64);
        let sol_amount_decimal = Decimal::from(sol_amount) / Decimal::from(1_000_000_000i64); // Convert lamports to SOL

        // Check if holder already exists
        match self.store.get_holder(mint, wallet_address) {
            Ok(Some(holder)) => {
                // Update existing holder
                let new_amount = holder.amount + token_amount_decimal;
                let new_sol_balance = holder.sol_balance + sol_amount_decimal;

                let update_data = store::UpdateHolder {
                    amount: Some(new_amount),
                    sol_balance: Some(new_sol_balance),
                    updated_at: Some(Utc::now()),
                };

                if let Err(e) = self.store.update_holder(mint, wallet_address, update_data) {
                    tracing::error!("Failed to update holder balance: {}", e);
                }
            }
            Ok(None) => {
                // Create new holder
                let new_holder = store::NewHolder {
                    mint: mint.to_string(),
                    wallet_address: wallet_address.to_string(),
                    amount: token_amount_decimal,
                    sol_balance: sol_amount_decimal,
                };

                if let Err(e) = self.store.upsert_holder(new_holder) {
                    tracing::error!("Failed to create holder: {}", e);
                }
            }
            Err(e) => {
                tracing::error!("Failed to check holder existence: {}", e);
            }
        }

        Ok(())
    }
}
