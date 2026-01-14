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
                1000,
                crate::get_current_timestamp(),
            ));

        self.quotes
            .get(&ticker.to_string())
            .map_or(None, |q| Some(q.clone()))
    }
}
