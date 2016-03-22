// Contains a basic HTTP server, built using rushttp

// Use our own library
extern crate rushttp;

use rushttp::http_parser::*;

use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::time::Duration;

fn read_request(stream: &mut TcpStream) -> Option<HttpRequest> {
    let mut result:Option<HttpRequest> = None;
    let mut ctx:ParseContext = Default::default();
    loop {
        let mut buffer: [u8; 8] = [0; 8];
        match stream.read(&mut buffer) {
            Ok(len) => {
                println!("I got {len} chars", len=len);
                match ctx.parse_header(&buffer) {
                    ParseResult::Complete(r) => {
                        println!("Got request {:?}", r);
                        result = Some(r);
                        break;
                    },
                    ParseResult::InProgress => { },
                    ParseResult::Error => break
                }
            },
            Err(e) => {
                println!("read failure: {}", e);
                break;
            }
        }
    }
    result
}

fn render_response(stream: &mut TcpStream, request: HttpRequest) {
    stream.write(b"HTTP/1.0 200 OK\r\nContent-Type: text/plain; charset=utf-8\r\n\r\nThis is a test.\r\n").unwrap();
}

fn render_error(stream: &mut TcpStream, error_code: u32, error: &str) {
    stream.write(b"HTTP/1.0 200 OK\r\nContent-Type: text/plain; charset=utf-8\r\n\r\n").unwrap();
    let msg = format!("Error {0}: {1}", error_code, error);
    stream.write(msg.as_bytes()).unwrap();
    stream.write(b"\r\n").unwrap();
}

fn handle_client(mut stream: TcpStream) {
    println!("Got a connection on {:?}!", stream);
    stream.set_read_timeout(Some(Duration::new(10, 0))).unwrap();
    let request = read_request(&mut stream);
    // Send response
    match request {
        Some(request) => render_response(&mut stream, request),
        None => render_error(&mut stream, 500, "Invalid request")
    }
}

// Program entry point
fn main() {
    println!("rushttp - an experimental rust-based HTTP server.");

    // 1. Handle arguments
    // 2. Bind socket
    // 3. Handle connections as they come
    // 4. Clean up gracefully

    let listener = TcpListener::bind("0.0.0.0:8000").unwrap();
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                thread::spawn(move || {
                    handle_client(stream)
                });
            }
            Err(e) => {
                println!("Connection failed!: {}", e);
            }
        }
    }

    drop(listener);
}

///////////////////////////////////////////////////////////////////////////////
//
// End of File
//
///////////////////////////////////////////////////////////////////////////////
