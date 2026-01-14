use bus::Bus;
use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;
use std::sync::Arc;
use std::sync::Mutex;
use chrono::Local;

use quote_generator_lib::core::StockQuote;

use crate::quote_udp_sender::{QuoteSender};

/// Initiates quote streaming to a client address
/// 
/// Returns the server's socket address on success, None on failure
fn stream_quotes(addr: &str, tickers: &str, bus: Arc<Mutex<Bus<StockQuote>>>) -> Option<String> {
    let addr = addr.to_string().clone();
    let tickers = tickers.to_string().clone();

    println!(
        "[{}] Streaming quotes for tickers: {} to address: {}",
        Local::now().format("%Y-%m-%d %H:%M:%S"),
        tickers, addr
    );

    match QuoteSender::new("0.0.0.0:0") {
        Ok(quote_sender) => {
            match quote_sender.start_broadcasting_with_bus(addr, tickers, bus) {
                Ok(server_addr) => Some(server_addr),
                Err(e) => {
                    eprintln!("[{}] Failed to start broadcasting: {}", Local::now().format("%Y-%m-%d %H:%M:%S"), e);
                    None
                }
            }
        }
        Err(e) => {
            eprintln!("[{}] Failed to create QuoteSender: {}", Local::now().format("%Y-%m-%d %H:%M:%S"), e);
            None
        }
    }
}

/// Handles a connected TCP client, processing commands and managing quote streams
pub fn handle_client(stream: TcpStream, bus: Arc<Mutex<Bus<StockQuote>>>) {
    let mut writer = stream.try_clone().expect("failed to clone stream");
    let mut reader = BufReader::new(stream);

    // send initial prompt
    let _ = writer.write_all(b"Welcome to the Quote Streamer!\n");
    let _ = writer.flush();

    let mut line = String::new();
    loop {
        line.clear();
        match reader.read_line(&mut line) {
            Ok(0) => {
                // EOF — клиент закрыл соединение
                return;
            }
            Ok(_) => {
                let input = line.trim();
                if input.is_empty() {
                    let _ = writer.flush();
                    continue;
                }

                let uppercased = line.to_uppercase();

                let mut parts = uppercased.split_whitespace();
                let response = match parts.next() {
                    Some("HELLO") => "Hi, there!\n",
                    
                    Some("STREAM") => {
                        let addr = parts.next();
                        let tickers = parts.next();
                        match (addr, tickers) {
                            (Some(addr), Some(tickers)) if addr.starts_with("UDP://") => {
                                let bus0 = Arc::clone(&bus);
                                match stream_quotes(&addr[6..], tickers, bus0) {
                                    Some(server_addr) => &format!(
                                        "Got STREAM command addr: {} tickers: {} server: {}\n",
                                        addr.to_lowercase(),
                                        tickers,
                                        server_addr
                                    ),
                                    None => "ERROR: Failed to start streaming\n",
                                }
                            }
                            _ => "ERROR: use like 'STREAM udp://127.0.0.1:1234 AAPL,TSLA'\n",
                        }
                    }
                    _ => "Unknown command!\n",
                };

                let _ = writer.write_all(response.as_bytes());
                let _ = writer.flush();
            }
            Err(_) => {
                // ошибка чтения — закрываем
                return;
            }
        }
    }
}
