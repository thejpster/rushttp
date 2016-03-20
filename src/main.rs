use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::time::Duration;

#[derive(PartialEq)]
#[derive(Debug)]
enum HttpMethod {
    GET,
    POST,
    PUT,
    OPTION,
    HEAD
}

impl Default for HttpMethod {
    fn default() -> HttpMethod { HttpMethod::GET }
}

#[derive(Debug)]
enum HttpHeader {
    //CharSet(String),
    Unknown { key: String, value: String }
}

#[derive(Debug)]
struct HttpRequest {
    url: String,
    method: HttpMethod,
    version: String,
    headers: Vec<HttpHeader>
}

#[derive(PartialEq)]
#[derive(Debug)]
enum ParseState {
    Method,
    URL,
    Version,
    VersionEOL,
    KeyStart,
    Key,
    Value,
    ValueEOL,
    FinalEOL
}

impl Default for ParseState {
    fn default() -> ParseState { ParseState::Method }
}

#[derive(Debug)]
enum HttpCharType {
    Other,
    Space,
    Colon,
    CR,
    NL
}

#[derive(Debug)]
#[derive(Default)]
struct ParseContext {
    state: ParseState,
    temp: Vec<u8>,
    url: String,
    method: HttpMethod,
    version: String,
    headers: Vec<HttpHeader>,
    key: String,
    value: String
}

enum HttpParseResult {
    Error,
    InProgress,
    Complete(HttpRequest)
}

fn get_char_type(b: u8) -> HttpCharType {
    let mut result: HttpCharType = HttpCharType::Other;
    if b == 0x20 {
        result = HttpCharType::Space;
    } else if b == 0x0D {
        result = HttpCharType::CR;
    } else if b == 0x0A {
        result = HttpCharType::NL;
    } else if b == 0x3A {
        result = HttpCharType::Colon;
    }
    result
}

