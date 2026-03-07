use thiserror::Error;

#[derive(Error, Debug)]
pub enum PolymarketBotError {
    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),

    #[error("API error: {0}")]
    ApiError(String),

    #[error("Authentication error: {0}")]
    AuthError(String),

    #[error("Order execution error: {0}")]
    OrderError(String),

    #[error("WebSocket error: {0}")]
    WebSocketError(String),

    #[error("Connection error: {0}")]
    ConnectionError(String),

    #[error("Strategy error: {0}")]
    StrategyError(String),

    #[error("Insufficient balance")]
    InsufficientBalance,

    #[error("Order timeout")]
    OrderTimeout,

    #[error("Slippage exceeded: {0}")]
    SlippageExceeded(f64),

    #[error("Market not found: {0}")]
    MarketNotFound(String),

    #[error("Invalid order state")]
    InvalidOrderState,

    #[error("JSON serialization error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("Parsing error: {0}")]
    ParseError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Unknown error: {0}")]
    Unknown(String),
}

pub type Result<T> = std::result::Result<T, PolymarketBotError>;
