use anyhow::Result;
use chrono::Utc;
use futures::SinkExt;
use redis::{
    AsyncCommands, Client, RedisResult, Value,
    aio::MultiplexedConnection,
    streams::{StreamReadOptions, StreamReadReply},
};
use rust_decimal::Decimal;
use std::collections::HashMap;
use tracing::{error, info};

use crate::state::SharedState;
use crate::types::{MeteoraEvent, ServerMessage, TradeData};

pub struct RedisStreamConsumer {
    connection: MultiplexedConnection,
    consumer_group: String,
    consumer_name: String,
    // WSS needs these specific streams
    pool_created_stream: String,
    swap_stream: String,
    klines_stream: String,
    state: SharedState,
}

impl RedisStreamConsumer {
    pub async fn new(client: Client, state: SharedState) -> Result<Self> {
        let connection = client.get_multiplexed_async_connection().await?;
        let consumer_group = "wss_consumer_group".to_string();
        let consumer_name = format!("wss-consumer-{}", uuid::Uuid::new_v4());
        
        // WSS needs these specific streams
        let pool_created_stream = std::env::var("POOL_CREATED_STREAM").unwrap_or_else(|_| "dbc:pool_created".to_string());
        let swap_stream = std::env::var("SWAP_STREAM").unwrap_or_else(|_| "dbc:swap".to_string());
        let klines_stream = std::env::var("KLINES_STREAM").unwrap_or_else(|_| "klines_updates".to_string());

        Ok(Self {
            connection,
            consumer_group,
            consumer_name,
            pool_created_stream,
            swap_stream,
            klines_stream,
            state,
        })
    }

    pub async fn run(&mut self) -> Result<()> {
        info!("Starting Redis stream consumer for WebSocket server");

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
            .xgroup_create_mkstream(&self.klines_stream, &self.consumer_group, "0")
            .await;

        info!(
            "Consumer group '{}' created/verified for streams: {}, {}, {}",
            self.consumer_group, self.pool_created_stream, self.swap_stream, self.klines_stream
        );

        // Main consumption loop
        loop {
            if let Err(e) = self.consume_messages().await {
                error!("Error consuming messages: {}", e);
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            }
        }
    }

    async fn consume_messages(&mut self) -> Result<()> {
        let opts = StreamReadOptions::default()
            .group(&self.consumer_group, &self.consumer_name)
            .count(10)
            .block(1000);

        // Read from all WSS-relevant streams
        let results: RedisResult<StreamReadReply> = self
            .connection
            .xread_options(&[&self.pool_created_stream, &self.swap_stream, &self.klines_stream], &[">", ">", ">"], &opts)
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
                        &self.pool_created_stream
                    } else if stream_name == &self.swap_stream {
                        &self.swap_stream
                    } else if stream_name == &self.klines_stream {
                        &self.klines_stream
                    } else {
                        // Unknown stream, skip
                        continue;
                    };

                    if let Err(e) = self.process_message(&stream_id.id, &fields, stream_name).await {
                        error!("Failed to process message {}: {}", stream_id.id, e);
                    } else {
                        // Acknowledge the message
                        let _: RedisResult<i32> = self
                            .connection
                            .xack(ack_stream, &self.consumer_group, &[&stream_id.id])
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
        &self,
        _message_id: &str,
        fields: &HashMap<String, Value>,
        stream_name: &str,
    ) -> Result<()> {
        let data = self.extract_field_as_string(fields, "data")?;

        // Handle events based on which stream they came from
        if stream_name == &self.pool_created_stream {
            // Parse as MeteoraEvent for pool created events
            let event = serde_json::from_str::<MeteoraEvent>(&data)
                .map_err(|e| anyhow::anyhow!("JSON parsing error: {}", e))?;
            self.handle_pool_created(&event).await?;
        } else if stream_name == &self.swap_stream {
            // Parse as MeteoraEvent for swap events
            let event = serde_json::from_str::<MeteoraEvent>(&data)
                .map_err(|e| anyhow::anyhow!("JSON parsing error: {}", e))?;
            self.handle_swap(&event).await?;
        } else if stream_name == &self.klines_stream {
            // Handle klines data directly (no MeteoraEvent parsing needed)
            self.handle_klines_update(&data).await?;
        } else {
            info!("Received event from unknown stream: {}", stream_name);
        }
        Ok(())
    }


