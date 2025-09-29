use std::{
    fmt::Debug,
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

use crate::{redis::RedisClient, types::MeteoraEvent};
use tracing::{error, info};
use yellowstone_vixen::{self as vixen};

#[derive(Debug, Clone)]
pub struct Logger {
    redis_client: Arc<RedisClient>,
    pool_created_stream: String,
    swap_stream: String,
    metadata_created_stream: String,
    migration_stream: String,
}

impl Debug for RedisClient {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Ok(())
    }
}

impl Logger {
    pub fn new(redis_url: &str, _stream_key: String) -> Result<Self, Box<dyn std::error::Error>> {
        let redis_client = Arc::new(RedisClient::new(redis_url)?);
        
        // Initialize event-specific streams
        let pool_created_stream = std::env::var("POOL_CREATED_STREAM").unwrap_or_else(|_| "dbc:pool_created".to_string());
        let swap_stream = std::env::var("SWAP_STREAM").unwrap_or_else(|_| "dbc:swap".to_string());
        let metadata_created_stream = std::env::var("METADATA_CREATED_STREAM").unwrap_or_else(|_| "dbc:metadata_created".to_string());
        let migration_stream = std::env::var("MIGRATION_STREAM").unwrap_or_else(|_| "dbc:migration".to_string());
        
        Ok(Self {
            redis_client,
            pool_created_stream,
            swap_stream,
            metadata_created_stream,
            migration_stream,
        })
    }

    fn get_timestamp() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }
}

impl<V: std::fmt::Debug + Sync> vixen::Handler<V> for Logger {
    async fn handle(&self, value: &V) -> vixen::HandlerResult<()> {
        let debug_str = format!("{:#?}", value);
        info!(" raw data: {:#?}", debug_str);

        // Try to parse and extract structured events
        if let Some(event) = self.parse_debug_to_event(&debug_str) {
            // Log only matched events with formatted output
            info!("Matched event:\n{:#?}", event);
            // Send structured event to appropriate stream based on event type
            match serde_json::to_string(&event) {
                Ok(json_str) => {
                    let timestamp = Self::get_timestamp();
                    
                    // Determine which stream to publish to based on event type
                    let (stream_key, event_type) = match &event {
                        MeteoraEvent::PoolCreated { .. } => (&self.pool_created_stream, "pool_created"),
                        MeteoraEvent::Swap { .. } => (&self.swap_stream, "swap"),
                        MeteoraEvent::MetadataCreated { .. } => (&self.metadata_created_stream, "metadata_created"),
                        MeteoraEvent::MigrationMetadataCreated { .. } => (&self.migration_stream, "migration_metadata_created"),
                        MeteoraEvent::LockerCreated { .. } => (&self.migration_stream, "locker_created"),
                        MeteoraEvent::MigrationDammV2 { .. } => (&self.migration_stream, "migration_damm_v2"),
                    };

                    let result: Result<String, redis::RedisError> = redis::cmd("XADD")
                        .arg(stream_key)
                        .arg("*")
                        .arg("timestamp")
                        .arg(timestamp)
                        .arg("event_type")
                        .arg(event_type)
                        .arg("data")
                        .arg(&json_str)
                        .query(&mut *self.redis_client.connection.lock().await);

                    match result {
                        Ok(_) => info!(
                            "Event sent to Redis stream '{}':\n{:#?}",
                            stream_key,
                            match &event {
                                MeteoraEvent::Swap {
                                    pool_id, amount_in, ..
                                } => format!("SWAP: {} lamports in pool {}", amount_in, pool_id),
                                MeteoraEvent::PoolCreated { pool_id, .. } =>
                                    format!("POOL_CREATED: {}", pool_id),
                                MeteoraEvent::MetadataCreated { virtual_pool, .. } =>
                                    format!("METADATA_CREATED: {}", virtual_pool),
                                MeteoraEvent::MigrationMetadataCreated { virtual_pool, .. } =>
                                    format!("MIGRATION_METADATA_CREATED: {}", virtual_pool),
                                MeteoraEvent::LockerCreated { virtual_pool, .. } =>
                                    format!("LOCKER_CREATED: {}", virtual_pool),
                                MeteoraEvent::MigrationDammV2 {
                                    virtual_pool, pool, ..
                                } => format!("MIGRATION_DAMM_V2: {} -> {}", virtual_pool, pool),
                            }
                        ),
                        Err(e) => error!("Failed to send to Redis stream '{}':\n{:#?}", stream_key, e),
                    }
                }
                Err(e) => error!("Failed to serialize event:\n{:#?}", e),
            }
        }
        // Silently ignore unmatched events - no logging needed

        Ok(())
    }
}

