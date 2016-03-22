//! # The rushttp Rust HTTP Library - HTTP Parser
//!
//! The HTTP Parser converts octet streams into objects, octet by octet.
//! Can also convert objects back to octet streams.

//
// Imports
//

//
// Public Types
//

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum HttpMethod {
	GET,
	POST,
	PUT,
	OPTION,
	HEAD
}

#[derive(Debug, PartialEq)]
pub enum HttpHeader {
	//More common header types here. Is this useful?
	//CharSet(String),
	Unknown { key: String, value: String }
}

#[derive(Debug)]
pub struct HttpRequest {
	pub url: String,
	pub method: HttpMethod,
	pub protocol: String,
	pub headers: Vec<HttpHeader>
}

/// Contains the internal state for the parser. Must be given
/// to the parse_header function.
#[derive(Debug)]
pub struct ParseContext {
	/// Our parser is stateful - incoming octets are handled based on the current state
	state: ParseState,
	/// Strings are collated into this temporary vector, until a seninel is seen
	temp: Vec<u8>,
	/// The URL in the request
	url: String,
	/// The method in the request
	method: HttpMethod,
	/// The protocol in the request
	protocol: String,
	/// A collection of HTTP headers (key,value) pairs
	headers: Vec<HttpHeader>,
	/// A temporary holder for the key while we read the value
	key: String,
}

/// Indicates whether the parser has seen enough, needs more data, or has abandoned the parse.
#[derive(Debug)]
pub enum ParseResult {
	/// Parse abandoned - there was a problem with the input
	Error,
	/// Parse in progress - need more input
	InProgress,
	/// Parse complete - request object available
	Complete(HttpRequest)
}

///////////////////////////////////////////////////////////////////////////////
//
// Private Types
//
///////////////////////////////////////////////////////////////////////////////

#[derive(PartialEq, Debug)]

enum ParseState {
	Method,
	URL,
	Version,
	VersionEOL,
	KeyStart,
	Key,
	ValueStart,
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

///////////////////////////////////////////////////////////////////////////////
//
// Public Functions
//
///////////////////////////////////////////////////////////////////////////////

/// Ensures a default ParseContext can be created and that it has the correct
/// starting values for a parse.
impl Default for ParseContext {
	fn default() -> ParseContext {
		ParseContext {
			state: ParseState::Method,
			temp: Vec::new(),
			url: String::new(),
			method: HttpMethod::GET,
			protocol: String::new(),
			headers: Vec::new(),
			key: String::new(),
		}
	}
}

impl ParseContext {
	/// Perform the HTTP parse. The first time, supply a default ParseContext
	/// object. Subsequently, supply the same object again.
	pub fn parse_header(&mut self, buffer: &[u8]) -> ParseResult {
		for b in buffer {
			let c = *b;
			let ct = get_char_type(c);
			println!("Got char type {:?} in state {:?}", ct, self.state);
			// switch on state, then switch on char type
			match self.state {
				ParseState::Method => {
					match ct {
						CharType::Other => self.temp.push(c),
						CharType::Space => {
							match String::from_utf8(self.temp.split_off(0)) {
								Ok(s) => {
									match s.as_str() {
										"GET" => self.method = HttpMethod::GET,
										"POST" => self.method = HttpMethod::POST,
										"PUT" => self.method = HttpMethod::PUT,
										"OPTION" => self.method = HttpMethod::OPTION,
										"HEAD" => self.method = HttpMethod::HEAD,
										_ => return ParseResult::Error
									}
									println!("Got method {:?}", self.method)
								},
								_ => return ParseResult::Error
							}
							self.state = ParseState::URL
						},
						_ => return ParseResult::Error
					}
				},
				ParseState::URL => {
					match ct {
						CharType::Other | CharType::Colon=> self.temp.push(c),
						CharType::Space => {
							match String::from_utf8(self.temp.split_off(0)) {
								Ok(s) => self.url = s,
								_ => return ParseResult::Error
							}
							println!("Got URL {:?}", self.url);
							self.state = ParseState::Version
						}
						_ => return ParseResult::Error
					}
				},
				ParseState::Version => {
					match ct {
						CharType::Other => self.temp.push(c),
						CharType::CR => {
							match String::from_utf8(self.temp.split_off(0)) {
								Ok(s) => self.protocol = s,
								_ => return ParseResult::Error
							}
							println!("Got protocol {:?}", self.protocol);
							self.state = ParseState::VersionEOL
						}
						_ => return ParseResult::Error
					}
				},
				ParseState::VersionEOL => {
					match ct {
						CharType::NL => self.state = ParseState::KeyStart,
						_ => return ParseResult::Error
					}
				},
				ParseState::KeyStart => {
					match ct {
						CharType::CR => self.state = ParseState::FinalEOL,
						CharType::Other => {
							self.temp.push(c);
							self.state = ParseState::Key
						},
						_ => return ParseResult::Error
					}
				},
				ParseState::Key => {
					match ct {
						CharType::Other => self.temp.push(c),
						CharType::Colon => {
							match String::from_utf8(self.temp.split_off(0)) {
								Ok(s) => self.key = s,
								_ => return ParseResult::Error
							}
							self.state = ParseState::ValueStart
						}
						_ => return ParseResult::Error
					}
				},
				ParseState::ValueStart => {
					match ct {
						CharType::Space => { },
						CharType::Other => {
							self.temp.push(c);
							self.state = ParseState::Value
						},
						_ => return ParseResult::Error
					}
				},
				ParseState::Value => {
					match ct {
						CharType::Other | CharType::Space | CharType::Colon => self.temp.push(c),
						CharType::CR => {
							match String::from_utf8(self.temp.split_off(0)) {
								Ok(s) => {
									let hdr:HttpHeader = HttpHeader::Unknown { key: self.key.clone(), value: s };
									println!("Got header {:?}", hdr);
									self.headers.push(hdr);
								},
								_ => return ParseResult::Error
							}
							self.state = ParseState::ValueEOL
						}
						_ => return ParseResult::Error
					}
				},
				ParseState::ValueEOL => {
					match ct {
						CharType::NL => self.state = ParseState::KeyStart,
						_ => return ParseResult::Error
					}
				},
				ParseState::FinalEOL => {
					match ct {
						CharType::NL => {
							let r: HttpRequest = HttpRequest {
								url: self.url.clone(),
								method: self.method.clone(),
								protocol: self.protocol.clone(),
								headers: self.headers.split_off(0),
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
}

///////////////////////////////////////////////////////////////////////////////
//
// Private Functions
//
///////////////////////////////////////////////////////////////////////////////

/// Allows us to create ParseState objects which default to the first state.
impl Default for ParseState {
	fn default() -> ParseState { ParseState::Method }
}

/// Map an octet (in US-ASCII) to a character
/// class, so we can decide what to do with it.
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

///////////////////////////////////////////////////////////////////////////////
//
// End of File
//
///////////////////////////////////////////////////////////////////////////////
