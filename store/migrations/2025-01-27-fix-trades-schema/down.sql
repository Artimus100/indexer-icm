-- Revert trades table schema changes
DROP TABLE IF EXISTS public.trades CASCADE;

-- Recreate original trades table
CREATE TABLE public.trades (
    id SERIAL PRIMARY KEY,
    base_mint VARCHAR NOT NULL,
    pool_id VARCHAR,
    timestamp TIMESTAMPTZ NOT NULL,
    price DOUBLE PRECISION NOT NULL,
    quantity DOUBLE PRECISION NOT NULL,
    volume DOUBLE PRECISION NOT NULL
);

-- Create index (TimescaleDB hypertable functionality removed for compatibility)
CREATE INDEX idx_trades_base_mint_timestamp ON public.trades (base_mint, timestamp DESC);
