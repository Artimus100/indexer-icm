-- Create holders table for tracking token holders and their balances
CREATE TABLE holders (
    id SERIAL PRIMARY KEY,
    mint TEXT NOT NULL,
    wallet_address TEXT NOT NULL,
    amount NUMERIC NOT NULL DEFAULT 0,
    sol_balance NUMERIC NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create indexes for efficient querying
CREATE INDEX idx_holders_mint ON holders (mint);
CREATE INDEX idx_holders_amount ON holders (mint, amount DESC);
CREATE INDEX idx_holders_sol_balance ON holders (mint, sol_balance DESC);
CREATE INDEX idx_holders_wallet_address ON holders (wallet_address);

-- Add unique constraint for mint + wallet_address
ALTER TABLE holders ADD CONSTRAINT unique_mint_wallet UNIQUE (mint, wallet_address);

-- Create trigger to automatically update updated_at
CREATE TRIGGER update_holders_updated_at 
    BEFORE UPDATE ON holders 
    FOR EACH ROW 
    EXECUTE FUNCTION update_updated_at_column();