impl Logger {
    fn parse_debug_to_event(&self, debug_str: &str) -> Option<MeteoraEvent> {
        // Check if it's a swap event first (most specific)
        if debug_str.contains("Swap(") {
            if let Some(event) = self.extract_swap(debug_str) {
                return Some(event);
            }
        }

        // Check if it's a pool creation event (contains InitializeVirtualPoolWithSplToken)
        if debug_str.contains("InitializeVirtualPoolWithSplToken") {
            if let Some(event) = self.extract_pool_creation(debug_str) {
                return Some(event);
            }
        }

        // Check for migration events
        if debug_str.contains("MigrationDammV2(") {
            if let Some(event) = self.extract_migration_damm_v2(debug_str) {
                return Some(event);
            }
        }

        if debug_str.contains("MigrationDammV2CreateMetadata(") {
            if let Some(event) = self.extract_migration_metadata_created(debug_str) {
                return Some(event);
            }
        }

        // Check for locker creation
        if debug_str.contains("CreateLocker(") {
            if let Some(event) = self.extract_locker_created(debug_str) {
                return Some(event);
            }
        }

        if debug_str.contains("MeteoraDammV2Metadata(") {
            if let Some(event) = self.extract_metadata_created(debug_str) {
                return Some(event);
            }
        }

        // Ignore VirtualPool events - they are just state updates, not actionable events
        if debug_str.contains("VirtualPool(") {
            return None;
        }

        None
    }

    fn extract_pool_creation(&self, debug_str: &str) -> Option<MeteoraEvent> {
        if !debug_str.contains("InitializeVirtualPoolWithSplToken(") {
            return None;
        }

        let timestamp = Self::get_timestamp();

        // Extract pool creation data from InitializeVirtualPoolWithSplToken structure
        let _config = self.extract_field(debug_str, "config:")?;
        let creator = self.extract_field(debug_str, "creator:")?;
        let base_mint = self.extract_field(debug_str, "base_mint:")?;
        let quote_mint = self.extract_field(debug_str, "quote_mint:")?;
        let pool = self.extract_field(debug_str, "pool:")?;
        let base_vault = self.extract_field(debug_str, "base_vault:")?;
        let quote_vault = self.extract_field(debug_str, "quote_vault:")?;

        // Extract pool parameters (name, symbol, uri)
        let name = self.extract_field(debug_str, "name:")?;
        let symbol = self.extract_field(debug_str, "symbol:")?;
        let uri = self.extract_field(debug_str, "uri:")?;

        // Clean up the extracted values (remove quotes)
        let name = name.trim_matches('"').to_string();
        let symbol = symbol.trim_matches('"').to_string();
        let uri = uri.trim_matches('"').to_string();

        Some(MeteoraEvent::PoolCreated {
            timestamp,
            pool_id: pool,
            creator,
            base_mint,
            quote_mint,
            name,
            symbol,
            uri,
            base_vault,
            quote_vault,
        })
    }

    fn extract_swap(&self, debug_str: &str) -> Option<MeteoraEvent> {
        if !debug_str.contains("Swap(") {
            return None;
        }

        let timestamp = Self::get_timestamp();

        // Extract swap data from the structured debug output
        let pool = self.extract_field(debug_str, "pool:")?;
        let payer = self.extract_field(debug_str, "payer:")?;
        let input_token_account = self.extract_field(debug_str, "input_token_account:")?;
        let output_token_account = self.extract_field(debug_str, "output_token_account:")?;
        let base_mint = self.extract_field(debug_str, "base_mint:")?;
        let quote_mint = self.extract_field(debug_str, "quote_mint:")?;

        // Extract amount_in and minimum_amount_out from SwapParameters
        let amount_in = self.extract_number_field(debug_str, "amount_in:")?;
        let minimum_amount_out = self.extract_number_field(debug_str, "minimum_amount_out:")?;

        Some(MeteoraEvent::Swap {
            timestamp,
            pool_id: pool,
            user: payer,
            amount_in,
            minimum_amount_out,
            input_token_account,
            output_token_account,
            base_mint,
            quote_mint,
        })
    }


