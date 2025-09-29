// @generated automatically by Diesel CLI.

diesel::table! {
    holders (id) {
        id -> Int4,
        mint -> Text,
        wallet_address -> Text,
        amount -> Numeric,
        sol_balance -> Numeric,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    launchpad_coins (id) {
        id -> Int4,
        mint -> Text,
        name -> Text,
        symbol -> Text,
        description -> Nullable<Text>,
        image_uri -> Nullable<Text>,
        metadata_uri -> Nullable<Text>,
        twitter -> Nullable<Text>,
        telegram -> Nullable<Text>,
        website -> Nullable<Text>,
        pool_address -> Text,
        creator -> Text,
        created_timestamp -> Timestamptz,
        complete -> Bool,
        virtual_sol_reserves -> Int8,
        virtual_token_reserves -> Int8,
        total_supply -> Int8,
        market_cap -> Nullable<Numeric>,
        usd_market_cap -> Nullable<Numeric>,
        reply_count -> Int4,
        is_banned -> Bool,
        program -> Text,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    trades (id) {
        id -> Int4,
        base_mint -> Varchar,
        pool_id -> Nullable<Varchar>,
        timestamp -> Timestamptz,
        price_raw -> Int8,
        quantity_raw -> Int8,
        volume_raw -> Int8,
        price_decimal -> Nullable<Numeric>,
        quantity_decimal -> Nullable<Numeric>,
        volume_decimal -> Nullable<Numeric>,
    }
}

diesel::allow_tables_to_appear_in_same_query!(holders, launchpad_coins, trades,);
