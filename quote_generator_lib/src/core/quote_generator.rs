use std::collections::HashMap;

use rand;

use crate::core::types::StockQuote;

/// Generator for creating and updating stock quotes
pub struct QuoteGenerator {
    pub quotes: HashMap<String, StockQuote>,
}

impl QuoteGenerator {
    /// Creates a new QuoteGenerator with an empty quotes map
    pub fn new() -> Self {
        QuoteGenerator {
            quotes: HashMap::new(),
        }
    }

    fn generate_volume (ticker: &str) -> u32 {
        match ticker {
             // Популярные акции имеют больший объём
            "AAPL" | "MSFT" | "TSLA" => 1000 + (rand::random::<f64>() * 5000.0) as u32,
            "GOOGL" | "AMZN" | "FB" => 500 + (rand::random::<f64>() * 2000.0) as u32,
            // Обычные акции - средний объём
            _ => 100 + (rand::random::<f64>() * 1000.0) as u32,
        }
    }

    /// Generates or updates a quote for the given ticker symbol
    /// 
    /// Returns the updated quote or None if generation fails
    pub fn generate_quote(&mut self, ticker: &str) -> Option<StockQuote> {
        let _ = self
            .quotes
            .entry(ticker.to_string())
            .and_modify(|q| {
                // Обновление цены, объема и временной метки для имитации реальных данных
                q.price += (rand::random::<f64>() - 0.5) * 2.0; // случайное изменение цены
                q.volume += rand::random::<u32>() % 100; // случайное изменение объем
                q.timestamp = crate::get_current_timestamp(); //текущее время
            })
            .or_insert(StockQuote::new(
                ticker,
                100.0 + rand::random::<f64>() * 100.0,
                Self::generate_volume(ticker),
                crate::get_current_timestamp(),
            ));

        self.quotes.get(ticker).cloned()

        /*self.quotes
            .get(&ticker.to_string())
            .map_or(None, |q| Some(q.clone()))*/
    }
}

impl Default for QuoteGenerator {
    fn default() -> Self {
        Self::new()
    }
}