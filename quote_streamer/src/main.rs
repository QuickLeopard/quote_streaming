
use std::net::TcpListener;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;

use crate::server::handle_client;

mod server;

use clap::Parser;

#[derive(Parser)]
#[command(name = "quote_streamer")]
#[command(about = "Quote Streamer")]
struct Cli {
    #[arg(short='H', long)]
    host: String,
    
    #[arg(short, long)]
    port: u16,
}


fn main() -> std::io::Result<()> {
    let cli = Cli::parse();
    let listener = TcpListener::bind (format! ("{}:{}", cli.host, cli.port))?;
    println!("Starting Quote Streamer listening on {}:{}", cli.host, cli.port);

    for stream in listener.incoming () {
        match stream {
            Ok (stream) => {
                thread::spawn (|| {
                    handle_client(stream);
                }
                );
            },
            Err(e) => eprintln!("Connection failed: {}", e)
        }
    }

    Ok (())
}
