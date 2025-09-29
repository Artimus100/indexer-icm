use chrono::Utc;
use diesel::{
    Connection, ExpressionMethods, PgConnection, QueryDsl, RunQueryDsl, prelude::*,
    result::Error as DieselError, upsert::excluded,
};
use rust_decimal::Decimal;

use crate::config::Config;
use crate::models::*;

pub struct Store {
    pub conn: PgConnection,
}

impl Default for Store {
    fn default() -> Self {
        let database_url = Config::default().db_url;
        let connection =
            PgConnection::establish(&database_url).expect("Error connecting to the database");
        Self { conn: connection }
    }
}

impl Store {
    pub fn new(database_url: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let connection = PgConnection::establish(database_url)?;
        Ok(Self { conn: connection })
    }

    // ===== LAUNCHPAD COINS OPERATIONS =====

    // Create a new launchpad coin
    pub fn create_launchpad_coin(
        &mut self,
        new_coin: NewLaunchpadCoin,
    ) -> Result<LaunchpadCoin, DieselError> {
        use crate::schema::launchpad_coins::dsl::*;

        diesel::insert_into(launchpad_coins)
            .values(&new_coin)
            .get_result(&mut self.conn)
    }

    // Get a launchpad coin by mint address
    pub fn get_launchpad_coin_by_mint(
        &mut self,
        target_mint: &str,
    ) -> Result<Option<LaunchpadCoin>, DieselError> {
        use crate::schema::launchpad_coins::dsl::*;

        launchpad_coins
            .filter(mint.eq(target_mint))
            .first(&mut self.conn)
            .optional()
    }

    // Get a launchpad coin by pool address
    pub fn get_launchpad_coin_by_pool(
        &mut self,
        target_pool: &str,
    ) -> Result<Option<LaunchpadCoin>, DieselError> {
        use crate::schema::launchpad_coins::dsl::*;

        launchpad_coins
            .filter(pool_address.eq(target_pool))
            .first(&mut self.conn)
            .optional()
    }


    // Update a launchpad coin
    pub fn update_launchpad_coin(
        &mut self,
        target_mint: &str,
        updates: UpdateLaunchpadCoin,
    ) -> Result<(), DieselError> {
        use crate::schema::launchpad_coins::dsl::*;

        diesel::update(launchpad_coins.filter(mint.eq(target_mint)))
            .set(&updates)
            .execute(&mut self.conn)?;

        Ok(())
    }

    // Upsert a launchpad coin (insert or update)
    pub fn upsert_launchpad_coin(
        &mut self,
        new_coin: NewLaunchpadCoin,
    ) -> Result<LaunchpadCoin, DieselError> {
        use crate::schema::launchpad_coins::dsl::*;

        diesel::insert_into(launchpad_coins)
            .values(&new_coin)
            .on_conflict(mint)
            .do_update()
            .set((
                name.eq(excluded(name)),
                symbol.eq(excluded(symbol)),
                description.eq(excluded(description)),
                image_uri.eq(excluded(image_uri)),
                metadata_uri.eq(excluded(metadata_uri)),
                twitter.eq(excluded(twitter)),
                telegram.eq(excluded(telegram)),
                website.eq(excluded(website)),
                pool_address.eq(excluded(pool_address)),
                creator.eq(excluded(creator)),
                created_timestamp.eq(excluded(created_timestamp)),
                complete.eq(excluded(complete)),
                virtual_sol_reserves.eq(excluded(virtual_sol_reserves)),
                virtual_token_reserves.eq(excluded(virtual_token_reserves)),
                total_supply.eq(excluded(total_supply)),
                market_cap.eq(excluded(market_cap)),
                usd_market_cap.eq(excluded(usd_market_cap)),
                reply_count.eq(excluded(reply_count)),
                is_banned.eq(excluded(is_banned)),
                program.eq(excluded(program)),
                updated_at.eq(Utc::now()),
            ))
            .get_result(&mut self.conn)
    }

    // Get trending coins (ordered by market cap)
    pub fn get_trending_coins(&mut self, limit: i64) -> Result<Vec<LaunchpadCoin>, DieselError> {
        use crate::schema::launchpad_coins::dsl::*;

        launchpad_coins
            .filter(complete.eq(false))
            .filter(is_banned.eq(false))
            .filter(market_cap.is_not_null())
            .order(market_cap.desc())
            .limit(limit)
            .load(&mut self.conn)
    }

