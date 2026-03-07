use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ArbType {
    YES,
    NO,
}

impl ArbType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::YES => "YES",
            Self::NO => "NO",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "YES" => Some(Self::YES),
            "NO" => Some(Self::NO),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArbitrageOpportunity {
    pub id: Uuid,
    pub market_id_a: String,
    pub market_id_b: String,
    pub token_id_a: String,
    pub token_id_b: String,
    pub outcome_a: String,
    pub outcome_b: String,
    pub arb_type: ArbType,
    pub combined_price: f64,
    pub price_a: f64,
    pub price_b: f64,
    pub spread: f64,
    pub quantity_a: f64,
    pub quantity_b: f64,
    pub total_cost_usd: f64,
    pub expected_profit_usd: f64,
    pub liquidity_a: Option<f64>,
    pub liquidity_b: Option<f64>,
    pub timestamp: DateTime<Utc>,
}

impl ArbitrageOpportunity {
    pub fn new(
        market_id_a: String,
        market_id_b: String,
        token_id_a: String,
        token_id_b: String,
        outcome_a: String,
        outcome_b: String,
        arb_type: ArbType,
        combined_price: f64,
        price_a: f64,
        price_b: f64,
    ) -> Self {
        let spread = 1.0 - combined_price;
        let quantity_a = 0.0;
        let quantity_b = 0.0;
        let total_cost_usd = 0.0;
        let expected_profit_usd = spread;

        Self {
            id: Uuid::new_v4(),
            market_id_a,
            market_id_b,
            token_id_a,
            token_id_b,
            outcome_a,
            outcome_b,
            arb_type,
            combined_price,
            price_a,
            price_b,
            spread,
            quantity_a,
            quantity_b,
            total_cost_usd,
            expected_profit_usd,
            liquidity_a: None,
            liquidity_b: None,
            timestamp: Utc::now(),
        }
    }

    /// Update quantities and costs based on sizing
    pub fn set_sized_position(
        &mut self,
        quantity_a: f64,
        quantity_b: f64,
        total_cost_usd: f64,
    ) {
        self.quantity_a = quantity_a;
        self.quantity_b = quantity_b;
        self.total_cost_usd = total_cost_usd;
        self.expected_profit_usd = (quantity_a + quantity_b) - total_cost_usd;
    }

    /// Check if opportunity is still valid
    pub fn is_valid(&self) -> bool {
        self.combined_price < 0.98 && self.spread > 0.0
    }

    /// Get unique identifier for deduplication
    pub fn dedup_key(&self) -> String {
        format!("{}_{}_{}",
            self.market_id_a,
            self.market_id_b,
            self.arb_type.as_str()
        )
    }
}
