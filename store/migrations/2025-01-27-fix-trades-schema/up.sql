-- Fix trades table schema to use proper integer types
-- Drop existing trades table and recreate with correct schema
DROP TABLE IF EXISTS public.trades CASCADE;

-- Create trades table with proper integer types
CREATE TABLE public.trades (
    id SERIAL PRIMARY KEY,
    base_mint VARCHAR NOT NULL,
    pool_id VARCHAR,
    timestamp TIMESTAMPTZ NOT NULL,
    price_raw BIGINT NOT NULL,  -- Price in smallest units (lamports)
    quantity_raw BIGINT NOT NULL,  -- Quantity in smallest units (token units)
    volume_raw BIGINT NOT NULL,  -- Volume in smallest units (lamports)
    price_decimal NUMERIC(20, 9),  -- Price as decimal for display
    quantity_decimal NUMERIC(20, 9),  -- Quantity as decimal for display
    volume_decimal NUMERIC(20, 9)  -- Volume as decimal for display
);

-- Create indexes (TimescaleDB hypertable functionality removed for compatibility)
CREATE INDEX idx_trades_base_mint_timestamp ON public.trades (base_mint, timestamp DESC);
CREATE INDEX idx_trades_pool_id_timestamp ON public.trades (pool_id, timestamp DESC);
