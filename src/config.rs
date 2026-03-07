use crate::error::{PolymarketBotError, Result};
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    // Wallet & Keys
    pub poly_private_key: String,
    pub wallet_type: String,  // "EOA" (MetaMask), "SAFE" (Gnosis Safe), "PROXY" (Magic Link)

    // API URLs
    pub poly_clob_url: String,
    pub poly_gamma_url: String,
    pub poly_data_url: String,
    pub poly_ws_url: String,
    pub poly_rpc_url: String,

    // Chain
    pub poly_chain_id: u32,

    // Database
    pub database_url: String,
    pub database_pool_size: u32,

    // Paper Trading
    pub paper_trading: bool,

    // Risk Management
    pub max_position_size_usd: f64,
    pub max_daily_exposure_usd: f64,
    pub max_slippage_tolerance: f64,
    pub stop_loss_threshold: f64,

    // Strategy
    pub min_combined_price: f64,
    pub arbitrage_buffer: f64,
    pub dedup_window_secs: u64,

    // Execution
    pub order_timeout_secs: u64,
    pub heartbeat_interval_secs: u64,
    pub websocket_reconnect_max_secs: u64,

    // Market Scanning
    pub market_limit: u32,
    pub scan_interval_ms: u64,
    pub market_category: String,  // "sports" (default), "crypto", "political", "all"

    // API Rate Limiting
    pub gamma_api_rps: u32,
    pub clob_api_rps: u32,

    // Logging
    pub rust_log: String,
    pub log_file: String,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        dotenv::dotenv().ok();

        Ok(Config {
            poly_private_key: env::var("POLY_PRIVATE_KEY")
                .map_err(|_| PolymarketBotError::ConfigError("POLY_PRIVATE_KEY not set".to_string()))?,

            wallet_type: env::var("WALLET_TYPE")
                .unwrap_or_else(|_| "EOA".to_string()),  // Default to MetaMask (EOA)

            poly_clob_url: env::var("POLY_CLOB_URL")
                .unwrap_or_else(|_| "https://clob.polymarket.com".to_string()),

            poly_gamma_url: env::var("POLY_GAMMA_URL")
                .unwrap_or_else(|_| "https://gamma-api.polymarket.com".to_string()),

            poly_data_url: env::var("POLY_DATA_URL")
                .unwrap_or_else(|_| "https://data-api.polymarket.com".to_string()),

            poly_ws_url: env::var("POLY_WS_URL")
                .unwrap_or_else(|_| "wss://ws-subscriptions-clob.polymarket.com/ws/market".to_string()),

            poly_rpc_url: env::var("POLY_RPC_URL")
                .unwrap_or_else(|_| "https://polygon-rpc.com".to_string()),

            poly_chain_id: env::var("POLY_CHAIN_ID")
                .unwrap_or_else(|_| "137".to_string())
                .parse()
                .map_err(|_| PolymarketBotError::ConfigError("Invalid POLY_CHAIN_ID".to_string()))?,

            database_url: env::var("DATABASE_URL")
                .map_err(|_| PolymarketBotError::ConfigError("DATABASE_URL not set".to_string()))?,

            database_pool_size: env::var("DATABASE_POOL_SIZE")
                .unwrap_or_else(|_| "10".to_string())
                .parse()
                .unwrap_or(10),

            paper_trading: env::var("PAPER_TRADING")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),

            max_position_size_usd: env::var("MAX_POSITION_SIZE_USD")
                .unwrap_or_else(|_| "1000".to_string())
                .parse()
                .unwrap_or(1000.0),

            max_daily_exposure_usd: env::var("MAX_DAILY_EXPOSURE_USD")
                .unwrap_or_else(|_| "20000".to_string())
                .parse()
                .unwrap_or(20000.0),

            max_slippage_tolerance: env::var("MAX_SLIPPAGE_TOLERANCE")
                .unwrap_or_else(|_| "0.02".to_string())
                .parse()
                .unwrap_or(0.02),

            stop_loss_threshold: env::var("STOP_LOSS_THRESHOLD")
                .unwrap_or_else(|_| "-100".to_string())
                .parse()
                .unwrap_or(-100.0),

            min_combined_price: env::var("MIN_COMBINED_PRICE")
                .unwrap_or_else(|_| "0.98".to_string())
                .parse()
                .unwrap_or(0.98),

            arbitrage_buffer: env::var("ARBITRAGE_BUFFER")
                .unwrap_or_else(|_| "0.02".to_string())
                .parse()
                .unwrap_or(0.02),

            dedup_window_secs: env::var("DEDUP_WINDOW_SECS")
                .unwrap_or_else(|_| "5".to_string())
                .parse()
                .unwrap_or(5),

            order_timeout_secs: env::var("ORDER_TIMEOUT_SECS")
                .unwrap_or_else(|_| "5".to_string())
                .parse()
                .unwrap_or(5),

            heartbeat_interval_secs: env::var("HEARTBEAT_INTERVAL_SECS")
                .unwrap_or_else(|_| "10".to_string())
                .parse()
                .unwrap_or(10),

            websocket_reconnect_max_secs: env::var("WEBSOCKET_RECONNECT_MAX_SECS")
                .unwrap_or_else(|_| "30".to_string())
                .parse()
                .unwrap_or(30),

            market_limit: env::var("MARKET_LIMIT")
                .unwrap_or_else(|_| "100".to_string())
                .parse()
                .unwrap_or(100),

            scan_interval_ms: env::var("SCAN_INTERVAL_MS")
                .unwrap_or_else(|_| "500".to_string())
                .parse()
                .unwrap_or(500),

            market_category: env::var("MARKET_CATEGORY")
                .unwrap_or_else(|_| "sports".to_string())
                .to_lowercase(),

            gamma_api_rps: env::var("GAMMA_API_RPS")
                .unwrap_or_else(|_| "2".to_string())
                .parse()
                .unwrap_or(2),

            clob_api_rps: env::var("CLOB_API_RPS")
                .unwrap_or_else(|_| "10".to_string())
                .parse()
                .unwrap_or(10),

            rust_log: env::var("RUST_LOG")
                .unwrap_or_else(|_| "info,polymarket_bot=debug".to_string()),

            log_file: env::var("LOG_FILE")
                .unwrap_or_else(|_| "./logs/bot.log".to_string()),
        })
    }
}
