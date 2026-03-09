-- Polymarket Bot Database Schema

-- Markets table: track market metadata
CREATE TABLE IF NOT EXISTS markets (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    polymarket_id VARCHAR(255) NOT NULL UNIQUE,
    token_id_a VARCHAR(255) NOT NULL,
    token_id_b VARCHAR(255) NOT NULL,
    outcome_a VARCHAR(255) NOT NULL,
    outcome_b VARCHAR(255) NOT NULL,
    question TEXT NOT NULL,
    yes_price_a NUMERIC(10, 6),
    yes_price_b NUMERIC(10, 6),
    no_price_a NUMERIC(10, 6),
    no_price_b NUMERIC(10, 6),
    liquidity NUMERIC(18, 2),
    volume_24h NUMERIC(18, 2),
    last_updated TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Opportunities table: track detected arbitrage opportunities
CREATE TABLE IF NOT EXISTS opportunities (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    market_id_a VARCHAR(255) NOT NULL,
    market_id_b VARCHAR(255) NOT NULL,
    arb_type VARCHAR(10) NOT NULL CHECK (arb_type IN ('YES', 'NO')), -- YES arb or NO arb
    combined_price NUMERIC(10, 6) NOT NULL,
    spread NUMERIC(10, 6) NOT NULL,
    timestamp TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Orders table: track all placed orders
CREATE TABLE IF NOT EXISTS orders (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    market_id_a VARCHAR(255) NOT NULL,
    market_id_b VARCHAR(255) NOT NULL,
    order_id_a VARCHAR(255),
    order_id_b VARCHAR(255),
    quantity_a NUMERIC(18, 2) NOT NULL,
    quantity_b NUMERIC(18, 2) NOT NULL,
    price_a NUMERIC(10, 6) NOT NULL,
    price_b NUMERIC(10, 6) NOT NULL,
    cost_usd NUMERIC(18, 2) NOT NULL,
    status VARCHAR(50) DEFAULT 'pending',
    paper_trade BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    filled_at TIMESTAMP WITH TIME ZONE,
    resolved_at TIMESTAMP WITH TIME ZONE
);

-- Settlements table: track resolved trades and P&L
CREATE TABLE IF NOT EXISTS settlements (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    order_id UUID NOT NULL REFERENCES orders(id),
    market_id_a VARCHAR(255) NOT NULL,
    market_id_b VARCHAR(255) NOT NULL,
    winning_side VARCHAR(10) NOT NULL CHECK (winning_side IN ('A', 'B')),
    payout_a NUMERIC(18, 2),
    payout_b NUMERIC(18, 2),
    total_cost NUMERIC(18, 2) NOT NULL,
    profit_loss NUMERIC(18, 2) NOT NULL,
    resolved_at TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Trade log table: comprehensive execution history
CREATE TABLE IF NOT EXISTS trade_log (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    order_id UUID REFERENCES orders(id),
    event_type VARCHAR(50) NOT NULL,
    market_id VARCHAR(255),
    token_id VARCHAR(255),
    action VARCHAR(50),
    price NUMERIC(10, 6),
    quantity NUMERIC(18, 2),
    profit_loss NUMERIC(18, 2),
    cumulative_pnl NUMERIC(18, 2),
    status VARCHAR(50),
    error_message TEXT,
    timestamp TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Create indexes for performance
CREATE INDEX IF NOT EXISTS idx_opportunities_timestamp ON opportunities(timestamp);
CREATE INDEX IF NOT EXISTS idx_orders_created_at ON orders(created_at);
CREATE INDEX IF NOT EXISTS idx_orders_status ON orders(status);
CREATE INDEX IF NOT EXISTS idx_settlements_resolved_at ON settlements(resolved_at);
CREATE INDEX IF NOT EXISTS idx_trade_log_timestamp ON trade_log(timestamp);
CREATE INDEX IF NOT EXISTS idx_markets_token_id_a ON markets(token_id_a);
CREATE INDEX IF NOT EXISTS idx_markets_token_id_b ON markets(token_id_b);
