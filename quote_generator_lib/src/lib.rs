pub mod core {
    mod quote_generator;
    mod types;

    pub use self::quote_generator::QuoteGenerator;
    pub use self::types::StockQuote;
}

use std::time::{SystemTime, UNIX_EPOCH};
use chrono::Local;

/// Returns the current timestamp in milliseconds since UNIX epoch
pub fn get_current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("System time before UNIX epoch")
        .as_millis() as u64
}

/// Returns current timestamp formatted as YYYY-MM-DD HH:MM:SS
pub fn timestamp() -> String {
    Local::now().format("%Y-%m-%d %H:%M:%S").to_string()
}

#[cfg(test)]
mod tests {
    use super::core::{QuoteGenerator, StockQuote};
    use super::*;

    #[test]
    fn quote_to_string() {
        let timestamp = get_current_timestamp();
        let quote = StockQuote::new("AAPL", 123.4, 1000, timestamp);
        assert_eq!(quote.to_string(), format!("AAPL|123.4|1000|{}", timestamp));
    }

    #[test]
    fn quote_from_string() {
        let quote = StockQuote::from_string("AAPL|123.4|1000|1234567890").unwrap();
        assert_eq!(quote.ticker, "AAPL");
        assert_eq!(quote.price, 123.4);
        assert_eq!(quote.volume, 1000);
        assert_eq!(quote.timestamp, 1234567890);
    }

    #[test]
    fn quote_from_string_invalid() {
        assert!(StockQuote::from_string("AAPL|123.4|1000").is_none());
        assert!(StockQuote::from_string("AAPL|invalid|1000|123").is_none());
    }

    #[test]
    fn quote_generator_new() {
        let generator = QuoteGenerator::new();
        assert_eq!(generator.quotes.len(), 0);
    }

    #[test]
    fn quote_generator_generate() {
        let mut generator = QuoteGenerator::new();
        let quote1 = generator.generate_quote("AAPL").unwrap();
        assert_eq!(quote1.ticker, "AAPL");
        assert!(quote1.price > 0.0);
        
        let quote2 = generator.generate_quote("AAPL").unwrap();
        assert_ne!(quote1.price, quote2.price);
    }

    #[test]
    fn timestamp_is_valid() {
        let ts = get_current_timestamp();
        assert!(ts > 1700000000000); // After 2023
    }

    #[test]
    fn timestamp_format() {
        let ts = timestamp();
        // Format: YYYY-MM-DD HH:MM:SS (19 characters)
        assert_eq!(ts.len(), 19);
        // Check format with regex-like pattern
        let parts: Vec<&str> = ts.split(' ').collect();
        assert_eq!(parts.len(), 2);
        
        let date_parts: Vec<&str> = parts[0].split('-').collect();
        assert_eq!(date_parts.len(), 3);
        assert_eq!(date_parts[0].len(), 4); // Year
        assert_eq!(date_parts[1].len(), 2); // Month
        assert_eq!(date_parts[2].len(), 2); // Day
        
        let time_parts: Vec<&str> = parts[1].split(':').collect();
        assert_eq!(time_parts.len(), 3);
        assert_eq!(time_parts[0].len(), 2); // Hour
        assert_eq!(time_parts[1].len(), 2); // Minute
        assert_eq!(time_parts[2].len(), 2); // Second
    }
}
