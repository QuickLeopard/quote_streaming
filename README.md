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
RUST_LOG=info cargo run -p quote_streamer -- --host 127.0.0.1 --port 8080
```

Or from the `quote_streamer` directory:

```bash
RUST_LOG=info cargo run -- --host 127.0.0.1 --port 8080
```

Options:
- `-H, --host`: Server host address (required)
- `-p, --port`: Server port number (required)

Log levels:
- `RUST_LOG=error` - Only errors
- `RUST_LOG=warn` - Warnings and errors (includes timeout events)
- `RUST_LOG=info` - Info, warnings, and errors (recommended)
- `RUST_LOG=debug` - All messages including ping/pong and individual quotes

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
│  - Ping Sender (2s)      │
│  - CLI Interface         │
└──────────────────────────┘
```

## Ping/Pong Keep-Alive

The system implements a UDP-based ping/pong mechanism to detect client disconnections:

- **Client**: Sends "ping" message every 2 seconds to the server
- **Server**: Responds with "pong" and tracks last ping time
- **Timeout**: If no ping received for 5 seconds, server gracefully shuts down all threads for that client
- **Graceful Shutdown**: When server disconnects, client detects the error and exits cleanly

This ensures:
- Server resources are freed when clients disconnect unexpectedly
- No zombie connections or resource leaks
- Clean shutdown on both sides

## Dependencies

- **clap**: Command-line argument parsing
- **bus**: Pub-sub message bus implementation
- **socket2**: Low-level socket operations
- **bincode**: Binary serialization format
- **serde**: Serialization framework
- **crossbeam**: Concurrency utilities
- **libc**: C library bindings
- **log**: Logging facade
- **env_logger**: Logger implementation
- **chrono**: Date and time utilities

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
$ RUST_LOG=info cargo run -p quote_streamer -- --host 127.0.0.1 --port 8080
[2026-01-14 15:00:00 INFO  quote_streamer] Starting Quote Streamer listening on 127.0.0.1:7777
[2026-01-14 15:00:00 INFO  quote_streamer] Starting streaming for tickers: ["AAPL", "MSFT", ...]
[2026-01-14 15:00:05 INFO  quote_streamer::server] Streaming quotes for tickers: AAPL,MSFT,TSLA to address: 127.0.0.1:5555
[2026-01-14 15:00:05] Received ping from 127.0.0.1:5555
[2026-01-14 15:00:07] Received ping from 127.0.0.1:5555
...
```

Terminal 2 (Client):
```bash
$ cargo run -p quote_client -- --host 127.0.0.1 --port 8080 -A 127.0.0.1 --tickers AAPL,MSFT,TSLA
[2026-01-14 15:00:05] Connecting Quote Client to 127.0.0.1:7777 stream_addr: 127.0.0.1:5555 tickers: AAPL,MSFT,TSLA
Welcome to the Quote Streamer!
[2026-01-14 15:00:05] Connected to server!
[2026-01-14 15:00:05] Server response: Got STREAM command addr: udp://127.0.0.1:5555 tickers: AAPL,MSFT,TSLA server: 127.0.0.1:62005
[2026-01-14 15:00:05] Ресивер запущен на 127.0.0.1:5555
[2026-01-14 15:00:05] Ожидание данных...
[2026-01-14 15:00:06] StockQuote { ticker: "AAPL", price: 183.1325004178757, volume: 3980, timestamp: 1768380781617 }
[2026-01-14 15:00:07] StockQuote { ticker: "MSFT", price: 112.58418974325119, volume: 3924, timestamp: 1768380782415 }
[2026-01-14 15:00:07] StockQuote { ticker: "TSLA", price: 183.52005988007448, volume: 4200, timestamp: 1768380782759 }
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
- Keep-alive mechanisms (ping/pong)
- Graceful shutdown handling
- Logging with log crate
- Error handling without unwrap()

## License

Educational project
