use bus::Bus;
use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;
use std::sync::Arc;
use std::sync::Mutex;

use quote_generator_lib::core::StockQuote;

use crate::quote_udp_sender::{QuoteSender};

fn stream_quotes(addr: &str, tickers: &str, bus: Arc<Mutex<Bus<StockQuote>>>) {
    //-> std::thread::JoinHandle<()> {

    let addr = addr.to_string().clone();
    let tickers = tickers.to_string().clone();

    // Implementation for streaming quotes
    //let handle = thread::spawn(move || {
    // Simulate streaming quotes to the given UDP address
    println!(
        "Streaming quotes for tickers: {} to address: {}",
        tickers, addr
    );

    //quote_udp_sender.start_broadcasting(addr.clone(), 1000);

    match QuoteSender::new("0.0.0.0:0") {
        Ok(quote_sender) => {
            if let Err(e) = quote_sender.start_broadcasting_with_bus(addr, tickers, bus) {
                eprintln!("Failed to start broadcasting: {}", e);
            }
        }
        Err(e) => {
            eprintln!("Failed to create QuoteSender: {}", e);
        }
    }
    //});
    //handle
}

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
                    //STREAM udp://127.0.0.1:34254 AAPL,TSLA.
                    Some("STREAM") => {
                        let addr = parts.next();
                        let tickers = parts.next();
                        match (addr, tickers) {
                            (Some(addr), Some(tickers)) if addr.starts_with("UDP://") => {
                                let bus0 = Arc::clone(&bus);
                                let _handle = stream_quotes(&addr[6..], tickers, bus0);
                                &format!(
                                    "Got STREAM command addr: {} tickers: {}\n",
                                    addr.to_lowercase(),
                                    tickers
                                )
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
