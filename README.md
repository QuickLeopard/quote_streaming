# Quote Streaming

A Rust-based system for streaming real-time stock quotes via TCP and UDP protocols. This project consists of three main components that work together to generate, transmit, and receive stock market data.

## Project Structure

The project is organized as a Rust workspace with three crates:

### 1. **quote_generator_lib**
A library crate that provides core functionality for generating and managing stock quotes.
- `QuoteGenerator`: Generates realistic stock quotes with random price movements
- `StockQuote`: Data structure representing a single stock quote with price and volume information
- Timestamp utilities for tracking when quotes were generated

### 2. **quote_streamer**
A TCP server that streams stock quotes to connected clients using a pub-sub architecture via the `bus` crate.
- Accepts TCP connections from clients
- Publishes real-time stock quotes over UDP to all connected clients
- Configurable host and port via CLI arguments
- Built-in streaming with configurable update intervals

### 3. **quote_client**
A CLI client that connects to the quote streamer and receives live stock quotes.
- TCP connection to the quote server
- UDP receiver for real-time quote updates
- Command-line interface for connecting to the server

## Building

```bash
# Build all crates
cargo build

# Build specific crate
cargo build -p quote_streamer
cargo build -p quote_client
cargo build -p quote_generator_lib
```

## Running

### Start the Quote Streamer Server

```bash
cargo run -p quote_streamer -- --host 127.0.0.1 --port 8080
```

Or from the `quote_streamer` directory:

```bash
cargo run -- --host 127.0.0.1 --port 8080
```

Options:
- `-H, --host`: Server host address (required)
- `-p, --port`: Server port number (required)

### Connect a Quote Client

In another terminal:

```bash
cargo run -p quote_client -- --host 127.0.0.1 --port 8080 -A 127.0.0.1 --tickers AAPL,MSFT,GOOGL
```

Or from the `quote_client` directory:

```bash
cargo run -- --host 127.0.0.1 --port 8080 -A 127.0.0.1 --tickers AAPL,MSFT,GOOGL
```

Options:
- `-H, --host`: Server host address (required)
- `-p, --port`: Server port number (required)
- `-A, --stream_addr`: UDP address to receive quotes on (required)
- `-T, --tickers`: Comma-separated list of stock tickers to subscribe to (required)

## Architecture

```
┌─────────────────────────┐
│  Quote Streamer Server  │
│  - TCP Listener         │
│  - Quote Generator      │
│  - Pub/Sub Bus          │
└────────────┬────────────┘
             │
      ┌──────┴──────┐
      │             │
   (TCP)         (UDP)
      │             │
      ▼             ▼
┌──────────────────────────┐
│   Quote Client           │
│  - TCP Connection        │
│  - UDP Quote Receiver    │
│  - CLI Interface         │
└──────────────────────────┘
```

## Dependencies

- **clap**: Command-line argument parsing
- **bus**: Pub-sub message bus implementation
- **socket2**: Low-level socket operations
- **bincode**: Binary serialization format
- **serde**: Serialization framework
- **crossbeam**: Concurrency utilities
- **libc**: C library bindings

## Stock Quote Format

Quotes are transmitted in the format:
```
{ ticker: String, price: f64, volume: u32, timestamp: u64 }
```

Example:
```
{ ticker: "AAPL", price: 183.1325004178757, volume: 3980, timestamp: 1768380781617 }
```

## Usage Example

Terminal 1 (Server):
```bash
$ cargo run -p quote_streamer -- --host 127.0.0.1 --port 8080
Starting Quote Streamer listening on 127.0.0.1:7777
Starting streaming for tickers: ["AAPL", "ABBV", "ABT", "ACN", "ADBE", "ADI", "ADP", "AEP", "AMGN", "AMT", "AMZN", "AON", "APTV", "AXP", "BDX", "BKNG", "BLK", "BMY", "BSX", "C", "CAT", "CI", "CL", "CMCSA", "CSCO", "COF", "COST", "CRM", "D", "DD", "DE", "DHR", "DIS", "DUK", "ECL", "EMR", "ETN", "EW", "FDX", "FIS", "FISV", "GE", "GILD", "GOOGL", "GS", "HD", "HON", "HUM", "ICE", "INTC", "INTU", "ISRG", "ITW", "JNJ", "JPM", "KLAC", "LLY", "LIN", "LMT", "LOW", "MCD", "MCO", "MDT", "MDLZ", "META", "MMM", "MO", "MS", "MSFT", "NEE", "NFLX", "NKE", "NOC", "NSC", "NVDA", "ORCL", "PEP", "PFE", "PG", "PGR", "PLD", "PNC", "PSA", "PYPL", "QCOM", "ROP", "RTX", "SBUX", "SCHW", "SHW", "SLB", "SO", "SPGI", "SYK", "T", "TGT", "TJX", "TMO", "TSLA", "TXN", "UNH", "UNP", "UPS", "USB", "V", "VRTX", "WM", "ZTS"]
Streaming quotes for tickers: AAPL,MSFT,TSLA to address: 127.0.0.1:5555
```

Terminal 2 (Client):
```bash
$ cargo run -p quote_client -- --host 127.0.0.1 --port 8080 -A 127.0.0.1 --tickers AAPL,MSFT,TSLA
Connecting Quote Client to 127.0.0.1:7777 stream_addr: 127.0.0.1:5555 tickers: AAPL,MSFT,TSLA
Welcome to the Quote Streamer!
Connected to server!
Server response: Got STREAM command addr: udp://127.0.0.1:5555 tickers: AAPL,MSFT,TSLA
Ресивер запущен на 127.0.0.1:5555
Ожидание данных...
StockQuote { ticker: "AAPL", price: 183.1325004178757, volume: 3980, timestamp: 1768380781617 }
StockQuote { ticker: "MSFT", price: 112.58418974325119, volume: 3924, timestamp: 1768380782415 }
StockQuote { ticker: "TSLA", price: 183.52005988007448, volume: 4200, timestamp: 1768380782759 }
StockQuote { ticker: "AAPL", price: 183.70661265927828, volume: 4079, timestamp: 1768380787876 }
StockQuote { ticker: "MSFT", price: 113.33087980964817, volume: 3937, timestamp: 1768380788662 }
StockQuote { ticker: "TSLA", price: 183.0703381506419, volume: 4258, timestamp: 1768380788997 }
StockQuote { ticker: "AAPL", price: 184.52031462143364, volume: 4112, timestamp: 1768380794109 }
StockQuote { ticker: "MSFT", price: 113.66540765863219, volume: 3992, timestamp: 1768380794888 }
...
```

## Development

This project is part of a Rust learning course and demonstrates:
- Rust workspace organization
- TCP/UDP networking
- Concurrent programming with threads
- Pub-sub message patterns
- CLI argument parsing
- Struct serialization

## License

Educational project
