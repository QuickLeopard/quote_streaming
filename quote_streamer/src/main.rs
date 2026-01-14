use clap::Parser;
use log::{info, error};
use std::net::TcpListener;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;

use std::time::Duration;

use bus::Bus;
use quote_generator_lib::core::{QuoteGenerator, StockQuote};

use crate::server::handle_client;

mod quote_udp_sender;
mod server;
mod tickers;

#[cfg(test)]
mod tests;

const QUOTE_GENERATION_INTERVAL_MS: u64 = 5000;
const QUOTE_GENERATION_DELAY_MS: u64 = 10;
const BUS_CAPACITY: usize = 10;

#[derive(Parser)]
#[command(name = "quote_streamer")]
#[command(about = "Quote Streamer")]
struct Cli {
    #[arg(short = 'H', long)]
    host: String,

    #[arg(short, long)]
    port: u16,
}

fn streaming(tickers: Vec<String>, bus: Arc<Mutex<Bus<StockQuote>>>, interval_ms: u64) {
    let mut generator = QuoteGenerator::new();

    thread::spawn(move || {
        //let mut quote = StockQuote::new ("AAPL", 150.0, 1000, quote_generator_lib::get_current_timestamp());

        loop {
            for ticker in &tickers[..] {

                let quote = generator.generate_quote(ticker);

                if let Some(quote) = quote {
                    if let Ok(mut bus) = bus.lock() {
                        bus.broadcast(quote.clone());
                    }
                    thread::sleep(Duration::from_millis(QUOTE_GENERATION_DELAY_MS));
                }
            }
            thread::sleep(Duration::from_millis(interval_ms));
        }
    });
}

fn main() -> std::io::Result<()> {
    env_logger::init();
    
    let cli = Cli::parse();
    let listener = TcpListener::bind(format!("{}:{}", cli.host, cli.port))?;
    info!(
        "Starting Quote Streamer listening on {}:{}",
        cli.host, cli.port
    );

    let tickers = tickers::get_tickers();

    info!("Starting streaming for tickers: {:?}", tickers);

    // Create internal bus for StockQuote streaming to the UDP clients in single producer -> multiple consumers mode
    let bus: Arc<Mutex<Bus<StockQuote>>> = Arc::new(Mutex::new(Bus::new(BUS_CAPACITY)));

    let bus_clone0 = Arc::clone(&bus);
    streaming(
        tickers,
        bus_clone0,
        QUOTE_GENERATION_INTERVAL_MS,
    );

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let bus_clone1 = Arc::clone(&bus);
                thread::spawn(move || {
                    handle_client(stream, bus_clone1);
                });
            }
            Err(e) => error!("Connection failed: {}", e),
        }
    }

    //drop(bus);

    Ok(())
}