    async fn handle_launch_pool_created(&self, event: &MeteoraEvent) -> Result<()> {
        if let MeteoraEvent::PoolCreated {
            pool_id,
            name,
            symbol,
            base_mint: _,
            quote_mint: _,
            creator,
            uri,
            base_vault: _,
            quote_vault: _,
            timestamp: _,
        } = event
        {
            // Create launch coin data for the new room-based system
            let launch_coin_data = crate::types::LaunchCoinData {
                id: 0, // This would need to be fetched from database
                mint: pool_id.clone(),
                name: name.clone(),
                symbol: symbol.clone(),
                description: None,
                image_uri: Some(uri.clone()),
                metadata_uri: Some(uri.clone()),
                twitter: None,
                telegram: None,
                website: None,
                pool_address: pool_id.clone(),
                creator: creator.clone(),
                created_timestamp: chrono::Utc::now().timestamp().to_string(),
                complete: false,
                virtual_sol_reserves: 0,
                virtual_token_reserves: 0,
                total_supply: 0,
                market_cap: None,
                usd_market_cap: None,
                reply_count: 0,
                is_banned: false,
                program: "meteora".to_string(),
                created_at: chrono::Utc::now().to_rfc3339(),
                updated_at: chrono::Utc::now().to_rfc3339(),
            };

            let launch_msg = ServerMessage::LaunchCoin { data: launch_coin_data };
            // Broadcast to coins room for new launch events
            self.broadcast_to_room("coins", &launch_msg).await?;
        }
        Ok(())
    }

    async fn handle_pool_created(&self, event: &MeteoraEvent) -> Result<()> {
        if let MeteoraEvent::PoolCreated {
            pool_id,
            name,
            symbol,
            base_mint: _,
            quote_mint: _,
            creator,
            uri,
            base_vault: _,
            quote_vault: _,
            timestamp: _,
        } = event
        {
            info!("Processing pool creation for pool {} - broadcasting to coins room", pool_id);
            
            // Create launch coin data for the new room-based system
            let launch_coin_data = crate::types::LaunchCoinData {
                id: 0, // This would need to be fetched from database
                mint: pool_id.clone(),
                name: name.clone(),
                symbol: symbol.clone(),
                description: None,
                image_uri: Some(uri.clone()),
                metadata_uri: Some(uri.clone()),
                twitter: None,
                telegram: None,
                website: None,
                pool_address: pool_id.clone(),
                creator: creator.clone(),
                created_timestamp: chrono::Utc::now().timestamp().to_string(),
                complete: false,
                virtual_sol_reserves: 0,
                virtual_token_reserves: 0,
                total_supply: 0,
                market_cap: None,
                usd_market_cap: None,
                reply_count: 0,
                is_banned: false,
                program: "meteora".to_string(),
                created_at: chrono::Utc::now().to_rfc3339(),
                updated_at: chrono::Utc::now().to_rfc3339(),
            };

            let launch_msg = ServerMessage::LaunchCoin { data: launch_coin_data };
            // Broadcast to coins room for new launch events
            self.broadcast_to_room("coins", &launch_msg).await?;
        }
        Ok(())
    }

