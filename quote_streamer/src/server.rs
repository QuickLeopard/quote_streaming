use std::net::TcpStream;
use std::io::{BufReader, Write};

pub fn handle_client (stream: TcpStream) {
    let mut writer = stream.try_clone().expect("failed to clone stream");
    let mut reader = BufReader::new(stream);

    // send initial prompt
    let _ = writer.write_all(b"Welcome to the Quote Streamer!\n");
    let _ = writer.flush();
}