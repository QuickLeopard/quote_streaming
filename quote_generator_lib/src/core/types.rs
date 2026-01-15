use std::fmt::Display;

use serde::{Deserialize, Serialize};

/// Represents a stock quote with ticker, price, volume, and timestamp
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StockQuote {
    pub ticker: String,
    pub price: f64,
    pub volume: u32,
    pub timestamp: u64,
}

impl StockQuote {
    /// Creates a new StockQuote
    pub fn new(ticker: &str, price: f64, volume: u32, timestamp: u64) -> Self {
        Self {
            ticker: ticker.to_string(),
            price,
            volume,
            timestamp,
        }
    }

    /// Parses a StockQuote from a pipe-delimited string
    /// 
    /// Returns None if the string format is invalid
    pub fn from_string(s: &str) -> Option<Self> {
        let parts: Vec<&str> = s.split('|').collect();
        if parts.len() == 4 {
            Some(StockQuote {
                ticker: parts[0].to_string(),
                price: parts[1].parse().ok()?,
                volume: parts[2].parse().ok()?,
                timestamp: parts[3].parse().ok()?,
            })
        } else {
            None
        }
    }
}

impl Default for StockQuote {
    fn default() -> Self {
        Self::new("", 0.0, 0, 0)
    }
}

impl Display for StockQuote {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}|{}|{}|{}",
            self.ticker, self.price, self.volume, self.timestamp
        )
    }
}
