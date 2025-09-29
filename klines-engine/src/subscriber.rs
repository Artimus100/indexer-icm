use anyhow::Result;
use redis::{Client, RedisResult, Value, AsyncCommands};
use std::collections::HashMap;
use tracing::{error, info};
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use sqlx::PgPool;
use crate::types::AddTradePayload;
use crate::manager::process_trade;

pub struct RedisStreamSubscriber {
    connection: redis::aio::MultiplexedConnection,
    consumer_group: String,
    consumer_name: String,
    // Klines-engine needs these specific streams
    swap_stream: String,
    pool: PgPool,
}

impl RedisStreamSubscriber {
    pub async fn new(client: Client, pool: PgPool) -> Result<Self> {
        let connection = client.get_multiplexed_async_connection().await?;
        let consumer_group = "klines_processor_group".to_string();
        let consumer_name = format!("klines-processor-{}", std::process::id());
        
        // Klines-engine needs these specific streams
        let swap_stream = std::env::var("SWAP_STREAM").unwrap_or_else(|_| "dbc:swap".to_string());

        Ok(Self {
            connection,
            consumer_group,
            consumer_name,
            swap_stream,
            pool,
        })
    }

    pub async fn run(&mut self) -> Result<()> {
        info!(
            "Starting Redis stream subscriber for klines-engine"
        );
        info!(
            "Consumer: {} in group: {}",
            self.consumer_name, self.consumer_group
        );

        // Create consumer groups for each stream (ignore error if they already exist)
        let _: RedisResult<String> = self
            .connection
            .xgroup_create_mkstream(&self.swap_stream, &self.consumer_group, "0")
            .await;

        info!(
            "Consumer group '{}' created/verified for streams: {}",
            self.consumer_group, self.swap_stream
        );

        // Main consumption loop
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
        let opts = redis::streams::StreamReadOptions::default()
            .group(&self.consumer_group, &self.consumer_name)
            .count(10) // Read up to 10 messages at once
            .block(1000); // Block for 1 second if no messages

        // Read from all klines-engine relevant streams
        let results: RedisResult<redis::streams::StreamReadReply> = self
            .connection
            .xread_options(
                &[&self.swap_stream],
                &[">"], // Read only new messages
                &opts,
            )
            .await;

        if let Ok(stream_reply) = results {
            info!("Received {} stream keys from Redis", stream_reply.keys.len());
            for stream_key in stream_reply.keys {
                info!("Processing {} messages from stream", stream_key.ids.len());
                for stream_id in stream_key.ids {
                    let mut fields = HashMap::new();
                    for (key, value) in stream_id.map.iter() {
                        fields.insert(key.clone(), value.clone());
                    }

                    // Determine which stream this message came from
                    let stream_name = &stream_key.key;
                    let ack_stream = if stream_name == &self.swap_stream {
                        self.swap_stream.clone()
                    } else {
                        // Unknown stream, skip
                        continue;
                    };

                    info!("Processing message ID: {} from stream: {}", stream_id.id, stream_name);
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
        } else {
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
        let event_type = self.extract_field_as_string(fields, "event_type")?;

        info!("Processing event: {} from stream {} with data: {}", event_type, stream_name, data);

        // Handle events based on which stream they came from
        if stream_name == &self.swap_stream {
            // Process swap events as trades
            let swap_data = serde_json::from_str::<serde_json::Value>(&data)
                .map_err(|e| anyhow::anyhow!("JSON parsing error: {}", e))?;

            self.process_swap_as_trade(&swap_data).await?;
        } else {
            // Log other events but don't process them
            info!("Ignoring event from unknown stream: {}", stream_name);
        }

        Ok(())
    }


    async fn process_swap_as_trade(
        &mut self,
        swap_data: &serde_json::Value,
    ) -> Result<()> {
        let pool_id = swap_data["pool_id"].as_str().unwrap_or("UNKNOWN");
        let amount_in = swap_data["amount_in"].as_u64().unwrap_or(0);
        let minimum_amount_out = swap_data["minimum_amount_out"].as_u64().unwrap_or(0);
        let timestamp = swap_data["timestamp"].as_u64().unwrap_or(0);
        let base_mint = swap_data["base_mint"].as_str().unwrap_or("UNKNOWN");

        // Convert timestamp to DateTime
        let time = DateTime::from_timestamp(timestamp as i64, 0)
            .unwrap_or_else(|| Utc::now());

        // Constants for decimal places - BOTH NOW HAVE 9 DECIMALS
        const BASE_MINT_DECIMALS: u32 = 9;  // Your token decimals
        const QUOTE_MINT_DECIMALS: u32 = 9;  // SOL decimals

        // Convert from raw units to decimal values
        let amount_in_decimal = Decimal::from(amount_in) / Decimal::from(10_u64.pow(QUOTE_MINT_DECIMALS));
        let amount_out_decimal = Decimal::from(minimum_amount_out) / Decimal::from(10_u64.pow(BASE_MINT_DECIMALS));

        // Calculate price: SOL per token (quote per base)
        let price = if amount_out_decimal > Decimal::ZERO {
            amount_in_decimal / amount_out_decimal
        } else {
            Decimal::ZERO
        };

        // Quantity is the amount of tokens (base mint) received
        let quantity = amount_out_decimal;
        let ticker = base_mint.to_string();

        // Create trade payload
        let trade_payload = AddTradePayload {
            ticker: ticker.clone(),
            pool_id: pool_id.to_string(),
            time,
            quantity,
            price,
        };

        // Process the trade
        process_trade(&self.pool, &trade_payload).await?;
        
        info!("Processed swap as trade: {} {} tokens at {} SOL per token (pool: {})", 
              ticker, quantity, price, pool_id);
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
}