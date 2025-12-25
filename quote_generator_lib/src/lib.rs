
pub mod core {
    mod types;

        pub use self::types::{StockQuote};
}

use std::time::{SystemTime, UNIX_EPOCH};
//use core::StockQuote;

pub fn get_current_timestamp () -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::core::StockQuote;

    #[test]
    fn quote_to_string() {
        let timestamp = get_current_timestamp();
        let quote = StockQuote::new ("AAPL", 123.4, 1000, timestamp);
        assert_eq!(quote.to_string(), format! ("AAPL|123.4|1000|{}", timestamp));
    }
}
