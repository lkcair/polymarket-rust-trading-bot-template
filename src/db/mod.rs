use crate::error::Result;
use sqlx::postgres::{PgPool, PgPoolOptions};
use tracing::info;

/// Initialize database pool and run migrations
pub async fn init_db(database_url: &str, pool_size: u32) -> Result<PgPool> {
    info!("Initializing database connection pool");

    let pool = PgPoolOptions::new()
        .max_connections(pool_size as u32)
        .connect(database_url)
        .await?;

    info!(
        "Database pool initialized with max {} connections",
        pool_size
    );

    Ok(pool)
}

/// Log arbitrage opportunity to database
pub async fn log_opportunity(
    pool: &PgPool,
    market_id_a: &str,
    market_id_b: &str,
    arb_type: &str,
    combined_price: f64,
    spread: f64,
) -> Result<()> {
    sqlx::query(
        "INSERT INTO opportunities (market_id_a, market_id_b, arb_type, combined_price, spread)
         VALUES ($1, $2, $3, $4, $5)"
    )
    .bind(market_id_a)
    .bind(market_id_b)
    .bind(arb_type)
    .bind(combined_price)
    .bind(spread)
    .execute(pool)
    .await?;

    Ok(())
}

/// Log order to database
pub async fn log_order(
    pool: &PgPool,
    market_id_a: &str,
    market_id_b: &str,
    quantity_a: f64,
    quantity_b: f64,
    price_a: f64,
    price_b: f64,
    cost_usd: f64,
    paper_trade: bool,
) -> Result<()> {
    sqlx::query(
        "INSERT INTO orders (market_id_a, market_id_b, quantity_a, quantity_b, price_a, price_b, cost_usd, paper_trade)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8)"
    )
    .bind(market_id_a)
    .bind(market_id_b)
    .bind(quantity_a)
    .bind(quantity_b)
    .bind(price_a)
    .bind(price_b)
    .bind(cost_usd)
    .bind(paper_trade)
    .execute(pool)
    .await?;

    Ok(())
}

/// Log trade event
pub async fn log_trade_event(
    pool: &PgPool,
    event_type: &str,
    market_id: &str,
    action: &str,
    price: f64,
    quantity: f64,
    status: &str,
) -> Result<()> {
    sqlx::query(
        "INSERT INTO trade_log (event_type, market_id, action, price, quantity, status)
         VALUES ($1, $2, $3, $4, $5, $6)"
    )
    .bind(event_type)
    .bind(market_id)
    .bind(action)
    .bind(price)
    .bind(quantity)
    .bind(status)
    .execute(pool)
    .await?;

    Ok(())
}

/// Get cumulative PnL
pub async fn get_cumulative_pnl(pool: &PgPool) -> Result<f64> {
    let row = sqlx::query_scalar::<_, Option<f64>>(
        "SELECT COALESCE(SUM(profit_loss), 0) FROM settlements"
    )
    .fetch_one(pool)
    .await?;

    Ok(row.unwrap_or(0.0))
}

/// Count total executed trades
pub async fn count_trades(pool: &PgPool) -> Result<i64> {
    let count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM orders WHERE status = 'filled'"
    )
    .fetch_one(pool)
    .await?;

    Ok(count.0)
}
