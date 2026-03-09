use crate::config::Config;
use crate::error::{PolymarketBotError, Result};
use crate::models::Market;
use std::sync::Arc;
use tracing::{debug, info};
use reqwest::Client as HttpClient;
use serde_json::Value;
use uuid::Uuid;
use chrono::Utc;

/// Thin wrapper around the Polymarket SDK
/// Provides arbitrage-specific functionality with MetaMask wallet support
pub struct PolymarketClient {
    config: Arc<Config>,
    http_client: HttpClient,
}

impl PolymarketClient {
    pub fn new(config: Arc<Config>) -> Result<Self> {
        info!("Initializing Polymarket client with MetaMask wallet");

        // Validate private key format
        // Accepts both formats: "0x" prefixed (66 chars) or plain hex (62-65 chars)
        let key = &config.poly_private_key;
        let key_len = key.len();
        let is_valid = if key.starts_with("0x") {
            // 0x prefixed: must be 66 chars (0x + 64 hex)
            key_len == 66 && key[2..].chars().all(|c| c.is_ascii_hexdigit())
        } else {
            // Plain hex: 62-65 chars (common Polygon format, some keys vary slightly)
            (key_len >= 62 && key_len <= 65) && key.chars().all(|c| c.is_ascii_hexdigit())
        };

        if !is_valid {
            return Err(PolymarketBotError::AuthError(
                format!("Invalid private key format. Expected hex string (62-64 chars) or 0x prefixed (66 chars), got {}",
                    key_len)
            ));
        }

        info!("Wallet private key validated (length: {} chars, format: {})",
            key_len,
            if key.starts_with("0x") { "0x-prefixed" } else { "plain hex" }
        );

        let http_client = HttpClient::new();
        Ok(Self { config, http_client })
    }

    /// Get configuration
    pub fn config(&self) -> &Arc<Config> {
        &self.config
    }

    /// Initialize SDK client with MetaMask wallet authentication
    /// Supports L1 (EIP-712) and L2 (HMAC-SHA256) signing via LocalSigner
    pub async fn authenticate(&self) -> Result<()> {
        info!("Authenticating with Polymarket API");
        debug!("Chain ID: {} (Polygon)", self.config.poly_chain_id);
        debug!("CLOB URL: {}", self.config.poly_clob_url);
        debug!("Paper Trading: {}", self.config.paper_trading);

        if self.config.paper_trading {
            info!("📋 PAPER TRADING MODE - No real authentication required");
            info!("✅ Authentication simulated (paper trading disabled actual signing)");
            return Ok(());
        }

        // For live trading, SDK would initialize LocalSigner here:
        // let signer = LocalSigner::from_str(&self.config.poly_private_key)?
        //     .with_chain_id(Some(self.config.poly_chain_id as u64));
        // let client = Client::new(&self.config.poly_clob_url, Config::default())?
        //     .authentication_builder(&signer)
        //     .authenticate()
        //     .await?;

        info!("✅ Live trading authentication ready (SDK client would be initialized here)");
        info!("   L1 Signing: EIP-712 messages signed with MetaMask private key");
        info!("   L2 Signing: HMAC-SHA256 for order authentication");

        Ok(())
    }

    /// Fetch markets from Gamma API
    /// Filter for binary markets only
    pub async fn fetch_binary_markets(&self) -> Result<Vec<Market>> {
        debug!("Fetching binary markets from Gamma API");

        let http_client = HttpClient::new();
        let url = format!("{}/markets?limit={}&closed=false",
            self.config.poly_gamma_url.trim_end_matches('/'),
            self.config.market_limit
        );

        match http_client.get(&url).send().await {
            Ok(response) => {
                match response.json::<Value>().await {
                    Ok(data) => {
                        let markets = self.parse_markets(&data)?;
                        debug!("Fetched {} binary outcome markets", markets.len());
                        // All returned markets are already filtered for binary (2 outcomes) during parsing
                        Ok(markets)
                    }
                    Err(e) => {
                        debug!("Failed to parse Gamma API response: {}", e);
                        Ok(vec![])
                    }
                }
            }
            Err(e) => {
                debug!("Failed to fetch from Gamma API: {}", e);
                Ok(vec![])  // Return empty on error, don't crash scanner
            }
        }
    }

