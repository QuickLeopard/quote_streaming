pub mod core {
    mod quote_generator;
    mod types;

    pub use self::quote_generator::QuoteGenerator;
    pub use self::types::StockQuote;
}

use std::time::{SystemTime, UNIX_EPOCH};

/// Returns the current timestamp in milliseconds since UNIX epoch
pub fn get_current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

#[cfg(test)]
mod tests {
    use super::core::StockQuote;
    use super::*;

    #[test]
    fn quote_to_string() {
        let timestamp = get_current_timestamp();
        let quote = StockQuote::new("AAPL", 123.4, 1000, timestamp);
        assert_eq!(quote.to_string(), format!("AAPL|123.4|1000|{}", timestamp));
    }
}
