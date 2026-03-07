use crate::error::{PolymarketBotError, Result};
use dashmap::DashMap;
use serde_json::{json, Value};
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio_tungstenite::{connect_async, tungstenite::Message, WebSocketStream};
use futures::{SinkExt, StreamExt};
use tracing::{debug, info, warn};

#[derive(Clone, Debug)]
pub struct OrderbookSnapshot {
    pub best_bid: f64,
    pub best_ask: f64,
    pub bids: Vec<(f64, f64)>,
    pub asks: Vec<(f64, f64)>,
    pub timestamp: i64,
}

/// Real-time orderbook cache updated via WebSocket
pub struct OrderbookCache {
    cache: Arc<DashMap<String, OrderbookSnapshot>>,
}

impl OrderbookCache {
    pub fn new() -> Self {
        Self {
            cache: Arc::new(DashMap::new()),
        }
    }

    pub fn get(&self, token_id: &str) -> Option<OrderbookSnapshot> {
        self.cache.get(token_id).map(|r| r.clone())
    }

    pub fn update(&self, token_id: String, snapshot: OrderbookSnapshot) {
        self.cache.insert(token_id, snapshot);
    }

    pub fn clone_cache(&self) -> Arc<DashMap<String, OrderbookSnapshot>> {
        Arc::clone(&self.cache)
    }
}

/// WebSocket client for real-time market data
pub struct PolymarketWebSocket {
    url: String,
    cache: OrderbookCache,
}

impl PolymarketWebSocket {
    pub fn new(url: String) -> Self {
        Self {
            url,
            cache: OrderbookCache::new(),
        }
    }

    pub fn get_cache(&self) -> OrderbookCache {
        self.cache.clone()
    }

    pub async fn connect_and_run(&self, token_ids: Vec<String>) -> Result<()> {
        info!("🔌 Connecting to WebSocket: {}", self.url);

        let (ws_stream, _) = connect_async(&self.url)
            .await
            .map_err(|e| PolymarketBotError::ConnectionError(format!("WebSocket connect failed: {}", e)))?;

        info!("✅ WebSocket connected");

        // Subscribe to market updates
        let subscription = json!({
            "assets_ids": token_ids,
            "type": "market",
            "custom_feature_enabled": true
        });

        let mut ws = ws_stream;
        ws.send(Message::text(subscription.to_string()))
            .await
            .map_err(|e| PolymarketBotError::ConnectionError(format!("Subscription send failed: {}", e)))?;

        info!("📡 Subscribed to {} token streams", token_ids.len());

        // Listen for messages
        while let Some(msg) = ws.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    if let Err(e) = self.handle_message(&text).await {
                        warn!("Error handling WebSocket message: {}", e);
                    }
                }
                Ok(Message::Close(_)) => {
                    warn!("WebSocket closed");
                    break;
                }
                Err(e) => {
                    warn!("WebSocket error: {}", e);
                    break;
                }
                _ => {}
            }
        }

        Ok(())
    }

    async fn handle_message(&self, text: &str) -> Result<()> {
        let data: Value = serde_json::from_str(text)
            .map_err(|e| PolymarketBotError::ParseError(format!("Failed to parse WebSocket message: {}", e)))?;

        let event_type = data.get("event_type")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");

        match event_type {
            "book" => {
                // Full orderbook snapshot
                if let Some(asset_id) = data.get("asset_id").and_then(|v| v.as_str()) {
                    if let Some(snapshot) = self.parse_orderbook(&data) {
                        self.cache.update(asset_id.to_string(), snapshot);
                        debug!("📊 Updated orderbook for {}", asset_id);
                    }
                }
            }
            "price_change" => {
                // Price level update - extract best bid/ask
                if let Some(asset_id) = data.get("asset_id").and_then(|v| v.as_str()) {
                    if let Some(best_bid) = data.get("best_bid").and_then(|v| v.as_f64()) {
                        if let Some(best_ask) = data.get("best_ask").and_then(|v| v.as_f64()) {
                            debug!("🔄 Price change {}: bid={}, ask={}", asset_id, best_bid, best_ask);
                        }
                    }
                }
            }
            "best_bid_ask" => {
                // Top-of-book update (requires custom_feature_enabled)
                if let Some(asset_id) = data.get("asset_id").and_then(|v| v.as_str()) {
                    if let Some(best_bid) = data.get("best_bid").and_then(|v| v.as_f64()) {
                        if let Some(best_ask) = data.get("best_ask").and_then(|v| v.as_f64()) {
                            debug!("⬆️ Best bid/ask {}: bid={:.4}, ask={:.4}", asset_id, best_bid, best_ask);
                        }
                    }
                }
            }
            "market_resolved" => {
                // Market resolved - settlement trigger
                if let Some(asset_id) = data.get("asset_id").and_then(|v| v.as_str()) {
                    if let Some(winning_id) = data.get("winning_asset_id").and_then(|v| v.as_str()) {
                        info!("🏁 Market resolved! Winner: {} for {}", winning_id, asset_id);
                    }
                }
            }
            _ => {
                debug!("📡 Event: {}", event_type);
            }
        }

        Ok(())
    }

    fn parse_orderbook(&self, data: &Value) -> Option<OrderbookSnapshot> {
        let bids = self.parse_levels(data.get("bids")?);
        let asks = self.parse_levels(data.get("asks")?);

        let best_bid = bids.first().map(|(p, _)| *p).unwrap_or(0.0);
        let best_ask = asks.first().map(|(p, _)| *p).unwrap_or(0.0);

        let timestamp = data
            .get("timestamp")
            .and_then(|v| v.as_i64())
            .unwrap_or_else(|| chrono::Utc::now().timestamp_millis());

        Some(OrderbookSnapshot {
            best_bid,
            best_ask,
            bids,
            asks,
            timestamp,
        })
    }

    fn parse_levels(&self, data: &Value) -> Vec<(f64, f64)> {
        data.as_array()
            .unwrap_or(&vec![])
            .iter()
            .filter_map(|level| {
                let arr = level.as_array()?;
                if arr.len() >= 2 {
                    let price = arr[0].as_str()?.parse::<f64>().ok()?;
                    let size = arr[1].as_str()?.parse::<f64>().ok()?;
                    Some((price, size))
                } else {
                    None
                }
            })
            .collect()
    }
}