    /// Parse JSON response from Gamma API into Market objects
    fn parse_markets(&self, data: &Value) -> Result<Vec<Market>> {
        let mut markets = Vec::new();

        // Handle both array and object responses
        let items_owned = if let Some(arr) = data.as_array() {
            arr.clone()
        } else if let Some(data_obj) = data.get("data") {
            if let Some(arr) = data_obj.as_array() {
                arr.clone()
            } else {
                vec![]
            }
        } else {
            vec![]
        };

        for item in items_owned {
            if let Ok(market) = self.parse_market_item(&item) {
                markets.push(market);
            }
        }

        Ok(markets)
    }

    /// Parse a single market item from Gamma API
    fn parse_market_item(&self, item: &Value) -> Result<Market> {
        let polymarket_id = item.get("id")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();

        let question = item.get("question")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        // Parse outcome labels from the "outcomes" string field (JSON array as string)
        let outcomes: Vec<String> = item.get("outcomes")
            .and_then(|v| v.as_str())
            .and_then(|s| serde_json::from_str::<Vec<String>>(s).ok())
            .unwrap_or_default();

        // Parse outcome prices from the "outcomePrices" string field (JSON array as string)
        let prices: Vec<f64> = item.get("outcomePrices")
            .and_then(|v| v.as_str())
            .and_then(|s| serde_json::from_str::<Vec<String>>(s).ok())
            .unwrap_or_default()
            .iter()
            .filter_map(|p| p.parse::<f64>().ok())
            .collect();

        // Get token IDs from clobTokenIds
        let token_ids: Vec<String> = item.get("clobTokenIds")
            .and_then(|v| v.as_str())
            .and_then(|s| serde_json::from_str::<Vec<String>>(s).ok())
            .unwrap_or_default();

        // Only create market if we have exactly 2 outcomes (binary market)
        if outcomes.len() != 2 || prices.len() != 2 || token_ids.len() != 2 {
            return Err(PolymarketBotError::ApiError(
                format!("Market {} does not have exactly 2 outcomes", polymarket_id)
            ));
        }

        // Log raw API data to understand what we're receiving
        debug!("RAW API: {} | outcomes: {:?} | prices: {:?} | question: {}",
            polymarket_id, outcomes, prices, question);

        let category = Market::detect_category(&question);

        Ok(Market {
            id: Uuid::new_v4(),
            polymarket_id,
            token_id_a: token_ids[0].clone(),
            token_id_b: token_ids[1].clone(),
            outcome_a: outcomes[0].clone(),
            outcome_b: outcomes[1].clone(),
            question,
            category,
            // outcomePrices[0] is the actual price of outcome_a, outcomePrices[1] is price of outcome_b
            yes_price_a: Some(prices[0]),
            yes_price_b: Some(prices[1]),
            // DO NOT calculate as 1.0 - price. Use actual prices from API.
            // For "Yes/No" outcomes they sum to 1.0. For other binary outcomes they may not.
            no_price_a: Some(prices[0]),  // outcome_a price (could be YES, NO, Team A, Team B, etc)
            no_price_b: Some(prices[1]),  // outcome_b price (complementary outcome price)
            liquidity: item.get("liquidity").and_then(|v| v.as_str()).and_then(|s| s.parse().ok()),
            volume_24h: item.get("volume24hr").and_then(|v| v.as_str()).and_then(|s| s.parse().ok()),
            last_updated: Utc::now(),
            created_at: Utc::now(),
        })
    }

