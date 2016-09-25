//! # HTTP Request Parser
//!
//! The `Parser` converts octet streams into objects, octet by octet.
//! Can also convert objects back to octet streams.

// ****************************************************************************
//
// Imports
//
// ****************************************************************************

use std::collections::HashMap;
use std::str;

use super::{Method, Protocol};

// ****************************************************************************
//
// Public Types
//
// ****************************************************************************

/// An HTTP Request.
/// Fully describes the HTTP request sent from the client to the server.
#[derive(Debug)]
pub struct Request {
    /// The URL the client is requesting
    pub url: String,
    /// The method the client is requesting
    pub method: Method,
    /// The protocol the client is using in the request
    pub protocol: Protocol,
    /// Any headers supplied by the client in the request
    pub headers: HashMap<String, String>,
}

/// Contains the internal state for the parser.
#[derive(Debug)]
pub struct Parser {
    /// Our parser is stateful - incoming octets are handled based on the current state
    state: ParseState,
    /// Strings are collated into this temporary vector, until a seninel is seen
    temp: Vec<u8>,
    /// The URL in the request
    url: String,
    /// The method in the request
    method: Method,
    /// The protocol in the request
    protocol: Protocol,
    /// A collection of HTTP headers (key,value) pairs. We need them in-order
    /// as if the next line begins with a space, we need to append to the
    /// previous header's value.
    headers: Vec<(String, String)>,
    /// A temporary holder for the key while we read the value
    key: String,
}

/// Indicates whether the parser has seen enough, needs more data, or has abandoned the parse.
#[derive(Debug)]
pub enum ParseResult {
    /// Parse abandoned - there was an unspecified problem with the input
    Error,
    /// Didn't like one of the header names
    ErrorBadHeader,
    /// Didn't like one of the header values
    ErrorBadHeaderValue,
    /// Didn't like the method (e.g. GET)
    ErrorBadMethod,
    /// Didn't like the protocol (e.g. HTTP/1.1)
    ErrorBadProtocol,
    /// Didn't like the URL,
    ErrorBadURL,
    /// Parse in progress - need more input
    InProgress,
    /// Parse complete - request object available, and we also report
    /// the number of octets taken from the given buffer. If there
    /// are any octets remaining, they are probably body content.
    Complete(Request, usize),
}

// ****************************************************************************
//
// Private Types
//
// ****************************************************************************

#[derive(PartialEq, Debug)]
enum ParseState {
    Method,
    URL,
    Protocol,
    ProtocolEOL,
    KeyStart,
    Key,
    WrappedValue,
    WrappedValueStart,
    WrappedValueEOL,
    ValueStart,
    Value,
    ValueEOL,
    FinalEOL,
}

#[derive(Debug)]
enum CharType {
    Other,
    Space,
    Colon,
    CR,
    LF,
}

// ****************************************************************************
//
// Public Functions
//
// ****************************************************************************

impl Request {
    pub fn get_content_length(&self) -> Result<usize, &str> {
        match self.headers.get("Content-Length") {
            Some(value) => {
                match value.parse::<usize>() {
                    Ok(v) => Ok(v),
                    Err(_) => Err("Header valid invalid"),
                }
            }
            None => Err("Header Not Found"),
        }
    }
}

impl Parser {
    /// Ensures a default Parser can be created and that it has the correct
    /// starting values for a parse.
    pub fn new() -> Parser {
        Parser {
            state: ParseState::Method,
            temp: Vec::new(),
            url: String::new(),
            method: Method::Get,
            protocol: Protocol::Http10,
            headers: Vec::new(),
            key: String::new(),
        }
    }

