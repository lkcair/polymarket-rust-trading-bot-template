use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MarketCategory {
    Sports,
    Crypto,
    Political,
    RealWorld,
    Other,
}

impl MarketCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Sports => "sports",
            Self::Crypto => "crypto",
            Self::Political => "political",
            Self::RealWorld => "realworld",
            Self::Other => "other",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Market {
    pub id: Uuid,
    pub polymarket_id: String,
    pub token_id_a: String,
    pub token_id_b: String,
    pub outcome_a: String,
    pub outcome_b: String,
    pub question: String,
    pub category: Option<String>,
    pub yes_price_a: Option<f64>,
    pub yes_price_b: Option<f64>,
    pub no_price_a: Option<f64>,
    pub no_price_b: Option<f64>,
    pub liquidity: Option<f64>,
    pub volume_24h: Option<f64>,
    pub last_updated: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

impl Market {
    pub fn new(
        polymarket_id: String,
        token_id_a: String,
        token_id_b: String,
        outcome_a: String,
        outcome_b: String,
        question: String,
    ) -> Self {
        let now = Utc::now();
        let category = Self::detect_category(&question);
        Self {
            id: Uuid::new_v4(),
            polymarket_id,
            token_id_a,
            token_id_b,
            outcome_a,
            outcome_b,
            question,
            category,
            yes_price_a: None,
            yes_price_b: None,
            no_price_a: None,
            no_price_b: None,
            liquidity: None,
            volume_24h: None,
            last_updated: now,
            created_at: now,
        }
    }

    /// Detect market category from question text
    pub fn detect_category(question: &str) -> Option<String> {
        let q_lower = question.to_lowercase();

        // Sports keywords
        if q_lower.contains(" nfl ") || q_lower.contains(" nba ") ||
           q_lower.contains(" nhl ") || q_lower.contains(" mlb ") ||
           q_lower.contains(" nfl ") || q_lower.contains(" world cup ") ||
           q_lower.contains(" super bowl ") || q_lower.contains(" ncaa ") ||
           q_lower.contains(" premier league ") || q_lower.contains(" laliga ") ||
           q_lower.contains("ncaa basketball") || q_lower.contains("olympic") ||
           q_lower.contains(" vs ") || q_lower.contains(" vs. ") ||
           q_lower.contains("match") || q_lower.contains("game") ||
           q_lower.contains("tournament") || q_lower.contains("championship") ||
           q_lower.contains(" fc ") || q_lower.contains(" fc.") ||
           q_lower.contains("team") || q_lower.contains("season") {
            return Some("sports".to_string());
        }

        // Crypto keywords
        if q_lower.contains("bitcoin") || q_lower.contains("ethereum") ||
           q_lower.contains("crypto") || q_lower.contains("btc") ||
           q_lower.contains("eth") || q_lower.contains("token") ||
           q_lower.contains("coin") || q_lower.contains("blockchain") {
            return Some("crypto".to_string());
        }

        // Political keywords
        if q_lower.contains("election") || q_lower.contains("president") ||
           q_lower.contains("senate") || q_lower.contains("congress") ||
           q_lower.contains("house of representatives") || q_lower.contains("governor") ||
           q_lower.contains("vote") || q_lower.contains("impeach") ||
           q_lower.contains("convicted") || q_lower.contains("indicted") ||
           q_lower.contains("brexit") || q_lower.contains("parliament") {
            return Some("political".to_string());
        }

        // Real-world events
        if q_lower.contains("weather") || q_lower.contains("earthquake") ||
           q_lower.contains("hurricane") || q_lower.contains("tornado") ||
           q_lower.contains("recession") || q_lower.contains("rate") ||
           q_lower.contains("gdp") || q_lower.contains("unemployment") ||
           q_lower.contains("inflation") || q_lower.contains("stock market") {
            return Some("realworld".to_string());
        }

        Some("other".to_string())
    }

    /// Check if market matches the configured category filter
    pub fn matches_category(&self, filter_category: &str) -> bool {
        if filter_category == "all" {
            return true;
        }

        match &self.category {
            Some(cat) => cat == filter_category,
            None => filter_category == "other",
        }
    }

    /// Calculate combined YES price for arbitrage detection
    pub fn combined_yes_price(&self) -> Option<f64> {
        match (self.yes_price_a, self.yes_price_b) {
            (Some(a), Some(b)) => Some(a + b),
            _ => None,
        }
    }

    /// Calculate combined NO price for arbitrage detection
    pub fn combined_no_price(&self) -> Option<f64> {
        match (self.yes_price_a, self.yes_price_b) {
            (Some(a), Some(b)) => {
                // NO price is complementary: (1 - YES_A) + (1 - YES_B)
                Some((1.0 - a) + (1.0 - b))
            }
            _ => None,
        }
    }

    /// Check if this market is suitable for arbitrage trading
    pub fn is_binary_market(&self) -> bool {
        // Ensure we have binary outcomes only (2 outcomes)
        !self.token_id_a.is_empty()
            && !self.token_id_b.is_empty()
            && !self.outcome_a.is_empty()
            && !self.outcome_b.is_empty()
    }
}
