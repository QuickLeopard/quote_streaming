use bincode;
use std::net::UdpSocket;
use std::sync::mpsc;
use std::thread;

use quote_generator_lib::core::StockQuote;

pub struct QuoteReceiver {
    socket: UdpSocket,
}

impl QuoteReceiver {
    pub fn new(bind_addr: &str) -> Result<Self, std::io::Error> {
        let socket = UdpSocket::bind(bind_addr)?;
        println!("Ресивер запущен на {}", bind_addr);
        Ok(Self { socket })
    }

    // Старый метод для простого запуска
    pub fn start_in_thread(self) -> thread::JoinHandle<()> {
        thread::spawn(move || {
            if let Err(e) = self.receive_loop() {
                eprintln!("Ошибка в receive_loop: {}", e);
            }
        })
    }

    // Метод с циклом для получения метрик 
    pub fn receive_loop(self) -> Result<(), Box<dyn std::error::Error>> {
        let mut buf = [0u8; 1024];

        println!("Ожидание данных...");

        loop {
            match self.socket.recv_from(&mut buf) {
                Ok((size, src_addr)) => match bincode::deserialize::<StockQuote>(&buf[..size]) {
                    Ok(quote) => {
                        println!(
                            "{:?}",
                            quote
                        );                        
                    }
                    Err(e) => {
                        eprintln!("Ошибка десериализации: {}", e);
                    }
                },
                Err(e) => {
                    eprintln!("Ошибка получения данных: {}", e);
                }
            }
        }
    }
}