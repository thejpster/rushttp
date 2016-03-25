//! Contains a basic HTTP server, built using rushttp

// ****************************************************************************
//
// Imports
//
// ****************************************************************************

// Use our own library
extern crate rushttp;

use rushttp::http_request::*;

use std::io::prelude::*;
use std::net::{TcpListener, TcpStream, Shutdown};
use std::thread;
use std::time::Duration;

// ****************************************************************************
//
// Public Types
//
// ****************************************************************************

// None

// ****************************************************************************
//
// Private Types
//
// ****************************************************************************

// None

// ****************************************************************************
//
// Public Functions
//
// ****************************************************************************

/// Program entry point. Starts an HTTP server on port 8000.
fn main() {
    println!("rushttpd - an experimental rust-based HTTP server.");

    // 1. Handle arguments
    // 2. Bind socket
    // 3. Handle connections as they come
    // 4. Clean up gracefully

    let listener = TcpListener::bind("0.0.0.0:8000").unwrap();
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                thread::spawn(move || handle_client(stream));
            }
            Err(e) => {
                println!("Connection failed!: {}", e);
            }
        }
    }

    drop(listener);
}

// ****************************************************************************
//
// Private Functions
//
// ****************************************************************************

/// This function is started in a new thread for every incoming connection.
fn handle_client(mut stream: TcpStream) {
    println!("+conn on {:?}!", stream);
    stream.set_read_timeout(Some(Duration::new(10, 0))).unwrap();
    let request = read_request(&mut stream);
    // Send response
    match request {
        Some(request) => render_response(&mut stream, request),
        None => render_error(&mut stream, 500, "Invalid request"),
    }
    stream.shutdown(Shutdown::Both).unwrap();
    println!("-conn on {:?}!", stream);
}

/// Process the incoming HTTP request
fn read_request(stream: &mut TcpStream) -> Option<HttpRequest> {
    let mut ctx: HttpRequestParser = HttpRequestParser::new();
    loop {
        let mut buffer: [u8; 8] = [0; 8];
        match stream.read(&mut buffer) {
            Ok(_) => {
                match ctx.parse(&buffer) {
                    ParseResult::Complete(r, _) => {
                        println!("<request {:?}: {:?}", stream, r);
                        return Some(r);
                    }
                    ParseResult::InProgress => {}
                    ParseResult::Error => break,
                }
            }
            Err(e) => {
                println!("err {:?}: {}", stream, e);
                return None;
            }
        }
    }
    None
}

/// Send back a noddy response based on the request
fn render_response(stream: &mut TcpStream, request: HttpRequest) {
    stream.write(b"HTTP/1.0 200 OK\r\n").unwrap();
    stream.write(b"Content-Type: text/plain; charset=utf-8\r\n\r\n").unwrap();
    stream.write(b"This is a test.\r\n").unwrap();
    let msg = format!("You asked for URL {}\r\n", request.url);
    stream.write(msg.as_bytes()).unwrap();
    for (k, v) in request.headers {
        let msg = format!("Key '{}' = '{}'\r\n", k, v);
        stream.write(msg.as_bytes()).unwrap();
    }
}

fn render_error(stream: &mut TcpStream, error_code: u32, error: &str) {
    stream.write(b"HTTP/1.0 200 OK\r\nContent-Type: text/plain; charset=utf-8\r\n\r\n").unwrap();
    let msg = format!("Error {0}: {1}", error_code, error);
    stream.write(msg.as_bytes()).unwrap();
    stream.write(b"\r\n").unwrap();
}


// ****************************************************************************
//
// End Of File
//
// ****************************************************************************
