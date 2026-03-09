# Polymarket Bot Template

A high-performance Rust-based trading bot template for Polymarket prediction markets. Includes a complete arbitrage strategy implementation as a reference example.

## Example Strategy: Statistical Arbitrage

The included example strategy detects pricing inefficiencies in binary outcome markets:

**Arbitrage Detection**: When `Outcome_A_Price + Outcome_B_Price < $1.00`
- Buy both outcomes simultaneously
- One outcome resolves to $1.00, the other to $0.00
- Theoretical profit = $1.00 - (Price_A + Price_B)

**Example Scenario**:
- Market: "Will it rain tomorrow?"
- YES price: $0.48
- NO price: $0.49  
- Combined: $0.97 < $1.00
- Theoretical spread: $0.03 per $1.00 invested

**⚠️ DISCLAIMER**: This is an educational example. Real-world trading involves:
- Slippage and execution delays
- Gas fees and transaction costs
- Market liquidity constraints
- Competition from other traders
- No guaranteed profits

## Architecture

### Core Components

```
src/
├── api/              # Polymarket SDK wrapper & HTTP client
├── config.rs         # Configuration management
├── db/              # PostgreSQL integration
├── error.rs         # Error handling
├── models/          # Data structures (Market, Order, Opportunity)
├── strategy/        # Trading strategy implementation
│   ├── scanner.rs   # Market scanning & opportunity detection
│   └── executor.rs  # Order placement & execution
├── lib.rs           # Module exports
└── main.rs          # Application entry point
```

### Data Flow

```
WebSocket (Real-time Order Books)
    ↓
Scanner (Detect Trading Opportunities)
    ↓
Executor (Size Position & Place Orders)
    ↓
Settlement (Track Fills & Log to Database)
    ↓
Database (Trade History & P&L)
```

### Technology Stack

- **Language**: Rust 1.88+
- **Async Runtime**: Tokio (multi-threaded)
- **API Client**: Official Polymarket SDK (`polymarket-client-sdk = "0.3"`)
- **Database**: PostgreSQL 15 (async driver: sqlx)
- **Logging**: Structured JSON logs via `tracing`
- **WebSocket**: `tokio-tungstenite` (persistence, auto-reconnect)
- **Deployment**: Docker & Docker Compose

## Customizing the Strategy

The arbitrage strategy in `src/strategy/` serves as a reference implementation. To build your own:

1. **Modify `scanner.rs`**: Implement your market analysis logic
2. **Modify `executor.rs`**: Customize order placement and position sizing
3. **Update `models/`**: Add data structures for your strategy
4. **Configure via `.env`**: Adjust thresholds and parameters

**Example strategies you could implement**:
- Market making (provide liquidity, capture spread)
- Trend following (momentum-based trading)
- Event-driven (news/social sentiment)
- Portfolio rebalancing (maintain target allocations)

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
- `MIN_COMBINED_PRICE`: Arbitrage threshold (example: 0.98)
- `ARBITRAGE_BUFFER`: Safety buffer percentage (example: 2%)
- `DEDUP_WINDOW_SECS`: Duplicate opportunity prevention (default: 5s)

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
| `opportunities` | Detected trading opportunities |
| `orders` | Placed orders (side A & side B) |
| `settlements` | Resolved trades with P&L |
| `trade_log` | Detailed execution history |

## Key Features

### Real-Time Detection
- WebSocket subscription to Polymarket's CLOB orderbook feed
- Processes `book`, `price_change`, `last_trade_price` messages
- Configurable scan interval (default: 500ms)

### Simultaneous Execution
- Places both order legs in parallel via async execution
- Fill-or-Kill (FOK) orders prevent partial fills
- Configurable timeout for synchronous execution (default: 5s)

### Position Sizing
- Calculates quantity based on prices for balanced cost allocation
- Respects liquidity constraints from order book depth
- Configurable maximum position size per trade

### Persistent Connections
- Single WebSocket connection for market data with heartbeat
- HTTP connection pooling for order placement
- Auto-reconnect with exponential backoff on disconnection

### Paper Trading
- Full simulation mode without placing real orders
- Simulates realistic fill rates for validation
- Identical to live environment for testing strategies

## Logging

Structured JSON logs to both stdout and `./logs/bot.log`:

```json
{
  "timestamp": "2026-02-21T12:34:56Z",
  "level": "INFO",
  "message": "Arbitrage opportunity detected",
  "market_id": "0x123...",
  "combined_price": 0.975,
  "spread": 0.025
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

- **Scan latency**: Configurable (default: 100-500ms)
- **Order execution**: Both sides placed concurrently
- **Fill rate**: Depends on market liquidity and order size
- **Slippage**: Configurable tolerance with strict enforcement

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
