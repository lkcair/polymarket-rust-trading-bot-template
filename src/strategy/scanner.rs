use crate::api::PolymarketClient;
use crate::config::Config;
use crate::error::Result;
use crate::models::arbitrage::ArbitrageOpportunity;
use dashmap::DashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use tracing::{debug, info, warn};

/// Detects arbitrage opportunities in binary outcome markets
pub struct Scanner {
    config: Arc<Config>,
    client: Arc<PolymarketClient>,
    // Track recently seen opportunities to prevent duplicates
    seen_opportunities: Arc<DashMap<String, Instant>>,
    // Channel to send found opportunities to executor
    tx_opportunities: mpsc::Sender<ArbitrageOpportunity>,
}

impl Scanner {
    pub fn new(
        config: Arc<Config>,
        client: Arc<PolymarketClient>,
        tx_opportunities: mpsc::Sender<ArbitrageOpportunity>,
    ) -> Self {
        Self {
            config,
            client,
            seen_opportunities: Arc::new(DashMap::new()),
            tx_opportunities,
        }
    }

    /// Start the scanning loop
    pub async fn run(&self) -> Result<()> {
        info!("Starting arbitrage scanner");

        loop {
            match self.scan_markets().await {
                Ok(count) => {
                    if count > 0 {
                        debug!("Scanner: found {} potential opportunities", count);
                    }
                }
                Err(e) => {
                    warn!("Scanner error: {}", e);
                    // Continue scanning despite errors
                }
            }

            // Rate limiting
            tokio::time::sleep(Duration::from_millis(self.config.scan_interval_ms)).await;

            // Clean up old entries in seen_opportunities
            self.cleanup_seen_opportunities();
        }
    }

    /// Scan markets for arbitrage opportunities
    async fn scan_markets(&self) -> Result<usize> {
        let markets = self.client.fetch_binary_markets().await?;

        let mut opportunities_found = 0;

        // Group markets by pair for arbitrage detection
        // In practice, we'd match markets by underlying event
        for market in markets.iter() {
            if !market.is_binary_market() {
                continue;
            }

            // Filter by configured category
            if !market.matches_category(&self.config.market_category) {
                debug!("Skipping market {} (category: {:?}, filter: {})",
                    market.polymarket_id, market.category, self.config.market_category);
                continue;
            }

            // Fetch real CLOB orderbooks for both outcomes (token A and token B)
            let book_a = self.client.get_orderbook(&market.token_id_a).await.ok();
            let book_b = self.client.get_orderbook(&market.token_id_b).await.ok();

            // Get best ASK prices (what we'd pay to buy)
            let ask_a = book_a.as_ref().and_then(|b| b.best_ask).unwrap_or(0.0);
            let ask_b = book_b.as_ref().and_then(|b| b.best_ask).unwrap_or(0.0);

            // Real combined price: what we'd actually pay to buy both outcomes
            let combined_ask = ask_a + ask_b;

            // Skip markets where either side has no orderbook data (0.0 ASK)
            // We need liquidity on BOTH sides to execute the arbitrage
            if ask_a == 0.0 || ask_b == 0.0 {
                debug!("Missing orderbook for market {} (A: {:.4}, B: {:.4}) - skipping",
                    market.polymarket_id, ask_a, ask_b);
                continue;
            }

            let category_label = market.category.as_ref()
                .map(|c| format!("📊 {}", c.to_uppercase()))
                .unwrap_or_else(|| "📊 OTHER".to_string());

            // Log markets with real orderbook ASK prices
            info!("{} | {} | {} | ASK {}: {:.4} | ASK {}: {:.4} | COMBINED ASK: {:.4}",
                category_label,
                market.polymarket_id,
                market.question,
                market.outcome_a,
                ask_a,
                market.outcome_b,
                ask_b,
                combined_ask);

            // Flag real arbitrage opportunity when combined ASK < 1.00 (any positive edge)
            // This means we can BUY both outcomes for <$1.00, SELL for $1.00 guaranteed
            if combined_ask < 1.00 && combined_ask > 0.0 {  // > 0.0 to avoid empty orderbooks
                let profit = 1.0 - combined_ask;
                info!("🎯 ARBITRAGE FOUND! [{}] {} | ASK SUM: ${:.4} | Profit: ${:.4} | {} vs {}",
                    market.category.as_ref().unwrap_or(&"OTHER".to_string()).to_uppercase(),
                    market.polymarket_id, combined_ask, profit,
                    market.outcome_a, market.outcome_b);
                opportunities_found += 1;
            }
        }

        Ok(opportunities_found)
    }

    /// Check if we should process this opportunity
    fn should_process(&self, opportunity: &ArbitrageOpportunity) -> bool {
        let key = opportunity.dedup_key();

        // Check if we've seen this recently
        if self.seen_opportunities.contains_key(&key) {
            return false;
        }

        // Check if opportunity is still valid
        opportunity.is_valid()
    }

    /// Mark opportunity as seen
    fn mark_seen(&self, opportunity: &ArbitrageOpportunity) {
        let key = opportunity.dedup_key();
        self.seen_opportunities.insert(key, Instant::now());
    }

    /// Remove old entries from the deduplication cache
    fn cleanup_seen_opportunities(&self) {
        let dedup_window = Duration::from_secs(self.config.dedup_window_secs);
        let now = Instant::now();

        self.seen_opportunities.retain(|_, instant| {
            now.duration_since(*instant) < dedup_window
        });
    }
}