    // Get recent coins (ordered by creation time)
    pub fn get_recent_coins(&mut self, limit: i64) -> Result<Vec<LaunchpadCoin>, DieselError> {
        use crate::schema::launchpad_coins::dsl::*;

        launchpad_coins
            .filter(complete.eq(false))
            .filter(is_banned.eq(false))
            .order(created_timestamp.desc())
            .limit(limit)
            .load(&mut self.conn)
    }

    // Get coins by creator
    pub fn get_coins_by_creator(
        &mut self,
        target_creator: &str,
        limit: i64,
    ) -> Result<Vec<LaunchpadCoin>, DieselError> {
        use crate::schema::launchpad_coins::dsl::*;

        launchpad_coins
            .filter(creator.eq(target_creator))
            .filter(is_banned.eq(false))
            .order(created_timestamp.desc())
            .limit(limit)
            .load(&mut self.conn)
    }

    // Search coins by name or symbol
    pub fn search_coins(
        &mut self,
        query: &str,
        limit: i64,
    ) -> Result<Vec<LaunchpadCoin>, DieselError> {
        use crate::schema::launchpad_coins::dsl::*;

        let search_pattern = format!("%{}%", query);

        launchpad_coins
            .filter(
                name.ilike(&search_pattern)
                    .or(symbol.ilike(&search_pattern)),
            )
            .filter(complete.eq(false))
            .filter(is_banned.eq(false))
            .order(created_timestamp.desc())
            .limit(limit)
            .load(&mut self.conn)
    }

    // Get all coins with pagination
    pub fn get_all_coins(
        &mut self,
        offset: i64,
        limit: i64,
    ) -> Result<Vec<LaunchpadCoin>, DieselError> {
        use crate::schema::launchpad_coins::dsl::*;

        launchpad_coins
            .filter(is_banned.eq(false))
            .order(created_timestamp.desc())
            .offset(offset)
            .limit(limit)
            .load(&mut self.conn)
    }

    // Update coin reserves and market cap
    pub fn update_coin_reserves(
        &mut self,
        target_mint: &str,
        sol_reserves: i64,
        token_reserves: i64,
    ) -> Result<(), DieselError> {
        use crate::schema::launchpad_coins::dsl::*;

        // Calculate new market cap
        let calculated_market_cap = if token_reserves > 0 {
            let sol_reserves_decimal =
                Decimal::from(sol_reserves) / Decimal::from(1_000_000_000i64);
            let token_reserves_decimal = Decimal::from(token_reserves);
            let price_per_token = sol_reserves_decimal / token_reserves_decimal;
            let total_supply_decimal = Decimal::from(1_000_000_000i64); // Default total supply
            Some(price_per_token * total_supply_decimal)
        } else {
            None
        };

        diesel::update(launchpad_coins.filter(mint.eq(target_mint)))
            .set((
                virtual_sol_reserves.eq(sol_reserves),
                virtual_token_reserves.eq(token_reserves),
                market_cap.eq(calculated_market_cap),
                updated_at.eq(Utc::now()),
            ))
            .execute(&mut self.conn)?;

        Ok(())
    }

    // Mark coin as complete (migrated)
    pub fn mark_coin_complete(&mut self, target_mint: &str) -> Result<(), DieselError> {
        use crate::schema::launchpad_coins::dsl::*;

        diesel::update(launchpad_coins.filter(mint.eq(target_mint)))
            .set((complete.eq(true), updated_at.eq(Utc::now())))
            .execute(&mut self.conn)?;

        Ok(())
    }

    // Ban/unban a coin
    pub fn set_coin_banned(&mut self, target_mint: &str, banned: bool) -> Result<(), DieselError> {
        use crate::schema::launchpad_coins::dsl::*;

        diesel::update(launchpad_coins.filter(mint.eq(target_mint)))
            .set((is_banned.eq(banned), updated_at.eq(Utc::now())))
            .execute(&mut self.conn)?;

        Ok(())
    }

    // ===== HOLDERS OPERATIONS =====

    // Create a new holder
    pub fn create_holder(&mut self, new_holder: NewHolder) -> Result<Holder, DieselError> {
        use crate::schema::holders::dsl::*;

        diesel::insert_into(holders)
            .values(&new_holder)
            .get_result(&mut self.conn)
    }

