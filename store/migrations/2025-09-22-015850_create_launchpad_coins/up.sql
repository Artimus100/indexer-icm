CREATE TABLE launchpad_coins (
    id SERIAL PRIMARY KEY,
    mint TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    symbol TEXT NOT NULL,
    description TEXT,
    image_uri TEXT,
    metadata_uri TEXT,
    twitter TEXT,
    telegram TEXT,
    website TEXT,
    pool_address TEXT NOT NULL,
    creator TEXT NOT NULL,
    created_timestamp TIMESTAMPTZ NOT NULL,
    complete BOOLEAN NOT NULL DEFAULT FALSE,
    virtual_sol_reserves BIGINT NOT NULL DEFAULT 0,
    virtual_token_reserves BIGINT NOT NULL DEFAULT 0,
    total_supply BIGINT NOT NULL DEFAULT 1000000000,
    market_cap NUMERIC(20, 8),
    usd_market_cap NUMERIC(20, 2),
    reply_count INTEGER NOT NULL DEFAULT 0,
    is_banned BOOLEAN NOT NULL DEFAULT FALSE,
    program TEXT NOT NULL DEFAULT 'dbc',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create indexes for performance
CREATE INDEX idx_launchpad_coins_market_cap ON launchpad_coins(market_cap DESC);
CREATE INDEX idx_launchpad_coins_created_timestamp ON launchpad_coins(created_timestamp DESC);
CREATE INDEX idx_launchpad_coins_creator ON launchpad_coins(creator);
CREATE INDEX idx_launchpad_coins_complete ON launchpad_coins(complete);
CREATE INDEX idx_launchpad_coins_program ON launchpad_coins(program);
CREATE INDEX idx_launchpad_coins_pool_address ON launchpad_coins(pool_address);

-- Create a function to automatically update the updated_at column
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

-- Create trigger to automatically update updated_at
CREATE TRIGGER update_launchpad_coins_updated_at 
    BEFORE UPDATE ON launchpad_coins 
    FOR EACH ROW 
    EXECUTE FUNCTION update_updated_at_column();