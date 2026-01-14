#[cfg(test)]
mod tests {
    use quote_generator_lib::core::StockQuote;

    #[test]
    fn deserialize_quote() {
        let quote = StockQuote::new("GOOGL", 150.0, 2000, 9876543210);
        let serialized = bincode::serialize(&quote).unwrap();
        let deserialized: StockQuote = bincode::deserialize(&serialized).unwrap();
        
        assert_eq!(deserialized.ticker, "GOOGL");
        assert_eq!(deserialized.price, 150.0);
    }

    #[test]
    fn pong_message_detection() {
        let pong_msg = b"pong";
        let result = std::str::from_utf8(pong_msg);
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "pong");
    }

    #[test]
    fn invalid_message_handling() {
        let invalid = b"invalid";
        let result = bincode::deserialize::<StockQuote>(invalid);
        
        assert!(result.is_err());
    }
}
