use std::net::TcpStream;
use std::io::{BufReader, BufRead, Write};

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
                                &format!("Got STREAM command addr: {} tickers: {}\n", addr.to_lowercase(), tickers)
                            }
                            _ => "ERROR: use STREAM udp://127.0.0.1:1234 AAPL,TSLA\n"
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