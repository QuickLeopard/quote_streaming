
use socket2::{Domain, Protocol, Socket, Type};
use std::io::{self, BufRead, BufReader, Write};
use std::net::{SocketAddr, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use clap::Parser;

mod quote_udp_receiver;

//use crate::quote_udp_receiver::{self, QuoteReceiver};

#[derive(Parser)]
#[command(name = "quote_client")]
#[command(about = "Quote Client")]
struct Cli {
    #[arg(short='H', long)]
    host: String,
    
    #[arg(short, long)]
    port: u16,
}

// Подключение к серверу
fn connect(host: &str, port: u16) -> io::Result<(TcpStream, BufReader<TcpStream>)> {
    let socket = Socket::new(Domain::IPV4, Type::STREAM, Some(Protocol::TCP))?;

    socket.set_keepalive(true)?;
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    {
        socket.set_tcp_keepalive(
            &socket2::TcpKeepalive::new()
                .with_time(Duration::from_secs(10))
                .with_interval(Duration::from_secs(5)),
        )?;
    }

    let addr: SocketAddr = format!("{}:{}", host, port).parse().unwrap();
    socket.connect(&addr.into())?;

    let stream: TcpStream = socket.into();
    stream.set_read_timeout(Some(Duration::from_secs(5)))?;
    let mut reader = BufReader::new(stream.try_clone()?);

    // Читаем welcome message один раз
    let mut line = String::new();
    reader.read_line(&mut line)?;
    print!("{}", line);

    println!("Connected to server!");
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
    let cli = Cli::parse();
    println!("Connecting Quote Client to {}:{}", cli.host, cli.port);
    let (mut stream, mut reader) = connect(&cli.host, cli.port)?;

    let addr = "127.0.0.1:12345";
    let command = "STREAM udp://127.0.0.1:12345 AAPL,TSLA";

    match send_command(&mut stream, &mut reader, command) {
        Ok(resp) => {
            print!("Server response: {}", resp);
            let quote_receiver = quote_udp_receiver::QuoteReceiver::new(addr)?;
            if let Err(e) = quote_receiver.receive_loop() {
                eprintln!("Receive loop failed: {}", e);
            }
        },
        Err(e) => {
            eprintln!("Command failed: {}.", e);               
        }
    }

    loop {

    }

    Ok(())
}
