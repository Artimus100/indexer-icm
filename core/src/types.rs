use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(clap::Parser)]
#[command(version, author, about)]
pub struct Opts {
    #[arg(long, short)]
    pub config: PathBuf,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "event_type")]
pub enum MeteoraEvent {
    #[serde(rename = "pool_created")]
    PoolCreated {
        timestamp: u64,
        pool_id: String,
        creator: String,
        base_mint: String,
        quote_mint: String,
        name: String,
        symbol: String,
        uri: String,
        base_vault: String,
        quote_vault: String,
    },
    #[serde(rename = "swap")]
    Swap {
        timestamp: u64,
        pool_id: String,
        user: String,
        amount_in: u64,
        minimum_amount_out: u64,
        input_token_account: String,
        output_token_account: String,
        base_mint: String,
        quote_mint: String,
    },
    #[serde(rename = "metadata_created")]
    MetadataCreated {
        timestamp: u64,
        virtual_pool: String,
        pool_creator: String,
        partner: String,
    },
    #[serde(rename = "migration_metadata_created")]
    MigrationMetadataCreated {
        timestamp: u64,
        virtual_pool: String,
        config: String,
        migration_metadata: String,
        payer: String,
    },
    #[serde(rename = "locker_created")]
    LockerCreated {
        timestamp: u64,
        virtual_pool: String,
        config: String,
        base_mint: String,
        creator: String,
        escrow: String,
        escrow_token: String,
    },
    #[serde(rename = "migration_damm_v2")]
    MigrationDammV2 {
        timestamp: u64,
        virtual_pool: String,
        config: String,
        pool: String,
        base_mint: String,
        quote_mint: String,
        payer: String,
        first_position_nft_mint: String,
        second_position_nft_mint: Option<String>,
    },
}
