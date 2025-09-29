use crate::models::{BondingCurveStatsResponse, CoinResponse};
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use store::LaunchpadCoin;

pub fn format_decimal(decimal: Option<Decimal>) -> Option<String> {
    decimal.map(|d| format!("{:.6}", d))
}

pub fn format_currency(decimal: Option<Decimal>) -> Option<String> {
    decimal.map(|d| format!("${:.2}", d))
}

pub fn format_percentage(decimal: Option<Decimal>) -> Option<String> {
    decimal.map(|d| format!("{:.2}%", d))
}

pub fn calculate_price_per_token(sol_reserves: i64, token_reserves: i64) -> Option<Decimal> {
    if token_reserves > 0 {
        let sol_reserves_decimal = Decimal::from(sol_reserves) / Decimal::from(1_000_000_000i64);
        let token_reserves_decimal = Decimal::from(token_reserves);
        Some(sol_reserves_decimal / token_reserves_decimal)
    } else {
        None
    }
}

pub fn calculate_bonding_curve_progress(sol_reserves: i64) -> Option<Decimal> {
    // For pump.fun style tokens, graduation happens at 30 SOL
    let graduation_target_sol = Decimal::from(30i64);
    let sol_in_curve = Decimal::from(sol_reserves) / Decimal::from(1_000_000_000i64); // Convert lamports to SOL

    if sol_in_curve >= graduation_target_sol {
        Some(Decimal::from(100)) // 100% if already graduated
    } else {
        Some((sol_in_curve / graduation_target_sol) * Decimal::from(100))
    }
}

pub fn calculate_bonding_curve_stats(sol_reserves: i64) -> BondingCurveStatsResponse {
    let graduation_target_sol = Decimal::from(30i64);
    let sol_in_curve = Decimal::from(sol_reserves) / Decimal::from(1_000_000_000i64);
    let progress = if sol_in_curve >= graduation_target_sol {
        Decimal::from(100)
    } else {
        (sol_in_curve / graduation_target_sol) * Decimal::from(100)
    };

    BondingCurveStatsResponse {
        progress_percentage: progress.to_f64().unwrap_or(0.0),
        sol_in_curve: sol_in_curve.to_f64().unwrap_or(0.0),
        graduation_target: graduation_target_sol.to_f64().unwrap_or(30.0),
        is_graduated: progress >= Decimal::from(100),
    }
}

pub fn convert_coin_to_response(coin: LaunchpadCoin) -> CoinResponse {
    let price_per_token =
        calculate_price_per_token(coin.virtual_sol_reserves, coin.virtual_token_reserves);
    let bonding_curve_progress = calculate_bonding_curve_progress(coin.virtual_sol_reserves);
    let bonding_curve_stats = calculate_bonding_curve_stats(coin.virtual_sol_reserves);

    CoinResponse {
        id: coin.id,
        mint: coin.mint,
        name: coin.name,
        symbol: coin.symbol,
        description: coin.description,
        image_uri: coin.image_uri,
        metadata_uri: coin.metadata_uri,
        twitter: coin.twitter,
        telegram: coin.telegram,
        website: coin.website,
        pool_address: coin.pool_address,
        creator: coin.creator,
        created_timestamp: coin.created_timestamp.to_rfc3339(),
        complete: coin.complete,
        virtual_sol_reserves: coin.virtual_sol_reserves,
        virtual_token_reserves: coin.virtual_token_reserves,
        total_supply: coin.total_supply,
        market_cap: format_currency(coin.market_cap),
        usd_market_cap: format_currency(coin.usd_market_cap),
        reply_count: coin.reply_count,
        is_banned: coin.is_banned,
        program: coin.program,
        created_at: coin.created_at.to_rfc3339(),
        updated_at: coin.updated_at.to_rfc3339(),
        price_per_token: format_decimal(price_per_token),
        bonding_curve_progress: format_percentage(bonding_curve_progress),
        bonding_curve_stats: Some(bonding_curve_stats),
    }
}
