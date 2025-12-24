#[derive(Debug, Clone)]
pub struct StockQuote {
    pub ticker: String,
    pub price: f64,
    pub volume: u32,
    pub timestamp: u64,
}

impl StockQuote {
    pub fn new(ticker: &str, price: f64, volume: u32, timestamp: u64) -> Self {
        Self {
            ticker: ticker.to_string(),
            price,
            volume,
            timestamp,
        }
    }
}