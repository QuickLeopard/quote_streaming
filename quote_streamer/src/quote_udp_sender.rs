use bincode;

use std::net::UdpSocket;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;

use std::collections::HashSet;

use bus::Bus;

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

    pub fn start_broadcasting_with_bus(
        self,
        target_addr: String,
        tickers: String,
        bus: Arc<Mutex<Bus<StockQuote>>>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let tickers = tickers
            .split(",")
            .map(|s| s.to_string())
            .collect::<HashSet<String>>();

        let mut bus = bus.lock().unwrap();

        let mut reader = bus.add_rx();

        let _ = thread::spawn(move || {
            while let Ok(quote) = reader.recv() {
                let ticker = quote.ticker.clone();
                if tickers.contains(&ticker) {
                    println!("Broadcasting with bus got: {:?}", quote);

                    if let Err(e) = self.send_to(&quote, &target_addr) {
                        eprintln!("Failed to send quote: {}", e);
                    }
                } else {
                    //println! ("Skipping ticker: {}", ticker);
                }
            }
        });

        Ok(())
    }
}
