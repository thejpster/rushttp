//! # The rushttp Rust HTTP Library - Unit Tests
//!
//! Unit tests for the rushttp library.

// ****************************************************************************
//
// Imports
//
// ****************************************************************************

use http_parser::*;


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
    let mut ctx = ParseContext::new();
    let test = b"GET /index.html HTTP/1.1\r\nUser-Agent: rust test\r\nHost: localhost\r\n\r\n";
    match ctx.parse_header(test) {
        ParseResult::Complete(r, c) => {
            assert_eq!(test.len() - c, 0);
            assert_eq!(r.method, HttpMethod::GET);
            assert_eq!(r.url, "/index.html");
            assert_eq!(r.protocol, "HTTP/1.1");
            assert_eq!(r.headers.len(), 2);
            assert_eq!(r.headers["User-Agent"], String::from("rust test"));
            assert_eq!(r.headers["Host"], String::from("localhost"));
        }
        _ => assert!(false),
    }
}

#[test]
fn get_complete_wrapped_header() {
    let mut ctx = ParseContext::new();
    let test = b"GET /index.html HTTP/1.1\r\nUser-Agent: rust test\r\n\t\tis the best \
                test\r\nHost: localhost\r\n\r\n";
    match ctx.parse_header(test) {
        ParseResult::Complete(r, c) => {
            assert_eq!(test.len() - c, 0);
            assert_eq!(r.method, HttpMethod::GET);
            assert_eq!(r.url, "/index.html");
            assert_eq!(r.protocol, "HTTP/1.1");
            assert_eq!(r.headers.len(), 2);
            assert_eq!(r.headers["User-Agent"], String::from("rust test is the best test"));
            assert_eq!(r.headers["Host"], String::from("localhost"));
        }
        _ => assert!(false),
    }
}

#[test]
fn put_complete_header() {
    let mut ctx = ParseContext::new();
    match ctx.parse_header(b"PUT ") {
        ParseResult::InProgress => {},
        _ => panic!()
    }
    let test = "/v1/api/frob?foo=bar HTTP/1.0\r\nUser-Agent: rust test\r\nHost: \
                localhost\r\nContent-Length: 12\r\n\r\nFlibble 💖"
                   .as_bytes();
    match ctx.parse_header(test) {
        ParseResult::Complete(r, c) => {
            assert_eq!(test.len() - c, 12);
            assert_eq!(r.method, HttpMethod::PUT);
            assert_eq!(r.url, "/v1/api/frob?foo=bar");
            assert_eq!(r.protocol, "HTTP/1.0");
            assert_eq!(r.headers.len(), 3);
            assert_eq!(r.headers["Content-Length"], String::from("12"));
            assert_eq!(r.headers["User-Agent"], String::from("rust test"));
            assert_eq!(r.headers["Host"], String::from("localhost"));
            let r = r.get_content_length().unwrap();
            assert_eq!(r, 12);
        }
        _ => assert!(false),
    }
}

#[test]
fn incomplete_header() {
    let mut ctx = ParseContext::new();
    let test = "GET /index.html HTTP/1.1\r\nUser-Agent: rust test\r\nHost: localhost\r\n"
                   .as_bytes();
    match ctx.parse_header(test) {
        ParseResult::InProgress => {}
        _ => assert!(false),
    }
}

#[test]
fn bad_method() {
    let mut ctx = ParseContext::new();
    let test = "GETA /index.html HTTP/1.1\r\nUser-Agent: rust test\r\nHost: localhost\r\n"
                   .as_bytes();
    match ctx.parse_header(test) {
        ParseResult::Error => {}
        _ => assert!(false),
    }
}

#[test]
fn bad_header() {
    let mut ctx = ParseContext::new();
    let test = "GETA /index.html HTTP/1.1\r\nUser-Agent: rust test\r\nHost\r\n\r\n".as_bytes();
    match ctx.parse_header(test) {
        ParseResult::Error => {}
        _ => assert!(false),
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
