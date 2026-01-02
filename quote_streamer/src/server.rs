use std::thread;
use std::net::TcpStream;
use std::io::{BufReader, BufRead, Write};

use crate::quote_udp_sender::{self, QuoteSender};

fn stream_quotes (addr: &str, tickers: &str) -> std::thread::JoinHandle<()> {
    let addr = addr.to_string().clone ();
    let tickers = tickers.to_string().clone ();
    // Implementation for streaming quotes
    let handle = thread::spawn(move || {
        // Simulate streaming quotes to the given UDP address
        println!("Streaming quotes for {} to {}", tickers, addr);

        //quote_udp_sender.start_broadcasting(addr.clone(), 1000);

        match QuoteSender::new("0.0.0.0:0") {
            Ok(quote_sender) => {
                if let Err(e) = quote_sender.start_broadcasting(addr, 1000) {
                    eprintln!("Failed to start broadcasting: {}", e);
                }
            }
            Err(e) => {
                eprintln!("Failed to create QuoteSender: {}", e);
            }
        }
    });
    handle
}

pub fn handle_client (stream: TcpStream) {
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
                    Some("HELLO") => {
                        "Hi, there!\n"
                    }
                    //STREAM udp://127.0.0.1:34254 AAPL,TSLA.
                    Some("STREAM") => {
                        let addr = parts.next ();
                        let tickers = parts.next ();
                        match (addr, tickers) {
                            (Some(addr), Some(tickers)) if addr.starts_with("UDP://") => {
                                let _handle = stream_quotes (&addr[6..], tickers);
                                &format!("Got STREAM command addr: {} tickers: {}\n", addr.to_lowercase(), tickers)
                            }
                            _ => "ERROR: use like STREAM udp://127.0.0.1:1234 AAPL,TSLA\n"
                        }
                    }
                    _ => {
                        "Unknown command!\n"
                    }
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