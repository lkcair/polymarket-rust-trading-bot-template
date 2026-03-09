use polymarket_bot::{api::PolymarketClient, config::Config, db, strategy::{Scanner, Executor}};
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{error, info};
use tracing_appender::rolling;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load configuration
    let config = match Config::from_env() {
        Ok(cfg) => cfg,
        Err(e) => {
            eprintln!("Failed to load configuration: {}", e);
            std::process::exit(1);
        }
    };

    // Initialize tracing/logging
    init_logging(&config)?;

    info!("Starting Polymarket Arbitrage Bot");
    info!("Paper Trading: {}", config.paper_trading);
    info!("Market Category: {}", config.market_category.to_uppercase());
    info!("Max Position Size: ${:.2}", config.max_position_size_usd);
    info!("Max Daily Exposure: ${:.2}", config.max_daily_exposure_usd);

    // Initialize database
    let pool = match db::init_db(&config.database_url, config.database_pool_size).await {
        Ok(p) => p,
        Err(e) => {
            error!("Failed to initialize database: {}", e);
            std::process::exit(1);
        }
    };

    let config = Arc::new(config);

    // Initialize API client
    let client = match PolymarketClient::new(config.clone()) {
        Ok(c) => Arc::new(c),
        Err(e) => {
            error!("Failed to initialize API client: {}", e);
            std::process::exit(1);
        }
    };

    // Authenticate with Polymarket
    if let Err(e) = client.authenticate().await {
        error!("Failed to authenticate: {}", e);
        std::process::exit(1);
    }

    // Create channels for inter-module communication
    let (tx_opportunities, rx_opportunities) = mpsc::channel(1000);
    let (tx_orders, rx_orders) = mpsc::channel(1000);

    // Spawn scanner task
    let scanner = Scanner::new(config.clone(), client.clone(), tx_opportunities);
    let scanner_handle = tokio::spawn(async move {
        if let Err(e) = scanner.run().await {
            error!("Scanner error: {}", e);
        }
    });

    // Spawn executor task
    let mut executor = Executor::new(config.clone(), client.clone(), rx_opportunities, tx_orders);
    let executor_handle = tokio::spawn(async move {
        if let Err(e) = executor.run().await {
            error!("Executor error: {}", e);
        }
    });

    // Spawn settlement task
    let pool_clone = pool.clone();
    let settlement_handle = tokio::spawn(async move {
        let mut rx = rx_orders;
        info!("Settlement task started");
        
        while let Some(order) = rx.recv().await {
            info!("Settlement: Processing order {} (paper: {})", order.id, order.paper_trade);
            
            // Log order to database
            if let Err(e) = db::log_order(
                &pool_clone,
                &order.market_id_a,
                &order.market_id_b,
                order.quantity_a,
                order.quantity_b,
                order.price_a,
                order.price_b,
                order.cost_usd,
                order.paper_trade,
            ).await {
                error!("Failed to log order to database: {}", e);
            }
            
            info!("Order logged: cost=${:.2}, expected_profit=${:.2}", 
                order.cost_usd, order.expected_profit());
        }
    });

    // Wait for all tasks
    let _ = tokio::join!(scanner_handle, executor_handle, settlement_handle);

    info!("Bot shutdown complete");
    Ok(())
}

fn init_logging(config: &Config) -> Result<(), Box<dyn std::error::Error>> {
    // Create a rolling file appender for logs
    let file_appender = rolling::daily("./logs", "bot.log");

    // Build the tracing subscriber with both stdout and file outputs
    tracing_subscriber::registry()
        .with(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new(&config.rust_log)),
        )
        .with(
            fmt::layer()
                .with_writer(std::io::stdout)
                .with_thread_ids(true)
                .with_target(true)
                .json(),
        )
        .with(
            fmt::layer()
                .with_writer(file_appender)
                .with_thread_ids(true)
                .with_target(true)
                .json(),
        )
        .init();

    info!("Logging initialized");
    Ok(())
}
