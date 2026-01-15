use socket2::{Domain, Protocol, Socket, Type};
use std::io::{self, BufRead, BufReader, Write};
use std::net::{SocketAddr, TcpStream};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use std::time::{Duration};

use clap::Parser;
use quote_generator_lib::timestamp;

mod cli_args;
mod quote_udp_receiver;

#[cfg(test)]
mod tests;

const TCP_KEEPALIVE_TIME_SECS: u64 = 10;
const TCP_KEEPALIVE_INTERVAL_SECS: u64 = 5;
const TCP_READ_TIMEOUT_SECS: u64 = 5;

// Подключение к серверу
fn connect(host: &str, port: u16) -> io::Result<(TcpStream, BufReader<TcpStream>)> {
    let socket = Socket::new(Domain::IPV4, Type::STREAM, Some(Protocol::TCP))?;

    socket.set_keepalive(true)?;
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    {
        socket.set_tcp_keepalive(
            &socket2::TcpKeepalive::new()
                .with_time(Duration::from_secs(TCP_KEEPALIVE_TIME_SECS))
                .with_interval(Duration::from_secs(TCP_KEEPALIVE_INTERVAL_SECS)),
        )?;
    }

    let addr: SocketAddr = format!("{}:{}", host, port)
        .parse()
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;
    socket.connect(&addr.into())?;

    let stream: TcpStream = socket.into();
    stream.set_read_timeout(Some(Duration::from_secs(TCP_READ_TIMEOUT_SECS)))?;
    let mut reader = BufReader::new(stream.try_clone()?);

    // Читаем welcome message один раз
    let mut line = String::new();
    reader.read_line(&mut line)?;
    print!("{}", line);

    println!("[{}] Connected to server!", timestamp());
    Ok((stream, reader))
}

// Отправка команды
fn send_command(
    stream: &mut TcpStream,
    reader: &mut BufReader<TcpStream>,
    command: &str,
) -> io::Result<String> {
    stream.write_all(command.as_bytes())?;
    stream.write_all(b"\n")?;
    stream.flush()?;

    let mut buffer = String::new();
    let bytes = reader.read_line(&mut buffer)?;
    if bytes == 0 {
        return Err(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            "Server closed connection",
        ));
    }
    Ok(buffer)
}

fn main() -> io::Result<()> {
    let cli = cli_args::CliArgs::parse();
    
    // Setup Ctrl+C handler for graceful shutdown
    let shutdown = Arc::new(AtomicBool::new(false));
    let shutdown_clone = Arc::clone(&shutdown);
    
    ctrlc::set_handler(move || {
        println!("\n[{}] Ctrl+C received, shutting down...", timestamp());
        shutdown_clone.store(true, Ordering::Relaxed);
    }).expect("Error setting Ctrl+C handler");
    
    println!(
        "[{}] Connecting Quote Client to {}:{} stream_addr: {} tickers: {}",
        timestamp(),
        cli.host, cli.port, cli.stream_addr, cli.tickers
    );
    let (mut stream, mut reader) = connect(&cli.host, cli.port)?;

    let command = &format!("STREAM udp://{} {}", cli.stream_addr, cli.tickers);

    match send_command(&mut stream, &mut reader, command) {
        Ok(resp) => {
            print!("[{}] Server response: {}", timestamp(), resp);
            
            // Extract server address from response
            let server_addr = resp
                .split("server: ")
                .nth(1)
                .and_then(|s| s.split_whitespace().next())
                .unwrap_or(&cli.stream_addr);
            
            let quote_receiver = quote_udp_receiver::QuoteReceiver::new(&cli.stream_addr)?;
            if let Err(e) = quote_receiver.receive_loop(server_addr, shutdown) {
                eprintln!("[{}] Receive loop failed: {}", timestamp(), e);
            }
            println!("[{}] Client shutdown complete", timestamp());
        }
        Err(e) => {
            eprintln!("[{}] Command failed: {}.", timestamp(), e);
        }
    }

    Ok(())
}
