use bincode;

use std::net::UdpSocket;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
use std::time::{Duration, Instant};
use std::sync::atomic::{AtomicBool, Ordering};

use std::collections::HashSet;

use bus::Bus;
use chrono::Local;

use quote_generator_lib::core::StockQuote;

/// UDP sender for broadcasting stock quotes to clients
pub struct QuoteSender {
    socket: UdpSocket,
}

impl QuoteSender {
    /// Creates a new QuoteSender bound to the specified address
    pub fn new(bind_addr: &str) -> Result<Self, std::io::Error> {
        let socket = UdpSocket::bind(bind_addr)?;
        Ok(Self { socket })
    }

    /// Sends a quote to the specified target address
    pub fn send_to(
        &self,
        quote: &StockQuote,
        target_addr: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let encoded = bincode::serialize(quote)?;
        self.socket.send_to(&encoded, target_addr)?;
        Ok(())
    }

    /// Starts broadcasting quotes from the bus to the target address
    /// 
    /// Creates three threads:
    /// - Ping listener: Receives ping messages from client and responds with pong
    /// - Timeout checker: Monitors last ping time, shuts down after 5 seconds without ping
    /// - Broadcasting: Sends filtered quotes to the connected client
    /// 
    /// Returns the server's local socket address for client connection
    pub fn start_broadcasting_with_bus(
        self,
        target_addr: String,
        tickers: String,
        bus: Arc<Mutex<Bus<StockQuote>>>,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let tickers = tickers
            .split(",")
            .map(|s| s.to_string())
            .collect::<HashSet<String>>();

        let mut bus = bus.lock().map_err(|_| "Bus lock poisoned")?;
        let mut reader = bus.add_rx();

        self.socket.connect(&target_addr)?;
        let server_addr = self.socket.local_addr()?.to_string();

        let shutdown = Arc::new(AtomicBool::new(false));
        let last_ping = Arc::new(Mutex::new(Instant::now()));

        // Ping listener thread
        let socket_clone = self.socket.try_clone()?;
        let shutdown_clone = Arc::clone(&shutdown);
        let last_ping_clone = Arc::clone(&last_ping);
        let target_addr_clone = target_addr.clone();
        
        thread::spawn(move || {
            socket_clone.set_read_timeout(Some(Duration::from_millis(100))).ok();
            let mut buf = [0u8; 64];
            
            while !shutdown_clone.load(Ordering::Relaxed) {
                if let Ok((size, src)) = socket_clone.recv_from(&mut buf) {
                    if let Ok(msg) = std::str::from_utf8(&buf[..size]) {
                        if msg.trim() == "ping" {
                            println!("[{}] Received ping from {}", Local::now().format("%Y-%m-%d %H:%M:%S"), src);
                            if let Ok(mut last_ping) = last_ping_clone.lock() {
                                *last_ping = Instant::now();
                            }
                            let _ = socket_clone.send(b"pong");
                        }
                    }
                }
            }
            println!("[{}] Ping listener thread stopped for {}", Local::now().format("%Y-%m-%d %H:%M:%S"), target_addr_clone);
        });

        // Timeout checker thread
        let shutdown_clone = Arc::clone(&shutdown);
        let last_ping_clone = Arc::clone(&last_ping);
        let target_addr_clone2 = target_addr.clone();
        
        thread::spawn(move || {
            while !shutdown_clone.load(Ordering::Relaxed) {
                thread::sleep(Duration::from_secs(1));
                if let Ok(last_ping) = last_ping_clone.lock() {
                    if last_ping.elapsed() > Duration::from_secs(5) {
                        println!("[{}] [TIMEOUT] No ping from {} for 5 seconds, shutting down all threads", Local::now().format("%Y-%m-%d %H:%M:%S"), target_addr_clone2);
                        shutdown_clone.store(true, Ordering::Relaxed);
                        break;
                    }
                }
            }
            println!("[{}] Timeout checker thread stopped for {}", Local::now().format("%Y-%m-%d %H:%M:%S"), target_addr_clone2);
        });

        // Broadcasting thread
        thread::spawn(move || {
            while !shutdown.load(Ordering::Relaxed) {
                if let Ok(quote) = reader.recv() {
                    if tickers.contains(&quote.ticker) {
                        println!("[{}] Broadcasting with bus got: {:?}", Local::now().format("%Y-%m-%d %H:%M:%S"), quote);
                        if let Ok(encoded) = bincode::serialize(&quote) {
                            if let Err(e) = self.socket.send(&encoded) {
                                eprintln!("[{}] Failed to send quote: {}", Local::now().format("%Y-%m-%d %H:%M:%S"), e);
                            }
                        }
                    }
                }
            }
            println!("[{}] Broadcasting thread stopped for {}", Local::now().format("%Y-%m-%d %H:%M:%S"), target_addr);
        });

        Ok(server_addr)
    }
}
