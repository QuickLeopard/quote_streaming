
use std::net::TcpListener;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;

use crate::server::handle_client;

mod server;

fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind ("127.0.0.1:12345")?;
    println!("Hello, from server!");

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
