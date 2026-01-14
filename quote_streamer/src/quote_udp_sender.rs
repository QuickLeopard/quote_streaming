use bincode;

use std::net::UdpSocket;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
use std::time::{Duration, Instant};
use std::sync::atomic::{AtomicBool, Ordering};

use std::collections::HashSet;

use bus::Bus;
use log::{info, error, warn, debug};

use quote_generator_lib::core::StockQuote;
use quote_generator_lib::timestamp;

const PING_TIMEOUT_SECS: u64 = 5;
const PING_CHECK_INTERVAL_SECS: u64 = 1;
const SOCKET_READ_TIMEOUT_MS: u64 = 100;

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

        // Connect socket to client address for bidirectional UDP communication
        self.socket.connect(&target_addr)?;
        let server_addr = self.socket.local_addr()?.to_string();

        // Shared state for coordinating thread shutdown
        let shutdown = Arc::new(AtomicBool::new(false));
        let last_ping = Arc::new(Mutex::new(Instant::now()));

        // Thread 1: Ping listener - receives ping messages from client and responds with pong
        // This thread listens for incoming UDP packets and filters for "ping" messages
        let socket_clone = self.socket.try_clone()?;
        let shutdown_clone = Arc::clone(&shutdown);
        let last_ping_clone = Arc::clone(&last_ping);
        let target_addr_clone = target_addr.clone();
        
        thread::spawn(move || {
            socket_clone.set_read_timeout(Some(Duration::from_millis(SOCKET_READ_TIMEOUT_MS))).ok();
            let mut buf = [0u8; 64];
            
            // Keep listening until shutdown flag is set
            while !shutdown_clone.load(Ordering::Relaxed) {
                if let Ok((size, src)) = socket_clone.recv_from(&mut buf) {
                    // Check if received message is "ping"
                    if let Ok(msg) = std::str::from_utf8(&buf[..size]) {
                        if msg.trim() == "ping" {
                            println!("[{}] Received ping from {}", timestamp(), src);
                            debug!("Received ping from {}", src);
                            // Update last ping timestamp
                            if let Ok(mut last_ping) = last_ping_clone.lock() {
                                *last_ping = Instant::now();
                            }
                            // Send pong response back to client
                            let _ = socket_clone.send(b"pong");
                        }
                    }
                }
            }
            println!("[{}] Ping listener thread stopped for {}", timestamp(), target_addr_clone);
        });

        // Thread 2: Timeout checker - monitors last ping time and triggers shutdown if timeout
        // Checks every second if client has sent a ping within the last 5 seconds
        let shutdown_clone = Arc::clone(&shutdown);
        let last_ping_clone = Arc::clone(&last_ping);
        let target_addr_clone2 = target_addr.clone();
        
        thread::spawn(move || {
            while !shutdown_clone.load(Ordering::Relaxed) {
                thread::sleep(Duration::from_secs(PING_CHECK_INTERVAL_SECS));
                // Check if last ping was more than PING_TIMEOUT_SECS ago
                if let Ok(last_ping) = last_ping_clone.lock() {
                    if last_ping.elapsed() > Duration::from_secs(PING_TIMEOUT_SECS) {
                        println!("[{}] [TIMEOUT] No ping from {} for {} seconds, shutting down all threads", timestamp(), target_addr_clone2, PING_TIMEOUT_SECS);
                        warn!("No ping from {} for {} seconds, shutting down all threads", target_addr_clone2, PING_TIMEOUT_SECS);
                        // Set shutdown flag to stop all threads
                        shutdown_clone.store(true, Ordering::Relaxed);
                        break;
                    }
                }
            }
            println!("[{}] Timeout checker thread stopped for {}", timestamp(), target_addr_clone2);
            info!("Timeout checker thread stopped for {}", target_addr_clone2);
        });

        // Thread 3: Broadcasting - receives quotes from bus and sends to client
        // Filters quotes by ticker and serializes them before sending via UDP
        thread::spawn(move || {
            while !shutdown.load(Ordering::Relaxed) {
                // Receive quote from the pub-sub bus
                if let Ok(quote) = reader.recv() {
                    // Only send quotes for subscribed tickers
                    if tickers.contains(&quote.ticker) {
                        println!("[{}] Broadcasting with bus got: {:?}", timestamp(), quote);
                        debug!("Broadcasting quote: {:?}", quote);
                        // Serialize quote to binary format
                        if let Ok(encoded) = bincode::serialize(&quote) {
                            // Send serialized quote to connected client
                            if let Err(e) = self.socket.send(&encoded) {
                                eprintln!("[{}] Failed to send quote: {}", timestamp(), e);
                                error!("Failed to send quote: {}", e);
                            }
                        }
                    }
                }
            }
            println!("[{}] Broadcasting thread stopped for {}", timestamp(), target_addr);
            info!("Broadcasting thread stopped for {}", target_addr);
        });

        Ok(server_addr)
    }
}
