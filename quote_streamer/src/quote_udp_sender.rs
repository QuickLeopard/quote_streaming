
use bincode;
use std::net::UdpSocket;
use std::thread;
use std::time::Duration;
use rand;

use quote_generator_lib::core::StockQuote;

pub struct QuoteSender {
    socket: UdpSocket,
}

impl QuoteSender {
    pub fn new(bind_addr: &str) -> Result<Self, std::io::Error> {
        let socket = UdpSocket::bind(bind_addr)?;
        Ok(Self { socket })
    }


    // Метод отправки сообщений в сокет
    pub fn send_to(
        &self,
        quote: &StockQuote,
        target_addr: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let encoded = bincode::serialize(quote)?;
        self.socket.send_to(&encoded, target_addr)?;
        Ok(())
    }

     pub fn start_broadcasting(
        self,
        target_addr: String,
        interval_ms: u64,
    ) -> Result<(), Box<dyn std::error::Error>> {
        thread::spawn(move || {
            let mut quote = StockQuote::new ("AAPL", 150.0, 1000, quote_generator_lib::get_current_timestamp());

            loop {
                // Обновление цены и объема для имитации реальных данных
                quote.price += (rand::random::<f64>() - 0.5) * 2.0; // случайное изменение цены
                quote.volume += rand::random::<u32>() % 100; // случайное изменение объема

                if let Err(e) = self.send_to(&quote, &target_addr) {
                    eprintln!("Failed to send quote: {}", e);
                }

                thread::sleep(Duration::from_millis(interval_ms));
            }
        });
        Ok(())
    }
    
}