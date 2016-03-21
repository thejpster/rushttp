// The rushttp HTTP parser.
// Converts octet streams into objects, octet by octet.
// Can also convert objects back to octet streams.

//
// Imports
//

//
// Public Types
//

#[derive(PartialEq)]
#[derive(Debug)]
pub enum HttpMethod {
    GET,
    POST,
    PUT,
    OPTION,
    HEAD
}

#[derive(Debug)]
pub enum HttpHeader {
    //CharSet(String),
    Unknown { key: String, value: String }
}

#[derive(Debug)]
pub struct HttpRequest {
    pub url: String,
    pub method: HttpMethod,
    pub version: String,
    pub headers: Vec<HttpHeader>
}

#[derive(Debug)]
#[derive(Default)]
pub struct ParseContext {
    state: ParseState,
    temp: Vec<u8>,
    url: String,
    method: HttpMethod,
    version: String,
    headers: Vec<HttpHeader>,
    key: String,
    value: String
}

#[derive(Debug)]
pub enum ParseResult {
    Error,
    InProgress,
    Complete(HttpRequest)
}

//
// Private Types
//

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

#[derive(Debug)]
enum CharType {
    Other,
    Space,
    Colon,
    CR,
    NL
}

//
// Public Functions
//

impl Default for HttpMethod {
    fn default() -> HttpMethod { HttpMethod::GET }
}

pub fn parse_header(ctx: &mut ParseContext, buffer: &[u8]) -> ParseResult {
    //let r = HttpRequest { url: "/index.html".to_string(), method: HttpMethod::GET, headers: Vec::new(), version: "1.1".to_string() };
    for b in buffer {
        let ct = get_char_type(*b);
        println!("Got char type {:?} in state {:?}", ct, ctx.state);
        // switch on state, then switch on char type
        match ctx.state {
            ParseState::Method => {
                match ct {
                    CharType::Other => ctx.temp.push(*b),
                    CharType::Space => {
                        match String::from_utf8(ctx.temp.split_off(0)) {
                            Ok(s) => {
                                match s.as_str() {
                                    "GET" => ctx.method = HttpMethod::GET,
                                    "POST" => ctx.method = HttpMethod::POST,
                                    "PUT" => ctx.method = HttpMethod::PUT,
                                    "OPTION" => ctx.method = HttpMethod::OPTION,
                                    "HEAD" => ctx.method = HttpMethod::HEAD,
                                    _ => return ParseResult::Error
                                }
                                println!("Got method {:?}", ctx.method)
                            },
                            _ => return ParseResult::Error
                        }
                        ctx.state = ParseState::URL
                    },
                    _ => return ParseResult::Error
                }
            },
            ParseState::URL => {
                match ct {
                    CharType::Other => ctx.temp.push(*b),
                    CharType::Colon => ctx.temp.push(*b),
                    CharType::Space => {
                        match String::from_utf8(ctx.temp.split_off(0)) {
                            Ok(s) => ctx.url = s,
                            _ => return ParseResult::Error
                        }
                        println!("Got URL {:?}", ctx.url);
                        ctx.state = ParseState::Version
                    }
                    _ => return ParseResult::Error
                }
            },
            ParseState::Version => {
                match ct {
                    CharType::Other => ctx.temp.push(*b),
                    CharType::CR => {
                        match String::from_utf8(ctx.temp.split_off(0)) {
                            Ok(s) => ctx.version = s,
                            _ => return ParseResult::Error
                        }
                        println!("Got version {:?}", ctx.version);
                        ctx.state = ParseState::VersionEOL
                    }
                    _ => return ParseResult::Error
                }
            },
            ParseState::VersionEOL => {
                match ct {
                    CharType::NL => ctx.state = ParseState::KeyStart,
                    _ => return ParseResult::Error
                }
            },
            ParseState::KeyStart => {
                match ct {
                    CharType::CR => ctx.state = ParseState::FinalEOL,
                    CharType::Other => {
                        ctx.temp.push(*b);
                        ctx.state = ParseState::Key
                    },
                    _ => return ParseResult::Error
                }
            },
            ParseState::Key => {
                match ct {
                    CharType::Other => ctx.temp.push(*b),
                    CharType::Colon => {
                        match String::from_utf8(ctx.temp.split_off(0)) {
                            Ok(s) => ctx.key = s,
                            _ => return ParseResult::Error
                        }
                        ctx.state = ParseState::Value
                    }
                    _ => return ParseResult::Error
                }
            },
            ParseState::Value => {
                match ct {
                    CharType::Other => ctx.temp.push(*b),
                    CharType::Space => ctx.temp.push(*b),
                    CharType::Colon => ctx.temp.push(*b),
                    CharType::CR => {
                        match String::from_utf8(ctx.temp.split_off(0)) {
                            Ok(s) => {
                                let hdr:HttpHeader = HttpHeader::Unknown { key: ctx.key.clone(), value: s };
                                println!("Got header {:?}", hdr);
                                ctx.headers.push(hdr);
                            },
                            _ => return ParseResult::Error
                        }
                        ctx.state = ParseState::ValueEOL
                    }
                    _ => return ParseResult::Error
                }
            },
            ParseState::ValueEOL => {
                match ct {
                    CharType::NL => ctx.state = ParseState::KeyStart,
                    _ => return ParseResult::Error
                }
            },
            ParseState::FinalEOL => {
                match ct {
                    CharType::NL => {
                        let r: HttpRequest = HttpRequest {
                            url: ctx.url.clone(),
                            method: HttpMethod::GET,
                            version: ctx.version.clone(),
                            headers: Vec::new(),
                        };
                        return ParseResult::Complete(r);
                    },
                    _ => return ParseResult::Error
                }
            }
        }
    }
    ParseResult::InProgress
}

//
// Private Functions
//

impl Default for ParseState {
    fn default() -> ParseState { ParseState::Method }
}

// Map an octet (in US-ASCII) to a character
// class, so we can decide what to do with it.
fn get_char_type(b: u8) -> CharType {
    let mut result: CharType = CharType::Other;
    if b == 0x20 {
        result = CharType::Space;
    } else if b == 0x0D {
        result = CharType::CR;
    } else if b == 0x0A {
        result = CharType::NL;
    } else if b == 0x3A {
        result = CharType::Colon;
    }
    result
}

//
// End of file
//