fn parse_header(ctx: &mut ParseContext, buffer: &[u8]) -> HttpParseResult {
    //let r = HttpRequest { url: "/index.html".to_string(), method: HttpMethod::GET, headers: Vec::new(), version: "1.1".to_string() };
    for b in buffer {
        let ct = get_char_type(*b);
        println!("Got char type {:?} in state {:?}", ct, ctx.state);
        // switch on state, then switch on char type
        match ctx.state {
            ParseState::Method => {
                match ct {
                    HttpCharType::Other => ctx.temp.push(*b),
                    HttpCharType::Space => {
                        match String::from_utf8(ctx.temp.split_off(0)) {
                            Ok(s) => {
                                match s.as_str() {
                                    "GET" => ctx.method = HttpMethod::GET,
                                    "POST" => ctx.method = HttpMethod::POST,
                                    "PUT" => ctx.method = HttpMethod::PUT,
                                    "OPTION" => ctx.method = HttpMethod::OPTION,
                                    "HEAD" => ctx.method = HttpMethod::HEAD,
                                    _ => return HttpParseResult::Error
                                }
                                println!("Got method {:?}", ctx.method)
                            },
                            _ => return HttpParseResult::Error
                        }
                        ctx.state = ParseState::URL
                    },
                    _ => return HttpParseResult::Error
                }
            },
            ParseState::URL => {
                match ct {
                    HttpCharType::Other => ctx.temp.push(*b),
                    HttpCharType::Colon => ctx.temp.push(*b),
                    HttpCharType::Space => {
                        match String::from_utf8(ctx.temp.split_off(0)) {
                            Ok(s) => ctx.url = s,
                            _ => return HttpParseResult::Error
                        }
                        println!("Got URL {:?}", ctx.url);
                        ctx.state = ParseState::Version
                    }
                    _ => return HttpParseResult::Error
                }
            },
            ParseState::Version => {
                match ct {
                    HttpCharType::Other => ctx.temp.push(*b),
                    HttpCharType::CR => {
                        match String::from_utf8(ctx.temp.split_off(0)) {
                            Ok(s) => ctx.version = s,
                            _ => return HttpParseResult::Error
                        }
                        println!("Got version {:?}", ctx.version);
                        ctx.state = ParseState::VersionEOL
                    }
                    _ => return HttpParseResult::Error
                }
            },
            ParseState::VersionEOL => {
                match ct {
                    HttpCharType::NL => ctx.state = ParseState::KeyStart,
                    _ => return HttpParseResult::Error
                }
            },
            ParseState::KeyStart => {
                match ct {
                    HttpCharType::CR => ctx.state = ParseState::FinalEOL,
                    HttpCharType::Other => {
                        ctx.temp.push(*b);
                        ctx.state = ParseState::Key
                    },
                    _ => return HttpParseResult::Error
                }
            },
            ParseState::Key => {
                match ct {
                    HttpCharType::Other => ctx.temp.push(*b),
                    HttpCharType::Colon => {
                        match String::from_utf8(ctx.temp.split_off(0)) {
                            Ok(s) => ctx.key = s,
                            _ => return HttpParseResult::Error
                        }
                        ctx.state = ParseState::Value
                    }
                    _ => return HttpParseResult::Error
                }
            },
            ParseState::Value => {
                match ct {
                    HttpCharType::Other => ctx.temp.push(*b),
                    HttpCharType::Space => ctx.temp.push(*b),
                    HttpCharType::Colon => ctx.temp.push(*b),
                    HttpCharType::CR => {
                        match String::from_utf8(ctx.temp.split_off(0)) {
                            Ok(s) => {
                                let hdr:HttpHeader = HttpHeader::Unknown { key: ctx.key.clone(), value: s };
                                println!("Got header {:?}", hdr);
                                ctx.headers.push(hdr);
                            },
                            _ => return HttpParseResult::Error
                        }
                        ctx.state = ParseState::ValueEOL
                    }
                    _ => return HttpParseResult::Error
                }
            },
            ParseState::ValueEOL => {
                match ct {
                    HttpCharType::NL => ctx.state = ParseState::KeyStart,
                    _ => return HttpParseResult::Error
                }
            },
            ParseState::FinalEOL => {
                match ct {
                    HttpCharType::NL => {
                        let r: HttpRequest = HttpRequest {
                            url: ctx.url.clone(),
                            method: HttpMethod::GET,
                            version: ctx.version.clone(),
                            headers: Vec::new(),
                        };
                        return HttpParseResult::Complete(r);
                    },
                    _ => return HttpParseResult::Error
                }
            }
        }
    }
    HttpParseResult::InProgress
}

#[cfg(test)]
mod tests {
    use super::parse_header;
    use super::HttpMethod;
    use super::ParseContext;
    use super::HttpParseResult;
    #[test]
    fn complete_header() {
        let mut ctx:ParseContext = Default::default();
        let test = "GET /index.html HTTP/1.1\r\nUser-Agent: rust test\r\nHost: localhost\r\n\r\n".as_bytes();
        match parse_header(&mut ctx, test) {
            HttpParseResult::Complete(r) => {
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
            HttpParseResult::InProgress => { },
            _ => assert!(false)
        }
    }
    #[test]
    fn bad_method() {
        let mut ctx:ParseContext = Default::default();
        let test = "GETA /index.html HTTP/1.1\r\nUser-Agent: rust test\r\nHost: localhost\r\n".as_bytes();
        match parse_header(&mut ctx, test) {
            HttpParseResult::Error => { },
            _ => assert!(false)
        }
    }
    #[test]
    fn bad_header() {
        let mut ctx:ParseContext = Default::default();
        let test = "GETA /index.html HTTP/1.1\r\nUser-Agent: rust test\r\nHost\r\n\r\n".as_bytes();
        match parse_header(&mut ctx, test) {
            HttpParseResult::Error => { },
            _ => assert!(false)
        }
    }
}

fn read_request(stream: &mut TcpStream) -> Option<HttpRequest> {
    let result:Option<HttpRequest> = None;
    loop {
        let mut buffer: [u8; 8] = [0; 8];
        match stream.read(&mut buffer) {
            Ok(len) => {
                println!("I got {len} chars", len=len);
                //let done = process(&mut state, buffer);
            },
            Err(e) => {
                println!("read failure: {}", e);
                break;
            }
        }
        // Process characters through state machine
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
