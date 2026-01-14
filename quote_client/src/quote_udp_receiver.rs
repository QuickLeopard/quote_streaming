use bincode;
use std::net::UdpSocket;
use std::thread;
use std::time::Duration;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use chrono::Local;

use quote_generator_lib::core::StockQuote;

const PING_INTERVAL_SECS: u64 = 2;
const RECEIVE_BUFFER_SIZE: usize = 1024;

/// UDP receiver for receiving stock quotes from the server
pub struct QuoteReceiver {
    socket: UdpSocket,
}

impl QuoteReceiver {
    /// Creates a new QuoteReceiver bound to the specified address
    pub fn new(bind_addr: &str) -> Result<Self, std::io::Error> {
        let socket = UdpSocket::bind(bind_addr)?;
        println!("[{}] Ресивер запущен на {}", Local::now().format("%Y-%m-%d %H:%M:%S"), bind_addr);
        Ok(Self { socket })
    }

    /// Starts the receive loop, connecting to server and handling quotes and ping/pong
    /// 
    /// Sends ping every 2 seconds and receives quotes from the server
    pub fn receive_loop(self, server_addr: &str) -> Result<(), Box<dyn std::error::Error>> {
        // Connect socket to server for bidirectional UDP communication
        self.socket.connect(server_addr)?;
        
        let mut buf = [0u8; RECEIVE_BUFFER_SIZE];
        // Shared flag to coordinate shutdown between threads
        let running = Arc::new(AtomicBool::new(true));
        
        // Thread 1: Ping sender - sends "ping" message to server every 2 seconds
        // This keeps the connection alive and lets server know client is still active
        let socket_clone = self.socket.try_clone()?;
        let running_clone = Arc::clone(&running);
        thread::spawn(move || {
            while running_clone.load(Ordering::Relaxed) {
                // Send ping message to server
                if let Err(e) = socket_clone.send(b"ping") {
                    eprintln!("[{}] Failed to send ping: {}", Local::now().format("%Y-%m-%d %H:%M:%S"), e);
                    break;
                }
                // Wait 2 seconds before sending next ping
                thread::sleep(Duration::from_secs(PING_INTERVAL_SECS));
            }
        });

        println!("[{}] Ожидание данных...", Local::now().format("%Y-%m-%d %H:%M:%S"));

        // Main receive loop - receives both pong responses and stock quotes from server
        loop {
            match self.socket.recv(&mut buf) {
                Ok(size) => {
                    // Check if received message is a "pong" response
                    if let Ok(msg) = std::str::from_utf8(&buf[..size]) {
                        if msg.trim() == "pong" {
                            // Ignore pong messages, they're just keep-alive responses
                            continue;
                        }
                    }
                    // Try to deserialize as StockQuote
                    match bincode::deserialize::<StockQuote>(&buf[..size]) {
                        Ok(quote) => {
                            // Successfully received and deserialized a quote
                            println!("[{}] {:?}", Local::now().format("%Y-%m-%d %H:%M:%S"), quote);
                        }
                        Err(_) => {
                            // Silently ignore deserialization errors (could be pong or corrupted data)
                        }
                    }
                }
                Err(e) => {
                    // Connection error - server likely disconnected
                    eprintln!("[{}] Ошибка получения данных: {}", Local::now().format("%Y-%m-%d %H:%M:%S"), e);
                    println!("[{}] Server disconnected, shutting down client", Local::now().format("%Y-%m-%d %H:%M:%S"));
                    // Signal ping sender thread to stop
                    running.store(false, Ordering::Relaxed);
                    break;
                }
            }
        }
        Ok(())
    }
}
