//! # HTTP Request Parser
//!
//! The `Parser` converts octet streams into objects, octet by octet.
//! Can also convert objects back to octet streams.

// ****************************************************************************
//
// Imports
//
// ****************************************************************************

use std::str;

use http;

// ****************************************************************************
//
// Public Types
//
// ****************************************************************************

/// Our request type. We don't include the body in our request, so its type is set to `()`.
pub type Request = http::Request<()>;

/// Contains the internal state for the parser.
#[derive(Debug)]
pub struct Parser {
    /// Our parser is stateful - incoming octets are handled based on the current state
    state: ParseState,
    /// Strings are collated into this temporary vector, until a seninel is seen
    temp: Vec<u8>,
    /// The HTTP request builder
    builder: http::request::Builder,
    /// A collection of HTTP headers (key,value) pairs. We need them in-order
    /// as if the next line begins with a space, we need to append to the
    /// previous header's value.
    headers: Vec<(String, Vec<u8>)>,
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

pub fn get_content_length(r: &Request) -> Result<usize, &'static str> {
    match r.headers().get("Content-Length") {
        Some(value) => {
            match value.to_str() {
                Ok(s) => match s.parse::<usize>() {
                    Ok(v) => Ok(v),
                    Err(_) => Err("Header value invalid"),
                },
                Err(_) => Err("Header value invalid")
            }
        }
        None => Err("Header Not Found"),
    }
}

impl Parser {
    /// Ensures a default Parser can be created and that it has the correct
    /// starting values for a parse.
    pub fn new() -> Parser {
        Parser {
            state: ParseState::Method,
            temp: Vec::new(),
            headers: Vec::new(),
            builder: http::request::Builder::new(),
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
                            match http::Method::from_bytes(&self.temp) {
                                Ok(s) => self.builder.method(s),
                                Err(_) => return ParseResult::ErrorBadMethod,
                            };
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
                            match http::Uri::from_shared(self.temp.split_off(0).into()) {
                                Ok(s) => self.builder.uri(s),
                                Err(_) => return ParseResult::ErrorBadURL,
                            };
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
                                Ok("HTTP/1.0") => self.builder.version(http::Version::HTTP_10),
                                Ok("HTTP/1.1") => self.builder.version(http::Version::HTTP_11),
                                Ok(_) => return ParseResult::ErrorBadProtocol,
                                Err(_) => return ParseResult::ErrorBadProtocol,
                            };
                            self.temp.clear();
                            self.state = ParseState::ProtocolEOL
                        }
                        CharType::LF => {
                            match str::from_utf8(&self.temp) {
                                Ok("HTTP/1.0") => self.builder.version(http::Version::HTTP_10),
                                Ok("HTTP/1.1") => self.builder.version(http::Version::HTTP_11),
                                Ok(_) => return ParseResult::ErrorBadProtocol,
                                Err(_) => return ParseResult::ErrorBadProtocol,
                            };
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
                            match self.build_request() {
                                Ok(s) => return ParseResult::Complete(s, read),
                                Err(_) => return ParseResult::Error,
                            }
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
                            let hdr = (self.key.clone(), self.temp.split_off(0));
                            self.headers.push(hdr);
                            self.state = ParseState::ValueEOL
                        }
                        CharType::LF => {
                            let hdr = (self.key.clone(), self.temp.split_off(0));
                            self.headers.push(hdr);
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
                            match self.headers.last_mut() {
                                Some(x) => x.1.append(&mut self.temp),
                                None => return ParseResult::Error,
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
                            match self.build_request() {
                                Ok(s) => return ParseResult::Complete(s, read),
                                Err(_) => return ParseResult::Error,
                            }
                        }
                        _ => return ParseResult::Error,
                    }
                }
            }
        }
        ParseResult::InProgress
    }

    fn build_request(&mut self) -> Result<Request, ParseResult> {
        for (k, v) in self.headers.drain(..) {
            self.builder.header(&k[..], &v[..]);
        }
        self.builder.body(()).map_err(|_| ParseResult::Error)
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