    // Upsert a holder (insert or update)
    pub fn upsert_holder(&mut self, holder: NewHolder) -> Result<Holder, DieselError> {
        use crate::schema::holders::dsl::*;

        diesel::insert_into(holders)
            .values(&holder)
            .on_conflict((mint, wallet_address))
            .do_update()
            .set((
                amount.eq(excluded(amount)),
                sol_balance.eq(excluded(sol_balance)),
                updated_at.eq(Utc::now()),
            ))
            .get_result(&mut self.conn)
    }

    // Get top holders for a specific mint
    pub fn get_top_holders(
        &mut self,
        target_mint: &str,
        limit: i64,
    ) -> Result<Vec<Holder>, DieselError> {
        use crate::schema::holders::dsl::*;

        holders
            .filter(mint.eq(target_mint))
            .order_by(amount.desc())
            .limit(limit)
            .load(&mut self.conn)
    }

    // Get all holders for a specific mint
    pub fn get_all_holders(&mut self, target_mint: &str) -> Result<Vec<Holder>, DieselError> {
        use crate::schema::holders::dsl::*;

        holders
            .filter(mint.eq(target_mint))
            .order_by(amount.desc())
            .load(&mut self.conn)
    }

    // Get holder by mint and wallet address
    pub fn get_holder(
        &mut self,
        target_mint: &str,
        _wallet_address: &str,
    ) -> Result<Option<Holder>, DieselError> {
        use crate::schema::holders::dsl::*;

        holders
            .filter(mint.eq(target_mint))
            .filter(wallet_address.eq(_wallet_address))
            .first(&mut self.conn)
            .optional()
    }

    // Update holder
    pub fn update_holder(
        &mut self,
        target_mint: &str,
        _wallet_address: &str,
        update_data: UpdateHolder,
    ) -> Result<(), DieselError> {
        use crate::schema::holders::dsl::*;

        diesel::update(
            holders
                .filter(mint.eq(target_mint))
                .filter(wallet_address.eq(_wallet_address)),
        )
        .set(update_data)
        .execute(&mut self.conn)?;

        Ok(())
    }

    // Delete holder
    pub fn delete_holder(
        &mut self,
        target_mint: &str,
        _wallet_address: &str,
    ) -> Result<(), DieselError> {
        use crate::schema::holders::dsl::*;

        diesel::delete(
            holders
                .filter(mint.eq(target_mint))
                .filter(wallet_address.eq(_wallet_address)),
        )
        .execute(&mut self.conn)?;

        Ok(())
    }

    // ===== BONDING CURVE CALCULATIONS =====

    /// Calculate bonding curve progress for a coin
    /// Returns (progress_percentage, sol_in_curve, graduation_target)
    pub fn calculate_bonding_curve_progress(
        &mut self,
        target_mint: &str,
    ) -> Result<(Decimal, Decimal, Decimal), DieselError> {
        use crate::schema::launchpad_coins::dsl::*;

        // Get the coin data
        let coin = launchpad_coins
            .filter(mint.eq(target_mint))
            .first::<LaunchpadCoin>(&mut self.conn)?;

        // Calculate progress based on SOL in the bonding curve
        // For pump.fun style tokens, graduation typically happens at 30 SOL
        let graduation_target_sol = Decimal::from(30i64);
        let sol_in_curve =
            Decimal::from(coin.virtual_sol_reserves) / Decimal::from(1_000_000_000i64); // Convert lamports to SOL

        let progress = if sol_in_curve >= graduation_target_sol {
            Decimal::from(100) // 100% if already graduated
        } else {
            (sol_in_curve / graduation_target_sol) * Decimal::from(100)
        };

        Ok((progress, sol_in_curve, graduation_target_sol))
    }

    /// Get bonding curve statistics for a coin
    pub fn get_bonding_curve_stats(
        &mut self,
        target_mint: &str,
    ) -> Result<BondingCurveStats, DieselError> {
        let (progress, sol_in_curve, graduation_target) =
            self.calculate_bonding_curve_progress(target_mint)?;

        Ok(BondingCurveStats {
            progress_percentage: progress,
            sol_in_curve,
            graduation_target,
            is_graduated: progress >= Decimal::from(100),
        })
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BondingCurveStats {
    pub progress_percentage: Decimal,
    pub sol_in_curve: Decimal,
    pub graduation_target: Decimal,
    pub is_graduated: bool,
}
