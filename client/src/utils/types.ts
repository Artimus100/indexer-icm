export interface KLine {
    close: string,
    end: string,
    high: string,
    low: string,
    open: string,
    quote_volume: string,
    start: string,
    trades: string,
    volume: string
}

export interface Trade {
    id: number;
    pool_id: string;
    timestamp: string;
    price: string;
    quantity: string;
    volume: string;
}

export interface Coin {
    id: number;
    mint: string;
    name: string;
    symbol: string;
    description?: string;
    image_uri?: string;
    metadata_uri?: string;
    twitter?: string;
    telegram?: string;
    website?: string;
    pool_address: string;
    creator: string;
    created_timestamp: string;
    complete: boolean;
    virtual_sol_reserves: number;
    virtual_token_reserves: number;
    total_supply: number;
    market_cap?: number;
    usd_market_cap?: number;
    reply_count: number;
    is_banned: boolean;
    program: string;
    created_at: string;
    updated_at: string;
}