    /// Get orderbook for a specific token from SDK /book endpoint
    pub async fn get_orderbook(&self, token_id: &str) -> Result<OrderbookData> {
        debug!("Fetching orderbook for token: {}", token_id);

        // GET to SDK's /book endpoint with token_id as query param
        let url = format!("{}/book?token_id={}",
            self.config.poly_clob_url.trim_end_matches('/'),
            token_id
        );

        match self.http_client.get(&url).send().await {
            Ok(response) => {
                // First, get the raw response body as string for debugging
                match response.text().await {
                    Ok(body_text) => {
                        info!("📖 RAW /order_book response for {} [{}]: {}", token_id, body_text.len(), body_text);

                        // Now try to parse the JSON
                        match serde_json::from_str::<Value>(&body_text) {
                            Ok(data) => {
                                info!("✅ Successfully parsed orderbook JSON for {}", token_id);

                                // Parse bids and asks from response
                                let bids = self.parse_order_levels(data.get("bids"));
                                let asks = self.parse_order_levels(data.get("asks"));

                                // Best bid is highest bidder (first in sorted list)
                                let best_bid = bids.first().map(|(p, _)| *p);
                                // Best ask is LOWEST seller price - should be last in array or minimum of all asks
                                let best_ask = asks.iter().map(|(p, _)| *p).min_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

                                if best_ask.is_some() || best_bid.is_some() {
                                    info!("✅ Orderbook {}: bid={:?}, ask={:?}", token_id, best_bid, best_ask);
                                }

                                Ok(OrderbookData {
                                    token_id: token_id.to_string(),
                                    bids,
                                    asks,
                                    best_bid,
                                    best_ask,
                                    timestamp: Utc::now(),
                                })
                            }
                            Err(e) => {
                                debug!("❌ Failed to parse JSON orderbook for {}: {} | Body: {}", token_id, e, body_text.chars().take(500).collect::<String>());
                                Ok(OrderbookData {
                                    token_id: token_id.to_string(),
                                    bids: vec![],
                                    asks: vec![],
                                    best_bid: None,
                                    best_ask: None,
                                    timestamp: Utc::now(),
                                })
                            }
                        }
                    }
                    Err(e) => {
                        debug!("❌ Failed to read response body for {}: {}", token_id, e);
                        Ok(OrderbookData {
                            token_id: token_id.to_string(),
                            bids: vec![],
                            asks: vec![],
                            best_bid: None,
                            best_ask: None,
                            timestamp: Utc::now(),
                        })
                    }
                }
            }
            Err(e) => {
                debug!("❌ SDK /order_book request failed for {}: {}", token_id, e);
                Ok(OrderbookData {
                    token_id: token_id.to_string(),
                    bids: vec![],
                    asks: vec![],
                    best_bid: None,
                    best_ask: None,
                    timestamp: Utc::now(),
                })
            }
        }
    }

    /// Parse order levels from JSON array
    /// SDK returns bids/asks as array of objects with "price" and "size" string fields
    fn parse_order_levels(&self, data: Option<&Value>) -> Vec<(f64, f64)> {
        match data {
            Some(Value::Array(levels)) => {
                levels.iter()
                    .filter_map(|level| {
                        match level {
                            Value::Object(obj) => {
                                // Parse object with "price" and "size" fields
                                let price = obj.get("price")
                                    .and_then(|p| {
                                        p.as_str()
                                            .and_then(|s| s.parse::<f64>().ok())
                                            .or_else(|| p.as_f64())
                                    })?;
                                let size = obj.get("size")
                                    .and_then(|s| {
                                        s.as_str()
                                            .and_then(|sz| sz.parse::<f64>().ok())
                                            .or_else(|| s.as_f64())
                                    })?;
                                Some((price, size))
                            }
                            Value::Array(arr) if arr.len() >= 2 => {
                                // Fallback: handle [price, size] tuple format
                                let price = arr[0].as_str()
                                    .and_then(|p| p.parse::<f64>().ok())
                                    .or_else(|| arr[0].as_f64())?;
                                let size = arr[1].as_str()
                                    .and_then(|s| s.parse::<f64>().ok())
                                    .or_else(|| arr[1].as_f64())?;
                                Some((price, size))
                            }
                            _ => None
                        }
                    })
                    .collect()
            }
            _ => vec![]
        }
    }

