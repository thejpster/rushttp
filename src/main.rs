//! Contains a basic HTTP server, built using rushttp

// ****************************************************************************
//
// Imports
//
// ****************************************************************************

// Use our own library
extern crate rushttp;

use rushttp::http_request::*;
use rushttp::http_response::*;

use std::collections::HashMap;
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
    match read_request(&mut stream) {
        Ok(r) => generate_response(&mut stream, r),
        Err(e) => render_parse_error(&mut stream, e),
    }
    stream.shutdown(Shutdown::Both).unwrap();
    println!("-conn on {:?}!", stream);
}

/// Process the incoming HTTP request
fn read_request(stream: &mut TcpStream) -> Result<HttpRequest, ParseResult> {
    let mut ctx: HttpRequestParser = HttpRequestParser::new();
    loop {
        let mut buffer: [u8; 8] = [0; 8];
        match stream.read(&mut buffer) {
            Ok(_) => {
                let r = ctx.parse(&buffer);
                match r {
                    ParseResult::Complete(req, _) => {
                        println!("<request {:?}: {:?}", stream, req);
                        return Ok(req);
                    }
                    ParseResult::InProgress => {}
                    _ => return Err(r),
                }
            }
            Err(e) => {
                println!("err {:?}: {}", stream, e);
                return Err(ParseResult::Error);
            }
        }
    }
}

/// Send back a noddy response based on the request
fn generate_response(stream: &mut TcpStream, request: HttpRequest) {
    let mut body:String = String::new();
    body.push_str("This is a test.\r\n");
    body.push_str(&format!("You asked for URL {}\r\n", request.url));
    for (k, v) in request.headers {
        body.push_str(&format!("Key '{}' = '{}'\r\n", k, v));
    }

    let mut response:HttpResponse = HttpResponse {
        status: HttpResponseStatus::OK,
        protocol: String::from("HTTP/1.1"),
        headers: HashMap::new(),
        body: body
    };
    response.headers.insert(String::from("Content-Type"), String::from("text/plain; charset=utf-8"));
    response.headers.insert(String::from("Connection"), String::from("close"));
    response.write(stream);
}

/// Handle a parsing error
fn render_parse_error(stream: &mut TcpStream, error: ParseResult) {
    match error {
        ParseResult::ErrorBadHeader => render_error(stream, HttpResponseStatus::BadRequest, "Bad Header"),
        ParseResult::ErrorBadHeaderValue => render_error(stream, HttpResponseStatus::BadRequest, "Bad Header Value"),
        ParseResult::ErrorBadMethod => render_error(stream, HttpResponseStatus::MethodNotAllowed, "Bad Method"),
        ParseResult::ErrorBadProtocol => render_error(stream, HttpResponseStatus::HTTPVersionNotSupported, "Bad Protocol"),
        ParseResult::ErrorBadURL => render_error(stream, HttpResponseStatus::BadRequest, "Bad URL"),
        _ => render_error(stream, HttpResponseStatus::BadRequest, "Unknown Error"),
    }
}

/// Send an error page
fn render_error(stream: &mut TcpStream, error_code: HttpResponseStatus, error_msg: &str) {
    let body = format!("Error {0}: {1}\r\n", error_code, error_msg);
    let mut response:HttpResponse = HttpResponse {
        status: error_code,
        protocol: String::from("HTTP/1.1"),
        headers: HashMap::new(),
        body: body
    };
    response.headers.insert(String::from("Content-Type"), String::from("text/plain; charset=utf-8"));
    response.headers.insert(String::from("Connection"), String::from("close"));
    response.write(stream);
}

// ****************************************************************************
//
// End Of File
//
// ****************************************************************************
