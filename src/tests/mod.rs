//! # The rushttp Rust HTTP Library - Unit Tests
//!
//! Unit tests for the rushttp library.

use http_parser::*;

#[test]
fn get_complete_header() {
    let mut ctx:ParseContext = Default::default();
    let test = "GET /index.html HTTP/1.1\r\nUser-Agent: rust test\r\nHost: localhost\r\n\r\n".as_bytes();
    match parse_header(&mut ctx, test) {
        ParseResult::Complete(r) => {
            assert_eq!(r.method, HttpMethod::GET);
            assert_eq!(r.url, "/index.html");
            assert_eq!(r.protocol, "HTTP/1.1");
            assert_eq!(r.headers.len(), 2);
            assert_eq!(r.headers[0], HttpHeader::Unknown { key: String::from("User-Agent"), value: String::from("rust test") });
            assert_eq!(r.headers[1], HttpHeader::Unknown { key: String::from("Host"), value: String::from("localhost") });
        },
        _ => assert!(false)
    }
}

#[test]
fn put_complete_header() {
    let mut ctx:ParseContext = Default::default();
    let test = "PUT /v1/api/frob?foo=bar HTTP/1.0\r\nUser-Agent: rust test\r\nHost: localhost\r\n\r\n".as_bytes();
    match parse_header(&mut ctx, test) {
        ParseResult::Complete(r) => {
            assert_eq!(r.method, HttpMethod::PUT);
            assert_eq!(r.url, "/v1/api/frob?foo=bar");
            assert_eq!(r.protocol, "HTTP/1.0");
            assert_eq!(r.headers.len(), 2);
            assert_eq!(r.headers[0], HttpHeader::Unknown { key: String::from("User-Agent"), value: String::from("rust test") });
            assert_eq!(r.headers[1], HttpHeader::Unknown { key: String::from("Host"), value: String::from("localhost") });
        },
        _ => assert!(false)
    }
}

#[test]
fn incomplete_header() {
    let mut ctx:ParseContext = Default::default();
    let test = "GET /index.html HTTP/1.1\r\nUser-Agent: rust test\r\nHost: localhost\r\n".as_bytes();
    match parse_header(&mut ctx, test) {
        ParseResult::InProgress => { },
        _ => assert!(false)
    }
}

#[test]
fn bad_method() {
    let mut ctx:ParseContext = Default::default();
    let test = "GETA /index.html HTTP/1.1\r\nUser-Agent: rust test\r\nHost: localhost\r\n".as_bytes();
    match parse_header(&mut ctx, test) {
        ParseResult::Error => { },
        _ => assert!(false)
    }
}

#[test]
fn bad_header() {
    let mut ctx:ParseContext = Default::default();
    let test = "GETA /index.html HTTP/1.1\r\nUser-Agent: rust test\r\nHost\r\n\r\n".as_bytes();
    match parse_header(&mut ctx, test) {
        ParseResult::Error => { },
        _ => assert!(false)
    }
}

///////////////////////////////////////////////////////////////////////////////
//
// End of File
//
///////////////////////////////////////////////////////////////////////////////