    async fn handle_swap(&self, event: &MeteoraEvent) -> Result<()> {
        if let MeteoraEvent::Swap {
            pool_id,
            user,
            amount_in,
            minimum_amount_out,
            base_mint,
            quote_mint,
            input_token_account,
            output_token_account: _,
            timestamp: _,
        } = event
        {
            // Determine if it's a buy or sell based on the token accounts
            // If input_token_account is the quote_mint (SOL), it's a buy
            // If input_token_account is the base_mint (Token), it's a sell
            let trade_type = if input_token_account.contains(quote_mint) {
                "buy" // SOL -> Token
            } else if input_token_account.contains(base_mint) {
                "sell" // Token -> SOL
            } else {
                // Fallback: determine based on amounts
                if *amount_in < *minimum_amount_out {
                    "buy" // Small amount in, large amount out = buy
                } else {
                    "sell" // Large amount in, small amount out = sell
                }
            };

            // Calculate price at trade (amount_out / amount_in)
            let price_at_trade = if *amount_in > 0 {
                Some(Decimal::from(*minimum_amount_out) / Decimal::from(*amount_in))
            } else {
                None
            };

            // Calculate trade value in SOL (for volume calculations)
            let _trade_value_sol = if trade_type == "buy" {
                Some(Decimal::from(*amount_in) / Decimal::from(1_000_000_000)) // Convert lamports to SOL
            } else {
                Some(Decimal::from(*minimum_amount_out) / Decimal::from(1_000_000_000)) // Convert lamports to SOL
            };

            let trade_data = TradeData {
                token_id: pool_id.clone(),
                time: Utc::now(),
                user_address: user.clone(),
                amount_in: *amount_in as i64,
                amount_out: *minimum_amount_out as i64,
                price_at_trade,
                trade_type: Some(trade_type.to_string()),
                tx_signature: None,
            };

            // Broadcast to trades room for this specific pool
            let trades_room = format!("trades@{}", pool_id);
            info!("Processing swap event for pool {} - broadcasting to room {}", pool_id, trades_room);
            let trade_msg = ServerMessage::Trade { data: trade_data.clone() };
            self.broadcast_to_room(&trades_room, &trade_msg).await?;

            // Legacy support
            let server_msg = ServerMessage::TradeData { data: trade_data };
            self.broadcast_to_token_subscribers(pool_id, &server_msg)
                .await?;
        }
        Ok(())
    }


    async fn handle_metadata_created(&self, event: &MeteoraEvent) -> Result<()> {
        if let MeteoraEvent::MetadataCreated {
            virtual_pool,
            pool_creator,
            partner,
            timestamp: _,
        } = event
        {
            let server_msg = ServerMessage::MetadataCreated {
                token_id: virtual_pool.clone(),
                pool_creator: pool_creator.clone(),
                partner: partner.clone(),
            };
            self.broadcast_to_token_subscribers(virtual_pool, &server_msg)
                .await?;
        }
        Ok(())
    }

    async fn handle_migration_metadata_created(&self, event: &MeteoraEvent) -> Result<()> {
        if let MeteoraEvent::MigrationMetadataCreated {
            virtual_pool,
            config,
            migration_metadata,
            payer,
            timestamp: _,
        } = event
        {
            let server_msg = ServerMessage::MigrationMetadataCreated {
                token_id: virtual_pool.clone(),
                config: config.clone(),
                migration_metadata: migration_metadata.clone(),
                payer: payer.clone(),
            };
            self.broadcast_to_token_subscribers(virtual_pool, &server_msg)
                .await?;
        }
        Ok(())
    }

    async fn handle_locker_created(&self, event: &MeteoraEvent) -> Result<()> {
        if let MeteoraEvent::LockerCreated {
            virtual_pool,
            config,
            base_mint,
            creator,
            escrow,
            escrow_token,
            timestamp: _,
        } = event
        {
            let server_msg = ServerMessage::LockerCreated {
                token_id: virtual_pool.clone(),
                config: config.clone(),
                base_mint: base_mint.clone(),
                creator: creator.clone(),
                escrow: escrow.clone(),
                escrow_token: escrow_token.clone(),
            };
            self.broadcast_to_token_subscribers(virtual_pool, &server_msg)
                .await?;
        }
        Ok(())
    }

    async fn handle_migration_damm_v2(&self, event: &MeteoraEvent) -> Result<()> {
        if let MeteoraEvent::MigrationDammV2 {
            virtual_pool,
            config,
            pool,
            base_mint,
            quote_mint,
            payer,
            first_position_nft_mint,
            second_position_nft_mint,
            timestamp: _,
        } = event
        {
            let server_msg = ServerMessage::MigrationDammV2 {
                token_id: virtual_pool.clone(),
                config: config.clone(),
                pool: pool.clone(),
                base_mint: base_mint.clone(),
                quote_mint: quote_mint.clone(),
                payer: payer.clone(),
                first_position_nft_mint: first_position_nft_mint.clone(),
                second_position_nft_mint: second_position_nft_mint.clone(),
            };
            self.broadcast_to_token_subscribers(virtual_pool, &server_msg)
                .await?;
        }
        Ok(())
    }

