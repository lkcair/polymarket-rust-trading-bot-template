use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum OrderStatus {
    Pending,
    PartiallyFilled,
    Filled,
    Cancelled,
    Failed,
    Resolved,
}

impl OrderStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::PartiallyFilled => "partially_filled",
            Self::Filled => "filled",
            Self::Cancelled => "cancelled",
            Self::Failed => "failed",
            Self::Resolved => "resolved",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "pending" => Some(Self::Pending),
            "partially_filled" => Some(Self::PartiallyFilled),
            "filled" => Some(Self::Filled),
            "cancelled" => Some(Self::Cancelled),
            "failed" => Some(Self::Failed),
            "resolved" => Some(Self::Resolved),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    pub id: Uuid,
    pub market_id_a: String,
    pub market_id_b: String,
    pub order_id_a: Option<String>,
    pub order_id_b: Option<String>,
    pub quantity_a: f64,
    pub quantity_b: f64,
    pub price_a: f64,
    pub price_b: f64,
    pub cost_usd: f64,
    pub status: OrderStatus,
    pub paper_trade: bool,
    pub created_at: DateTime<Utc>,
    pub filled_at: Option<DateTime<Utc>>,
    pub resolved_at: Option<DateTime<Utc>>,
}

impl Order {
    pub fn new(
        market_id_a: String,
        market_id_b: String,
        quantity_a: f64,
        quantity_b: f64,
        price_a: f64,
        price_b: f64,
        paper_trade: bool,
    ) -> Self {
        let cost_usd = (quantity_a * price_a) + (quantity_b * price_b);

        Self {
            id: Uuid::new_v4(),
            market_id_a,
            market_id_b,
            order_id_a: None,
            order_id_b: None,
            quantity_a,
            quantity_b,
            price_a,
            price_b,
            cost_usd,
            status: OrderStatus::Pending,
            paper_trade,
            created_at: Utc::now(),
            filled_at: None,
            resolved_at: None,
        }
    }

    pub fn expected_payout(&self) -> f64 {
        // Both sides are YES or both sides are NO
        // One side wins and pays $1.00, the other pays $0.00
        self.quantity_a + self.quantity_b
    }

    pub fn expected_profit(&self) -> f64 {
        self.expected_payout() - self.cost_usd
    }

    pub fn mark_filled(&mut self, order_id_a: String, order_id_b: String) {
        self.order_id_a = Some(order_id_a);
        self.order_id_b = Some(order_id_b);
        self.status = OrderStatus::Filled;
        self.filled_at = Some(Utc::now());
    }

    pub fn mark_resolved(&mut self, _profit_loss: f64) {
        self.status = OrderStatus::Resolved;
        self.resolved_at = Some(Utc::now());
    }
}
