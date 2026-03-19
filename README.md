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

## Key Features

- **Real-Time Detection**: WebSocket subscription to Polymarket's CLOB orderbook feed with configurable scan interval
- **Simultaneous Execution**: Places both order legs in parallel via async execution with Fill-or-Kill (FOK) orders
- **Position Sizing**: Calculates quantity based on prices for balanced cost allocation, respects liquidity constraints
- **Persistent Connections**: Single WebSocket connection with heartbeat, HTTP connection pooling, auto-reconnect
- **Paper Trading**: Full simulation mode without placing real orders, simulates realistic fill rates

## Logging

Structured JSON logs to stdout and `./logs/bot.log`:

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

**Log Levels**: ERROR (critical failures), WARN (recoverable issues), INFO (important events), DEBUG (detailed execution)

## API Integration

Uses official **`polymarket-client-sdk`** v0.3:
- **Authentication**: L1 (EIP-712 signing) → L2 (HMAC-SHA256)
- **Market Data**: Gamma API for binary market listing
- **Order Placement**: CLOB API with typed order builders
- **WebSocket**: Real-time orderbook streaming

**Endpoints**: CLOB (`clob.polymarket.com`), Gamma (`gamma-api.polymarket.com`), Data (`data-api.polymarket.com`), WebSocket (`ws-subscriptions-clob.polymarket.com`)

## Performance

- **Scan latency**: Configurable (default: 100-500ms)
- **Order execution**: Both sides placed concurrently
- **Fill rate**: Depends on market liquidity and order size
- **Slippage**: Configurable tolerance with strict enforcement

## Risk Warnings

⚠️ **Before running live trading**:

1. Start with **paper trading** (`PAPER_TRADING=true`)
2. Validate with small positions (`MAX_POSITION_SIZE_USD=100`)
3. Monitor logs closely for errors
4. Never commit private keys to version control

## Troubleshooting

**Bot won't start**: Check `docker-compose logs bot | tail -50` for DATABASE_URL and POLY_PRIVATE_KEY errors

**Database connection error**: Verify postgres is healthy with `docker-compose ps`

**No opportunities found**: Increase `MARKET_LIMIT` or verify `MIN_COMBINED_PRICE` threshold

**Orders not filling**: Enable paper trading first, check `MAX_SLIPPAGE_TOLERANCE`, verify wallet balance

## License

MIT

## Support

For issues: Check `.env` configuration, `./logs/bot.log`, PostgreSQL tables, or Polymarket API status

For SDK issues: https://github.com/Polymarket/rs-clob-client
