#[cfg(test)]
mod tests {
    use quote_generator_lib::core::StockQuote;

    #[test]
    fn stock_quote_serialization() {
        let quote = StockQuote::new("TSLA", 250.5, 5000, 1234567890);
        let serialized = bincode::serialize(&quote).unwrap();
        let deserialized: StockQuote = bincode::deserialize(&serialized).unwrap();
        
        assert_eq!(deserialized.ticker, "TSLA");
        assert_eq!(deserialized.price, 250.5);
        assert_eq!(deserialized.volume, 5000);
        assert_eq!(deserialized.timestamp, 1234567890);
    }

    #[test]
    fn ping_pong_messages() {
        let ping = b"ping";
        let pong = b"pong";
        
        assert_eq!(std::str::from_utf8(ping).unwrap().trim(), "ping");
        assert_eq!(std::str::from_utf8(pong).unwrap().trim(), "pong");
    }
}
