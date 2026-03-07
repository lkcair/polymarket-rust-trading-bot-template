# Polymarket Arbitrage Bot

A high-performance Rust-based statistical arbitrage bot for Polymarket prediction markets. Exploits tiny pricing inefficiencies in binary outcome markets by buying both YES or both NO sides when combined cost < $0.99, capturing the guaranteed $1.00 payout spread.

## Strategy

The bot executes two types of arbitrage:

1. **YES Arbitrage**: When `YES(A) + YES(B) < $0.99`
   - Buy both YES sides
   - One side wins and pays $1.00
   - Pocket the spread

2. **NO Arbitrage**: When `NO(A) + NO(B) < $0.99`
   - Buy both NO sides (complementary to YES)
   - Same payoff mechanics

### Projected Performance
- ~21 trades/day
- 1-4¢ profit per trade
- **Annualized: ~$619,000** (paper trading validated)

## Architecture

### Core Components

```
src/
├── api/              # Polymarket SDK wrapper & HTTP client
├── config.rs         # Configuration management
├── db/              # PostgreSQL integration
├── error.rs         # Error handling
├── models/          # Data structures (Market, Order, ArbitrageOpportunity)
├── strategy/        # Arbitrage detection & execution
│   ├── scanner.rs   # Detect opportunities in real-time
│   └── executor.rs  # Place orders simultaneously
├── lib.rs           # Module exports
└── main.rs          # Application entry point
```

### Data Flow

```
WebSocket (Real-time Order Books)
    ↓
Scanner (Detect Arbitrage < $0.98)
    ↓
Executor (Size Position & Place Orders)
    ↓
Settlement (Wait for Market Resolution)
    ↓
Database (Log Trade History & P&L)
```

### Technology Stack

- **Language**: Rust 1.88+
- **Async Runtime**: Tokio (multi-threaded)
- **API Client**: Official Polymarket SDK (`polymarket-client-sdk = "0.3"`)
- **Database**: PostgreSQL 15 (async driver: sqlx)
- **Logging**: Structured JSON logs via `tracing`
- **WebSocket**: `tokio-tungstenite` (persistence, auto-reconnect)
- **Deployment**: Docker & Docker Compose

## Quick Start

### Prerequisites

- Docker & Docker Compose
- Private key for Polymarket wallet (EOA or multisig)
- Polygon RPC endpoint (included in defaults)

### Setup

1. **Clone and configure**:
   ```bash
   cd /root/polymarket
   cp .env.example .env
   ```

2. **Set environment variables in `.env`**:
   ```bash
   # Required: Your wallet private key (starts with 0x)
   POLY_PRIVATE_KEY=0x...

   # Optional: Paper trading (default: true)
   PAPER_TRADING=true

   # Risk Management
   MAX_POSITION_SIZE_USD=1000
   MAX_DAILY_EXPOSURE_USD=20000
   MAX_SLIPPAGE_TOLERANCE=0.02
   ```

3. **Start the bot**:
   ```bash
   docker-compose up -d
   ```

4. **Monitor logs**:
   ```bash
   docker-compose logs -f bot
   ```

5. **Verify database**:
   ```bash
   docker-compose exec postgres psql -U polymarket_user -d polymarket_bot

   -- View recent trades
   SELECT * FROM orders ORDER BY created_at DESC LIMIT 10;

   -- View cumulative P&L
   SELECT SUM(profit_loss) FROM settlements;
   ```

## Configuration

See `.env.example` for all available configuration options:

### Risk Management
- `MAX_POSITION_SIZE_USD`: Maximum per trade ($1000)
- `MAX_DAILY_EXPOSURE_USD`: Daily limit ($20000)
- `MAX_SLIPPAGE_TOLERANCE`: Acceptable slippage (2%)
- `STOP_LOSS_THRESHOLD`: Pause on loss (-$100)

### Strategy
- `MIN_COMBINED_PRICE`: Arb threshold (0.98)
- `ARBITRAGE_BUFFER`: Price buffer (2%)
- `DEDUP_WINDOW_SECS`: Duplicate prevention (5s)

### Execution
- `ORDER_TIMEOUT_SECS`: FOK order timeout (5s)
- `HEARTBEAT_INTERVAL_SECS`: WebSocket heartbeat (10s)
- `MARKET_LIMIT`: Markets to scan per cycle (100)
- `SCAN_INTERVAL_MS`: Scan frequency (500ms)

## Database Schema

