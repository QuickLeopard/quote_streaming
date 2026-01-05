use socket2::{Domain, Protocol, Socket, Type};
use std::io::{self, BufRead, BufReader, Write};
use std::net::{SocketAddr, TcpStream};
//use std::sync::{Arc, Mutex};

use std::time::{Duration};

use clap::Parser;

mod cli_args;
mod quote_udp_receiver;

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
    let cli = cli_args::CliArgs::parse();
    println!(
        "Connecting Quote Client to {}:{} stream_addr: {} tickers: {}",
        cli.host, cli.port, cli.stream_addr, cli.tickers
    );
    let (mut stream, mut reader) = connect(&cli.host, cli.port)?;

    let command = &format!("STREAM udp://{} {}", cli.stream_addr, cli.tickers);

    match send_command(&mut stream, &mut reader, command) {
        Ok(resp) => {
            print!("Server response: {}", resp);
            let quote_receiver = quote_udp_receiver::QuoteReceiver::new(&cli.stream_addr)?;
            if let Err(e) = quote_receiver.receive_loop() {
                eprintln!("Receive loop failed: {}", e);
            }
        }
        Err(e) => {
            eprintln!("Command failed: {}.", e);
        }
    }

    loop {}

    Ok(())
}