    /// Perform the HTTP parse.
    /// This reads the buffer octet by octet, collating strings into
    /// temporary vectors. If any sort of error occurs, we bail out.
    pub fn parse(&mut self, buffer: &[u8]) -> ParseResult {
        let mut read = 0;
        for b in buffer {
            let c = *b;
            read = read + 1;
            let ct = get_char_type(c);
            // switch on state, then switch on char type
            match self.state {
                ParseState::Method => {
                    match ct {
                        CharType::Other => self.temp.push(c),
                        CharType::Space => {
                            match str::from_utf8(&self.temp) {
                                Ok(s) => {
                                    self.method = match s {
                                        "OPTIONS" => Method::Options,
                                        "GET" => Method::Get,
                                        "POST" => Method::Post,
                                        "PUT" => Method::Put,
                                        "DELETE" => Method::Delete,
                                        "HEAD" => Method::Head,
                                        "TRACE" => Method::Trace,
                                        "CONNECT" => Method::Connect,
                                        "PATCH" => Method::Patch,
                                        _ => return ParseResult::ErrorBadMethod,
                                    };
                                }
                                Err(_) => return ParseResult::ErrorBadMethod,
                            }
                            self.temp.clear();
                            self.state = ParseState::URL
                        }
                        CharType::Colon | CharType::CR | CharType::LF => return ParseResult::Error,
                    }
                }
                ParseState::URL => {
                    match ct {
                        CharType::Other | CharType::Colon => self.temp.push(c),
                        CharType::Space => {
                            match String::from_utf8(self.temp.split_off(0)) {
                                Ok(s) => self.url = s,
                                Err(_) => return ParseResult::ErrorBadURL,
                            }
                            self.state = ParseState::Protocol
                        }
                        CharType::CR | CharType::LF => return ParseResult::Error,
                    }
                }
                ParseState::Protocol => {
                    match ct {
                        CharType::Other => self.temp.push(c),
                        CharType::CR => {
                            match str::from_utf8(&self.temp) {
                                Ok("HTTP/1.0") => self.protocol = Protocol::Http10,
                                Ok("HTTP/1.1") => self.protocol = Protocol::Http11,
                                Ok(_) => return ParseResult::ErrorBadProtocol,
                                Err(_) => return ParseResult::ErrorBadProtocol,
                            }
                            self.temp.clear();
                            self.state = ParseState::ProtocolEOL
                        }
                        CharType::LF => {
                            match str::from_utf8(&self.temp) {
                                Ok("HTTP/1.0") => self.protocol = Protocol::Http10,
                                Ok("HTTP/1.1") => self.protocol = Protocol::Http11,
                                Ok(_) => return ParseResult::ErrorBadProtocol,
                                Err(_) => return ParseResult::ErrorBadProtocol,
                            }
                            self.temp.clear();
                            self.state = ParseState::KeyStart
                        }
                        CharType::Space | CharType::Colon => return ParseResult::ErrorBadProtocol,
                    }
                }
                ParseState::ProtocolEOL => {
                    match ct {
                        CharType::LF => self.state = ParseState::KeyStart,
                        _ => return ParseResult::Error,
                    }
                }
                ParseState::KeyStart => {
                    match ct {
                        CharType::Space => self.state = ParseState::WrappedValueStart,
                        CharType::LF => {
                            return ParseResult::Complete(self.build_request(), read);
                        }
                        CharType::CR => self.state = ParseState::FinalEOL,
                        CharType::Other => {
                            self.temp.push(c);
                            self.state = ParseState::Key
                        }
                        CharType::Colon => return ParseResult::Error,
                    }
                }
                ParseState::Key => {
                    match ct {
                        CharType::Other => self.temp.push(c),
                        CharType::Colon => {
                            match String::from_utf8(self.temp.split_off(0)) {
                                Ok(s) => self.key = s,
                                Err(_) => return ParseResult::ErrorBadHeader,
                            }
                            self.state = ParseState::ValueStart
                        }
                        CharType::Space | CharType::LF | CharType::CR => return ParseResult::Error,
                    }
                }
                ParseState::ValueStart => {
                    match ct {
                        CharType::Space => {}
                        CharType::Other => {
                            self.temp.push(c);
                            self.state = ParseState::Value
                        }
                        CharType::LF | CharType::CR | CharType::Colon => return ParseResult::Error,
                    }
                }
                ParseState::Value => {
                    match ct {
                        CharType::Other | CharType::Space | CharType::Colon => self.temp.push(c),
                        CharType::CR => {
                            match String::from_utf8(self.temp.split_off(0)) {
                                Ok(s) => {
                                    let hdr = (self.key.clone(), s);
                                    self.headers.push(hdr);
                                }
                                Err(_) => return ParseResult::ErrorBadHeaderValue,
                            }
                            self.state = ParseState::ValueEOL
                        }
                        CharType::LF => {
                            match String::from_utf8(self.temp.split_off(0)) {
                                Ok(s) => {
                                    let hdr = (self.key.clone(), s);
                                    self.headers.push(hdr);
                                }
                                Err(_) => return ParseResult::ErrorBadHeaderValue,
                            }
                            self.state = ParseState::KeyStart
                        }
                    }
                }
                ParseState::ValueEOL => {
                    match ct {
                        CharType::LF => self.state = ParseState::KeyStart,
                        _ => return ParseResult::Error,
                    }
                }
                ParseState::WrappedValueStart => {
                    match ct {
                        CharType::Space => {}
                        CharType::Other | CharType::Colon => {
                            self.temp.push(0x20); // single space
                            self.temp.push(c);
                            self.state = ParseState::WrappedValue
                        }
                        CharType::CR => self.state = ParseState::WrappedValueEOL,
                        CharType::LF => return ParseResult::Error,
                    }
                }
                ParseState::WrappedValue => {
                    match ct {
                        CharType::Other | CharType::Colon | CharType::Space => self.temp.push(c),
                        CharType::CR => {
                            match String::from_utf8(self.temp.split_off(0)) {
                                Ok(s) => {
                                    match self.headers.last_mut() {
                                        Some(x) => x.1.push_str(s.as_str()),
                                        None => return ParseResult::Error,
                                    }
                                }
                                Err(_) => return ParseResult::ErrorBadHeaderValue,
                            }
                            self.state = ParseState::WrappedValueEOL
                        }
                        CharType::LF => return ParseResult::Error,
                    }
                }
                ParseState::WrappedValueEOL => {
                    match ct {
                        CharType::LF => self.state = ParseState::KeyStart,
                        _ => return ParseResult::Error,
                    }
                }
                ParseState::FinalEOL => {
                    match ct {
                        CharType::LF => {
                            return ParseResult::Complete(self.build_request(), read);
                        }
                        _ => return ParseResult::Error,
                    }
                }
            }
        }
        ParseResult::InProgress
    }

    /// Construct the Request object based on what
    /// we've picked up so far.
    fn build_request(&mut self) -> Request {
        let mut r = Request {
            url: self.url.clone(),
            method: self.method,
            protocol: self.protocol,
            headers: HashMap::new()
        };
        for (k, v) in self.headers.drain(..) {
            r.headers.insert(k, v);
        }
        return r;
    }
}


// ****************************************************************************
//
// Private Functions
//
// ****************************************************************************


/// Map an octet (in US-ASCII) to a character
/// class, so we can decide what to do with it.
fn get_char_type(b: u8) -> CharType {
    if (b == 0x20) || (b == 0x09) {
        CharType::Space
    } else if b == 0x0D {
        CharType::CR
    } else if b == 0x0A {
        CharType::LF
    } else if b == 0x3A {
        CharType::Colon
    } else {
        CharType::Other
    }
}

// ****************************************************************************
//
// End Of File
//
// ****************************************************************************