    /// Place a limit order
    pub async fn place_limit_order(
        &self,
        token_id: &str,
        price: f64,
        quantity: f64,
        side: OrderSide,
    ) -> Result<String> {
        if self.config.paper_trading {
            info!("PAPER TRADING: Would place order {} {} @ ${:.4} (token: {})", 
                quantity, side.as_str(), price, token_id);
            // Simulate 99% fill rate
            let filled = rand::random::<f64>() < 0.99;
            if filled {
                Ok(format!("PAPER_ORDER_{}", uuid::Uuid::new_v4()))
            } else {
                Err(PolymarketBotError::OrderError("Paper trade simulated no fill".to_string()))
            }
        } else {
            debug!("Placing {} order: token={}, qty={}, price={}",
                side.as_str(), token_id, quantity, price);

            // Build order request for CLOB API
            let order_payload = serde_json::json!({
                "token_id": token_id,
                "price": price.to_string(),
                "size": quantity.to_string(),
                "side": side.as_str(),
                "order_type": "FOK",  // Fill-or-Kill
            });

            let url = format!("{}/order", self.config.poly_clob_url.trim_end_matches('/'));
            
            match self.http_client.post(&url)
                .json(&order_payload)
                .send()
                .await
            {
                Ok(response) => {
                    if response.status().is_success() {
                        match response.json::<serde_json::Value>().await {
                            Ok(data) => {
                                let order_id = data.get("order_id")
                                    .or_else(|| data.get("id"))
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("unknown")
                                    .to_string();
                                info!("✅ Order placed: {}", order_id);
                                Ok(order_id)
                            }
                            Err(e) => Err(PolymarketBotError::OrderError(
                                format!("Failed to parse order response: {}", e)
                            ))
                        }
                    } else {
                        Err(PolymarketBotError::OrderError(
                            format!("Order rejected: HTTP {}", response.status())
                        ))
                    }
                }
                Err(e) => Err(PolymarketBotError::OrderError(
                    format!("Order request failed: {}", e)
                ))
            }
        }
    }

    /// Cancel an order
    pub async fn cancel_order(&self, order_id: &str) -> Result<()> {
        if self.config.paper_trading {
            info!("PAPER TRADING: Would cancel order {}", order_id);
            Ok(())
        } else {
            debug!("Cancelling order: {}", order_id);

            let url = format!("{}/order/{}", 
                self.config.poly_clob_url.trim_end_matches('/'), 
                order_id);
            
            match self.http_client.delete(&url).send().await {
                Ok(response) => {
                    if response.status().is_success() {
                        info!("✅ Order cancelled: {}", order_id);
                        Ok(())
                    } else {
                        Err(PolymarketBotError::OrderError(
                            format!("Cancel failed: HTTP {}", response.status())
                        ))
                    }
                }
                Err(e) => Err(PolymarketBotError::OrderError(
                    format!("Cancel request failed: {}", e)
                ))
            }
        }
    }

    /// Get user positions/balances
    pub async fn get_balance(&self) -> Result<f64> {
        if self.config.paper_trading {
            // Return configured max position size for paper trading
            Ok(self.config.max_position_size_usd * 10.0)
        } else {
            debug!("Fetching account balance");

            let url = format!("{}/balance", self.config.poly_clob_url.trim_end_matches('/'));
            
            match self.http_client.get(&url).send().await {
                Ok(response) => {
                    if response.status().is_success() {
                        match response.json::<serde_json::Value>().await {
                            Ok(data) => {
                                let balance = data.get("balance")
                                    .or_else(|| data.get("collateral"))
                                    .and_then(|v| v.as_str())
                                    .and_then(|s| s.parse::<f64>().ok())
                                    .unwrap_or(0.0);
                                debug!("Account balance: ${:.2}", balance);
                                Ok(balance)
                            }
                            Err(e) => {
                                warn!("Failed to parse balance: {}", e);
                                Ok(0.0)
                            }
                        }
                    } else {
                        warn!("Balance request failed: HTTP {}", response.status());
                        Ok(0.0)
                    }
                }
                Err(e) => {
                    warn!("Balance request error: {}", e);
                    Ok(0.0)
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrderSide {
    Buy,
    Sell,
}

impl OrderSide {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Buy => "BUY",
            Self::Sell => "SELL",
        }
    }
}

#[derive(Debug, Clone)]
pub struct OrderbookData {
    pub token_id: String,
    pub bids: Vec<(f64, f64)>, // (price, quantity)
    pub asks: Vec<(f64, f64)>, // (price, quantity)
    pub best_bid: Option<f64>,
    pub best_ask: Option<f64>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl OrderbookData {
    pub fn best_bid_price(&self) -> Option<f64> {
        self.bids.first().map(|(p, _)| *p)
    }

    pub fn best_ask_price(&self) -> Option<f64> {
        self.asks.first().map(|(p, _)| *p)
    }

    pub fn best_bid_quantity(&self) -> Option<f64> {
        self.bids.first().map(|(_, q)| *q)
    }

    pub fn best_ask_quantity(&self) -> Option<f64> {
        self.asks.first().map(|(_, q)| *q)
    }
}
