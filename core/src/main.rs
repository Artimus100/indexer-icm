#![deny(
    clippy::disallowed_methods,
    clippy::suspicious,
    clippy::style,
    clippy::clone_on_ref_ptr
)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

use clap::Parser;
use std::str::FromStr;
use tracing::info;
use yellowstone_vixen::{
    self as vixen, Pipeline,
    filter_pipeline::FilterPipeline,
    vixen_core::{Prefilter, Pubkey},
};
use yellowstone_vixen_yellowstone_grpc_source::YellowstoneGrpcSource;

use crate::{logger::Logger, types::Opts};

mod logger;
mod redis;
mod types;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    rustls::crypto::aws_lc_rs::default_provider()
        .install_default()
        .expect("Failed to install default crypto provider");

    info!("DBC Launchpad Indexer started!");

    let Opts { config } = Opts::parse();
    let config_content = std::fs::read_to_string(config).expect("Error reading config file");
    let config = toml::from_str(&config_content).expect("Error parsing config");

    // Redis configuration - you can make these configurable
    let redis_url =
        std::env::var("REDIS_URL").unwrap_or_else(|_| "".to_string());
    let stream_key = std::env::var("REDIS_STREAM_KEY").unwrap_or_else(|_| "dbc:events".to_string());

    // Create logger with Redis client
    let logger =
        Logger::new(&redis_url, stream_key).expect("Failed to create logger with Redis client");

    vixen::Runtime::<YellowstoneGrpcSource>::builder()
        .account(Pipeline::new(
            yellowstone_vixen_meteora_dbc_parser::accounts_parser::AccountParser,
            [logger.clone()], // Clone for account handler
        ))
        .instruction(FilterPipeline::new(
            yellowstone_vixen_meteora_dbc_parser::instructions_parser::InstructionParser,
            [logger.clone()], // Use original for instruction handler
            Prefilter::builder().transaction_accounts_include([Pubkey::from_str(
                "28eYKBRnoVjVCHaJUeLKYzZyBJR3c5TG1UMGQccpSZgE",
            )
            .unwrap()]),
        ))
        .build(config)
        .run();

    Ok(())
}