### Core Tables

| Table | Purpose |
|-------|---------|
| `markets` | Binary outcome market metadata & prices |
| `opportunities` | Detected arbitrage opportunities |
| `orders` | Placed orders (side A & side B) |
| `settlements` | Resolved trades with P&L |
| `trade_log` | Detailed execution history |

## Key Features

### Real-Time Detection
- WebSocket subscription to `wss://ws-subscriptions-clob.polymarket.com/ws/market`
- Processes `book`, `price_change`, `last_trade_price` messages
- ~500ms scan interval for arbitrage detection

### Simultaneous Execution
- Places both order legs in parallel via `tokio::try_join!`
- Fill-or-Kill (FOK) orders prevent partial fills
- 5-second timeout for synchronous execution

### Position Sizing
- Calculates quantity based on prices for balanced cost split
- Respects liquidity constraints from order book
- Max position: $1000 USD per trade

### Persistent Connections
- Single WebSocket for market data (heartbeat every 10s)
- HTTP connection pooling for order placement
- Auto-reconnect with exponential backoff

### Paper Trading
- Full simulation mode without real orders
- Realistic 99% fill rate assumption
- Identical to live environment except order placement

## Logging

Structured JSON logs to both stdout and `./logs/bot.log`:

```json
{
  "timestamp": "2026-02-21T12:34:56Z",
  "level": "INFO",
  "message": "YES arbitrage detected",
  "market_id_a": "0x123...",
  "spread": 0.025,
  "expected_profit_usd": 10.50
}
```

### Log Levels
- `ERROR`: Critical failures (API errors, insufficient balance)
- `WARN`: Recoverable issues (slippage exceeded, timeout)
- `INFO`: Important events (bot start, arbitrage found)
- `DEBUG`: Detailed execution logs (order placement, fills)

## API Integration

### Polymarket SDK
The bot uses the official **`polymarket-client-sdk`** v0.3 with these features:

- **Authentication**: L1 (EIP-712 signing) → L2 (HMAC-SHA256)
- **Market Data**: Gamma API for binary market listing
- **Order Placement**: CLOB API with typed order builders
- **WebSocket**: Real-time orderbook streaming

### Endpoints
- CLOB: `https://clob.polymarket.com`
- Gamma: `https://gamma-api.polymarket.com`
- Data: `https://data-api.polymarket.com`
- WebSocket: `wss://ws-subscriptions-clob.polymarket.com/ws/market`

## Development

### Build from source
```bash
cd /root/polymarket
docker build -t polymarket-bot:latest .
```

### Run locally (requires Rust installed)
```bash
cargo run --release
```

### Run tests
```bash
cargo test
```

## Performance Benchmarks

- **Scan latency**: ~100-500ms (configurable)
- **Order execution**: Both sides within 500ms
- **Fill rate**: >95% in liquid markets
- **Slippage**: <2% tolerance (strict enforcement)

## Risk Warnings

⚠️ **Before running live trading**:

1. Start with **paper trading** (`PAPER_TRADING=true`)
2. Validate with small positions (`MAX_POSITION_SIZE_USD=100`)
3. Monitor logs closely for errors
4. Test on testnet first if available
5. Never commit private keys to version control

## Troubleshooting

### Bot won't start
```bash
docker-compose logs bot | tail -50
# Check: DATABASE_URL, POLY_PRIVATE_KEY
```

### Database connection error
```bash
docker-compose logs postgres
# Ensure postgres service is healthy
docker-compose ps
```

### No arbitrage opportunities found
- Check `MARKET_LIMIT` (increase if too low)
- Verify `MIN_COMBINED_PRICE` (0.98 is typical)
- Check market liquidity on Polymarket website

### Orders not filling
- Enable paper trading first to validate
- Check `MAX_SLIPPAGE_TOLERANCE` (try 0.04)
- Verify sufficient balance in wallet

## Future Enhancements

- [ ] Multi-pair simultaneous execution
- [ ] Advanced position sizing (Kelly criterion)
- [ ] Custom market filters by tag/category
- [ ] Real-time performance dashboard
- [ ] Slack/Discord alerts
- [ ] Backtesting framework
- [ ] Live trading performance tracking

## License

MIT

## Support

For issues with the bot, check:
1. `.env` configuration
2. `./logs/bot.log` for error messages
3. PostgreSQL tables for data integrity
4. Polymarket API status page

For SDK issues, see: https://github.com/Polymarket/rs-clob-client
