//! Contains a basic HTTP server, built using rushttp

// ****************************************************************************
//
// Imports
//
// ****************************************************************************

// Use our own library
extern crate rushttp;

extern crate http;

use rushttp::request::*;
use rushttp::response::*;

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
// Private Data
//
// ****************************************************************************

const TCP_READ_TIMEOUT_SECONDS: u64 = 300;

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
    println!("Listening on 0.0.0.0:8000.");
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
    if let Ok(_) = stream.set_read_timeout(Some(Duration::from_secs(TCP_READ_TIMEOUT_SECONDS))) {
        match read_request(&mut stream) {
            Ok(r) => generate_response(&mut stream, r),
            Err(e) => render_parse_error(&mut stream, e),
        }
    }
    stream.shutdown(Shutdown::Both).unwrap();
    println!("-conn on {:?}!", stream);
}

/// Process the incoming HTTP request
fn read_request(stream: &mut TcpStream) -> Result<Request, ParseResult> {
    let mut ctx: Parser = Parser::new();
    loop {
        let mut buffer = vec![0; 1024];
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
fn generate_response(stream: &mut TcpStream, request: Request) {
    if *request.method() == http::Method::GET {
        let mut body: String = String::new();
        body.push_str("This is a test.\r\n");
        body.push_str(&format!("You asked for URL {}\r\n", request.uri()));
        body.push_str(&format!("You are stream {:?}\r\n", stream));
        for (k, v) in request.headers() {
            body.push_str(&format!("Key {:?} = {:?}\r\n", k, v));
        }

        let mut response = HttpResponse::new_with_body(HttpResponseStatus::OK, "HTTP/1.1", body);
        response.add_header("Content-Type", "text/plain; charset=utf-8");
        response.add_header("Connection", "close");
        response.write(stream).unwrap();
    } else {
        render_error(stream,
                     HttpResponseStatus::MethodNotAllowed,
                     &format!("Method {:?} not allowed.", request.method()));
    }
}

/// Handle a parsing error
fn render_parse_error(stream: &mut TcpStream, error: ParseResult) {
    let (status, msg) = match error {
        ParseResult::ErrorBadHeader => (HttpResponseStatus::BadRequest, "Bad Header"),
        ParseResult::ErrorBadHeaderValue => (HttpResponseStatus::BadRequest, "Bad Header Value"),
        ParseResult::ErrorBadMethod => (HttpResponseStatus::MethodNotAllowed, "Bad Method"),
        ParseResult::ErrorBadProtocol => {
            (HttpResponseStatus::HTTPVersionNotSupported, "Bad Protocol")
        }
        ParseResult::ErrorBadURL => (HttpResponseStatus::BadRequest, "Bad URL"),
        _ => (HttpResponseStatus::BadRequest, "Unknown Error"),
    };
    render_error(stream, status, msg);
}

/// Send an error page
fn render_error(stream: &mut TcpStream, error_code: HttpResponseStatus, error_msg: &str) {
    let body = format!("Error {0}: {1}\r\n", error_code, error_msg);
    let mut response = HttpResponse::new_with_body(error_code, "HTTP/1.1", body);
    response.add_header("Content-Type", "text/plain; charset=utf-8");
    response.add_header("Connection", "close");
    response.write(stream).unwrap();
}

// ****************************************************************************
//
// End Of File
//
// ****************************************************************************
