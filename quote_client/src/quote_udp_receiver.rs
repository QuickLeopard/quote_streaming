use bincode;
use std::net::UdpSocket;
use std::thread;
use std::time::Duration;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use chrono::Local;

use quote_generator_lib::core::StockQuote;

pub struct QuoteReceiver {
    socket: UdpSocket,
}

impl QuoteReceiver {
    pub fn new(bind_addr: &str) -> Result<Self, std::io::Error> {
        let socket = UdpSocket::bind(bind_addr)?;
        println!("[{}] Ресивер запущен на {}", Local::now().format("%Y-%m-%d %H:%M:%S"), bind_addr);
        Ok(Self { socket })
    }

    // Метод с циклом для получения метрик
    pub fn receive_loop(self, server_addr: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.socket.connect(server_addr)?;
        
        let mut buf = [0u8; 1024];
        let running = Arc::new(AtomicBool::new(true));
        
        // Ping sender thread
        let socket_clone = self.socket.try_clone()?;
        let running_clone = Arc::clone(&running);
        thread::spawn(move || {
            while running_clone.load(Ordering::Relaxed) {
                if let Err(e) = socket_clone.send(b"ping") {
                    eprintln!("[{}] Failed to send ping: {}", Local::now().format("%Y-%m-%d %H:%M:%S"), e);
                    break;
                }
                thread::sleep(Duration::from_secs(2));
            }
        });

        println!("[{}] Ожидание данных...", Local::now().format("%Y-%m-%d %H:%M:%S"));

        loop {
            match self.socket.recv(&mut buf) {
                Ok(size) => {
                    if let Ok(msg) = std::str::from_utf8(&buf[..size]) {
                        if msg.trim() == "pong" {
                            continue;
                        }
                    }
                    match bincode::deserialize::<StockQuote>(&buf[..size]) {
                        Ok(quote) => {
                            println!("[{}] {:?}", Local::now().format("%Y-%m-%d %H:%M:%S"), quote);
                        }
                        Err(_) => {}
                    }
                }
                Err(e) => {
                    eprintln!("[{}] Ошибка получения данных: {}", Local::now().format("%Y-%m-%d %H:%M:%S"), e);
                    running.store(false, Ordering::Relaxed);
                    break;
                }
            }
        }
        Ok(())
    }
}