    // Helper methods for parsing fields from debug strings
    fn extract_field(&self, debug_str: &str, field_name: &str) -> Option<String> {
        if let Some(start) = debug_str.find(field_name) {
            let after_field = &debug_str[start + field_name.len()..];
            let trimmed = after_field.trim_start();

            // Find the value until comma or newline
            let value_end = trimmed
                .find(',')
                .or_else(|| trimmed.find('\n'))
                .unwrap_or(trimmed.len());

            if value_end > 0 {
                return Some(trimmed[..value_end].trim().to_string());
            }
        }
        None
    }

    fn extract_number_field(&self, debug_str: &str, field_name: &str) -> Option<u64> {
        self.extract_field(debug_str, field_name)?
            .parse::<u64>()
            .ok()
    }

    fn extract_metadata_created(&self, debug_str: &str) -> Option<MeteoraEvent> {
        if !debug_str.contains("MeteoraDammV2Metadata(") {
            return None;
        }

        let timestamp = Self::get_timestamp();
        let virtual_pool = self.extract_field(debug_str, "virtual_pool:")?;
        let pool_creator = self.extract_field(debug_str, "pool_creator:")?;
        let partner = self.extract_field(debug_str, "partner:")?;

        Some(MeteoraEvent::MetadataCreated {
            timestamp,
            virtual_pool,
            pool_creator,
            partner,
        })
    }

    fn extract_migration_metadata_created(&self, debug_str: &str) -> Option<MeteoraEvent> {
        if !debug_str.contains("MigrationDammV2CreateMetadata(") {
            return None;
        }

        let timestamp = Self::get_timestamp();
        let virtual_pool = self.extract_field(debug_str, "virtual_pool:")?;
        let config = self.extract_field(debug_str, "config:")?;
        let migration_metadata = self.extract_field(debug_str, "migration_metadata:")?;
        let payer = self.extract_field(debug_str, "payer:")?;

        Some(MeteoraEvent::MigrationMetadataCreated {
            timestamp,
            virtual_pool,
            config,
            migration_metadata,
            payer,
        })
    }

    fn extract_locker_created(&self, debug_str: &str) -> Option<MeteoraEvent> {
        if !debug_str.contains("CreateLocker(") {
            return None;
        }

        let timestamp = Self::get_timestamp();
        let virtual_pool = self.extract_field(debug_str, "virtual_pool:")?;
        let config = self.extract_field(debug_str, "config:")?;
        let base_mint = self.extract_field(debug_str, "base_mint:")?;
        let creator = self.extract_field(debug_str, "creator:")?;
        let escrow = self.extract_field(debug_str, "escrow:")?;
        let escrow_token = self.extract_field(debug_str, "escrow_token:")?;

        Some(MeteoraEvent::LockerCreated {
            timestamp,
            virtual_pool,
            config,
            base_mint,
            creator,
            escrow,
            escrow_token,
        })
    }

    fn extract_migration_damm_v2(&self, debug_str: &str) -> Option<MeteoraEvent> {
        if !debug_str.contains("MigrationDammV2(") {
            return None;
        }

        let timestamp = Self::get_timestamp();
        let virtual_pool = self.extract_field(debug_str, "virtual_pool:")?;
        let config = self.extract_field(debug_str, "config:")?;
        let pool = self.extract_field(debug_str, "pool:")?;
        let base_mint = self.extract_field(debug_str, "base_mint:")?;
        let quote_mint = self.extract_field(debug_str, "quote_mint:")?;
        let payer = self.extract_field(debug_str, "payer:")?;
        let first_position_nft_mint = self.extract_field(debug_str, "first_position_nft_mint:")?;

        // Second position NFT mint is optional
        let second_position_nft_mint = if debug_str.contains("second_position_nft_mint: Some(") {
            // Extract the value inside Some(...)
            if let Some(start) = debug_str.find("second_position_nft_mint: Some(") {
                let after_some = &debug_str[start + "second_position_nft_mint: Some(".len()..];
                if let Some(end) = after_some.find(")") {
                    Some(after_some[..end].trim().to_string())
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        };

        Some(MeteoraEvent::MigrationDammV2 {
            timestamp,
            virtual_pool,
            config,
            pool,
            base_mint,
            quote_mint,
            payer,
            first_position_nft_mint,
            second_position_nft_mint,
        })
    }
}
