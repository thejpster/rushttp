//! # The rushttp Rust HTTP Library - Unit Tests
//!
//! Unit tests for the rushttp library.

// ****************************************************************************
//
// Imports
//
// ****************************************************************************

use super::request::*;
use super::*;

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

#[test]
fn get_complete_header() {
    let mut ctx = Parser::new();
    let test = b"GET /index.html HTTP/1.1\r\nUser-Agent: rust test\r\nHost: localhost\r\n\r\n";
    match ctx.parse(test) {
        ParseResult::Complete(r, c) => {
            assert_eq!(test.len() - c, 0);
            assert_eq!(r.method, http::method::GET);
            assert_eq!(r.url, "/index.html");
            assert_eq!(r.protocol, http::version::HTTP_11);
            assert_eq!(r.headers.len(), 2);
            assert_eq!(r.headers["User-Agent"], "rust test");
            assert_eq!(r.headers["Host"], "localhost");
        }
        _ => panic!(),
    }
}

#[test]
fn get_complete_header_no_cr() {
    let mut ctx = Parser::new();
    let test = b"GET /index.html HTTP/1.1\nUser-Agent: rust test\nHost: localhost\n\n";
    match ctx.parse(test) {
        ParseResult::Complete(r, c) => {
            assert_eq!(test.len() - c, 0);
            assert_eq!(r.method, http::method::GET);
            assert_eq!(r.url, "/index.html");
            assert_eq!(r.protocol, http::version::HTTP_11);
            assert_eq!(r.headers.len(), 2);
            assert_eq!(r.headers["User-Agent"], "rust test");
            assert_eq!(r.headers["Host"], "localhost");
        }
        _ => panic!(),
    }
}

#[test]
fn get_complete_header_some_cr() {
    let mut ctx = Parser::new();
    let test = b"GET /index.html HTTP/1.1\nUser-Agent:rust test\r\nHost:localhost\n\r\n";
    match ctx.parse(test) {
        ParseResult::Complete(r, c) => {
            assert_eq!(test.len() - c, 0);
            assert_eq!(r.method, http::method::GET);
            assert_eq!(r.url, "/index.html");
            assert_eq!(r.protocol, http::version::HTTP_11);
            assert_eq!(r.headers.len(), 2);
            assert_eq!(r.headers["User-Agent"], "rust test");
            assert_eq!(r.headers["Host"], "localhost");
        }
        _ => panic!(),
    }
}

#[test]
fn get_complete_wrapped_header() {
    let mut ctx = Parser::new();
    let test = b"GET /index.html HTTP/1.1\r\nUser-Agent: rust test\r\n\t\tis the best \
                test\r\nHost: localhost\r\n\r\n";
    match ctx.parse(test) {
        ParseResult::Complete(r, c) => {
            assert_eq!(test.len() - c, 0);
            assert_eq!(r.method, http::method::GET);
            assert_eq!(r.url, "/index.html");
            assert_eq!(r.protocol, http::version::HTTP_11);
            assert_eq!(r.headers.len(), 2);
            assert_eq!(r.headers["User-Agent"], "rust test is the best test");
            assert_eq!(r.headers["Host"], "localhost");
        }
        _ => panic!(),
    }
}

#[test]
fn put_complete_header() {
    let mut ctx = Parser::new();
    match ctx.parse(b"PUT ") {
        ParseResult::InProgress => {}
        _ => panic!(),
    }
    let test = "/v1/api/frob?foo=bar HTTP/1.0\r\nUser-Agent: rust test\r\nHost: \
                localhost\r\nContent-Length: 12\r\n\r\nFlibble 💖"
                   .as_bytes();
    match ctx.parse(test) {
        ParseResult::Complete(r, c) => {
            assert_eq!(test.len() - c, 12);
            assert_eq!(r.method, http::method::PUT);
            assert_eq!(r.url, "/v1/api/frob?foo=bar");
            assert_eq!(r.protocol, http::version::HTTP_10);
            assert_eq!(r.headers.len(), 3);
            assert_eq!(r.headers["Content-Length"], "12");
            assert_eq!(r.headers["User-Agent"], "rust test");
            assert_eq!(r.headers["Host"], "localhost");
            let r = r.get_content_length().unwrap();
            assert_eq!(r, 12);
        }
        _ => panic!(),
    }
}

#[test]
fn incomplete_header() {
    let mut ctx = Parser::new();
    let test = "GET /index.html HTTP/1.1\r\nUser-Agent: rust test\r\nHost: localhost\r\n"
                   .as_bytes();
    match ctx.parse(test) {
        ParseResult::InProgress => {}
        _ => panic!(),
    }
}

#[test]
fn bad_method() {
    let mut ctx = Parser::new();
    let test = "GET@ /index.html HTTP/1.1\r\nUser-Agent: rust test\r\nHost: localhost\r\n"
                   .as_bytes();
    match ctx.parse(test) {
        ParseResult::ErrorBadMethod => {}
        _ => panic!(),
    }
}

#[test]
fn bad_header() {
    let mut ctx = Parser::new();
    let test = b"GET /index.html HTTP/1.1\r\nUser-Agent: rust test\r\nHost\r\n\r\n";
    match ctx.parse(test) {
        ParseResult::Error => {}
        _ => panic!(),
    }
}

// ****************************************************************************
//
// Private Functions
//
// ****************************************************************************

// None

// ****************************************************************************
//
// End Of File
//
// ****************************************************************************
