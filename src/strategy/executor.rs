use crate::api::client::{OrderSide, PolymarketClient};
use crate::config::Config;
use crate::error::Result;
use crate::models::arbitrage::ArbitrageOpportunity;
use crate::models::order::Order;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};

/// Executes arbitrage orders once opportunities are detected
pub struct Executor {
    config: Arc<Config>,
    client: Arc<PolymarketClient>,
    // Channel to receive opportunities from scanner
    rx_opportunities: mpsc::Receiver<ArbitrageOpportunity>,
    // Channel to send executed orders for settlement
    tx_orders: mpsc::Sender<Order>,
}

impl Executor {
    pub fn new(
        config: Arc<Config>,
        client: Arc<PolymarketClient>,
        rx_opportunities: mpsc::Receiver<ArbitrageOpportunity>,
        tx_orders: mpsc::Sender<Order>,
    ) -> Self {
        Self {
            config,
            client,
            rx_opportunities,
            tx_orders,
        }
    }

    /// Start the execution loop
    pub async fn run(&mut self) -> Result<()> {
        info!("Starting order executor");

        while let Some(opportunity) = self.rx_opportunities.recv().await {
            match self.execute_opportunity(opportunity).await {
                Ok(order) => {
                    debug!("Order executed successfully: {}", order.id);
                    // Send to settlement
                    if let Err(e) = self.tx_orders.send(order).await {
                        error!("Failed to send order to settlement: {}", e);
                    }
                }
                Err(e) => {
                    warn!("Failed to execute opportunity: {}", e);
                    // Continue to next opportunity
                }
            }
        }

        Ok(())
    }

    /// Execute both sides of an arbitrage opportunity
    async fn execute_opportunity(&self, mut opportunity: ArbitrageOpportunity) -> Result<Order> {
        info!(
            "Executing {} arbitrage: {}-{}",
            opportunity.arb_type.as_str(),
            opportunity.market_id_a,
            opportunity.market_id_b
        );

        // Size the position based on available liquidity
        let (qty_a, qty_b, total_cost) =
            self.size_position(&opportunity).await?;

        opportunity.set_sized_position(qty_a, qty_b, total_cost);

        // Execute both orders simultaneously
        let order_a_future = self.client.place_limit_order(
            &opportunity.token_id_a,
            opportunity.price_a,
            qty_a,
            OrderSide::Buy,
        );

        let order_b_future = self.client.place_limit_order(
            &opportunity.token_id_b,
            opportunity.price_b,
            qty_b,
            OrderSide::Buy,
        );

        // Wait for both with timeout
        let result = tokio::time::timeout(
            Duration::from_secs(self.config.order_timeout_secs),
            async {
                let order_a = order_a_future.await?;
                let order_b = order_b_future.await?;
                Ok::<_, crate::error::PolymarketBotError>((order_a, order_b))
            },
        )
        .await;

        match result {
            Ok(Ok((order_id_a, order_id_b))) => {
                info!(
                    "Both orders filled: {} | {} | Profit: ${:.2}",
                    order_id_a, order_id_b, opportunity.expected_profit_usd
                );

                // Create order record
                let mut order = Order::new(
                    opportunity.market_id_a.clone(),
                    opportunity.market_id_b.clone(),
                    qty_a,
                    qty_b,
                    opportunity.price_a,
                    opportunity.price_b,
                    self.config.paper_trading,
                );

                order.mark_filled(order_id_a, order_id_b);

                Ok(order)
            }
            Ok(Err(e)) => {
                warn!("Order placement failed: {}", e);
                Err(e)
            }
            Err(_) => {
                error!("Order timeout");
                Err(crate::error::PolymarketBotError::OrderTimeout)
            }
        }
    }

    /// Calculate position size based on liquidity and risk constraints
    async fn size_position(
        &self,
        opportunity: &ArbitrageOpportunity,
    ) -> Result<(f64, f64, f64)> {
        let balance = self.client.get_balance().await?;

        // Position size is limited by:
        // 1. Max position size config
        // 2. Available balance
        // 3. Available liquidity in orderbook
        // 4. Expected profit exceeding minimum

        let max_size = self.config.max_position_size_usd;
        let available = balance.min(max_size);

        if available < 100.0 {
            return Err(crate::error::PolymarketBotError::InsufficientBalance);
        }

        // Allocate based on prices to achieve even cost split
        // For YES arb: total cost should be < $1.00
        // Allocate: qty_a * price_a + qty_b * price_b = available

        let ratio_a = opportunity.price_a / (opportunity.price_a + opportunity.price_b);
        let ratio_b = opportunity.price_b / (opportunity.price_a + opportunity.price_b);

        let qty_a = (available * ratio_a) / opportunity.price_a;
        let qty_b = (available * ratio_b) / opportunity.price_b;
        let total_cost = (qty_a * opportunity.price_a) + (qty_b * opportunity.price_b);

        debug!(
            "Position size: A={:.0} @ {:.6}, B={:.0} @ {:.6}, Cost: ${:.2}",
            qty_a, opportunity.price_a, qty_b, opportunity.price_b, total_cost
        );

        Ok((qty_a, qty_b, total_cost))
    }
}