    async fn handle_klines_update(&self, data: &str) -> Result<()> {
        info!("Processing klines data: {}", data);
        
        // Parse the klines data from the stream
        let klines_data: serde_json::Value = serde_json::from_str(data)
            .map_err(|e| anyhow::anyhow!("Failed to parse klines data: {}", e))?;
        
        let pool_id = klines_data["pool_id"].as_str().unwrap_or("unknown");
        let interval = klines_data["interval"].as_str().unwrap_or("1m");
        
        // Create KlineData for WebSocket broadcasting
        let kline_data = crate::types::KlineData {
            close: klines_data["close"].as_str().unwrap_or("0").to_string(),
            end: klines_data["end"].as_str().unwrap_or("0").to_string(),
            high: klines_data["high"].as_str().unwrap_or("0").to_string(),
            low: klines_data["low"].as_str().unwrap_or("0").to_string(),
            open: klines_data["open"].as_str().unwrap_or("0").to_string(),
            quote_volume: klines_data["quote_volume"].as_str().unwrap_or("0").to_string(),
            start: klines_data["start"].as_str().unwrap_or("0").to_string(),
            trades: klines_data["trades"].as_str().unwrap_or("0").to_string(),
            volume: klines_data["volume"].as_str().unwrap_or("0").to_string(),
        };
        
        let kline_msg = ServerMessage::Kline { data: kline_data };
        
        // Broadcast to klines room for this specific pool and interval
        let klines_room = format!("klines@{}@{}", pool_id, interval);
        info!("Broadcasting klines update to room: {}", klines_room);
        self.broadcast_to_room(&klines_room, &kline_msg).await?;
        
        Ok(())
    }

    async fn broadcast_to_room(&self, room: &str, message: &ServerMessage) -> Result<()> {
        let subscribers = self.state.room_subscribers.read().await;
        if let Some(room_subscribers) = subscribers.get(room) {
            info!("Broadcasting to room '{}' with {} subscribers", room, room_subscribers.len());
            let message_json = serde_json::to_string(message)?;
            for subscriber in room_subscribers {
                let mut sender = subscriber.lock().await;
                if let Err(e) = sender
                    .send(axum::extract::ws::Message::Text(
                        message_json.clone().into(),
                    ))
                    .await
                {
                    error!("Failed to send message to room subscriber: {}", e);
                }
            }
        } else {
            info!("No subscribers found for room '{}'", room);
        }
        Ok(())
    }

    async fn broadcast_to_token_subscribers(
        &self,
        token_id: &str,
        message: &ServerMessage,
    ) -> Result<()> {
        let subscribers = self.state.token_subscribers.read().await;
        if let Some(token_subscribers) = subscribers.get(token_id) {
            let message_json = serde_json::to_string(message)?;
            for subscriber in token_subscribers {
                let mut sender = subscriber.lock().await;
                if let Err(e) = sender
                    .send(axum::extract::ws::Message::Text(
                        message_json.clone().into(),
                    ))
                    .await
                {
                    error!("Failed to send message to subscriber: {}", e);
                }
            }
        }
        Ok(())
    }

    async fn broadcast_to_all_tokens_subscribers(&self, message: &ServerMessage) -> Result<()> {
        let subscribers = self.state.all_tokens_subscribers.read().await;
        let message_json = serde_json::to_string(message)?;
        for subscriber in subscribers.iter() {
            let mut sender = subscriber.lock().await;
            if let Err(e) = sender
                .send(axum::extract::ws::Message::Text(
                    message_json.clone().into(),
                ))
                .await
            {
                error!("Failed to send message to all-tokens subscriber: {}", e);
            }
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
            Some(value) => match value {
                Value::SimpleString(s) => Ok(s.clone()),
                Value::Int(i) => Ok(i.to_string()),
                _ => Err(anyhow::anyhow!(
                    "Field {} has unexpected type: {:?}",
                    field_name,
                    value
                )),
            },
            None => Err(anyhow::anyhow!("Missing required field: {}", field_name)),
        }
    }
}
