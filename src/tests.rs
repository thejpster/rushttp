
use http_parser::*;

#[test]
fn complete_header() {
    let mut ctx:ParseContext = Default::default();
    let test = "GET /index.html HTTP/1.1\r\nUser-Agent: rust test\r\nHost: localhost\r\n\r\n".as_bytes();
    match parse_header(&mut ctx, test) {
        ParseResult::Complete(r) => {
            assert_eq!(r.method, HttpMethod::GET);
            assert_eq!(r.url, "/index.html");
            assert_eq!(r.version, "HTTP/1.1");
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
