//! # HTTP Response generation
//!
//! The HTTP Response code converts response objects into octets and
//! writes them to a stream.

// ****************************************************************************
//
// Imports
//
// ****************************************************************************

use std::collections::HashMap;
use std::fmt;
use std::io;
use std::borrow::Cow;

// ****************************************************************************
//
// Public Types
//
// ****************************************************************************

#[derive(Debug, Clone, Copy)]
pub enum HttpResponseStatus {
    Continue = 100,
    SwitchingProtocols = 101,
    Processing = 102,
    OK = 200,
    Created = 201,
    Accepted = 202,
    NonAuthoritativeInformation = 203,
    NoContent = 204,
    ResetContent = 205,
    PartialContent = 206,
    MultiStatus = 207,
    AlreadyReported = 208,
    ImUsed = 226,
    MultipleChoices = 300,
    MovedPermanently = 301,
    Found = 302,
    SeeOther = 303,
    NotModified = 304,
    UseProxy = 305,
    SwitchProxy = 306,
    TemporaryRedirect = 307,
    PermanentRedirect = 308,
    BadRequest = 400,
    Unauthorized = 401,
    PaymentRequired = 402,
    Forbidden = 403,
    NotFound = 404,
    MethodNotAllowed = 405,
    NotAcceptable = 406,
    ProxyAuthenticationRequired = 407,
    RequestTimeout = 408,
    Conflict = 409,
    Gone = 410,
    LengthRequired = 411,
    PreconditionFailed = 412,
    PayloadTooLarge = 413,
    URITooLong = 414,
    UnsupportedMediaType = 415,
    RangeNotSatisfiable = 416,
    ExpectationFailed = 417,
    IAmATeapot = 418,
    MisdirectedRequest = 421,
    UnprocessableEntity = 422,
    Locked = 423,
    FailedDependency = 424,
    UpgradeRequired = 426,
    PreconditionRequired = 428,
    TooManyRequests = 429,
    RequestHeaderFieldsTooLarge = 431,
    UnavailableForLegalReasons = 451,
    InternalServerError = 500,
    NotImplemented = 501,
    BadGateway = 502,
    ServiceUnavailable = 503,
    GatewayTimeout = 504,
    HTTPVersionNotSupported = 505,
    VariantAlsoNegotiates = 506,
    InsufficientStorage = 507,
    LoopDetected = 508,
    NotExtended = 510,
    NetworkAuthenticationRequired = 511,
}

/// An HTTP Response.
/// Fully describes the HTTP response sent from the server to the client.
/// Because the user can create these objects, we use a Cow to allow them
/// to supply either an `&str` or a `std::string::String`.
#[derive(Debug)]
pub struct HttpResponse<'a> {
    /// The HTTP result code - @todo should be an enum
    pub status: HttpResponseStatus,
    /// The protocol the client is using in the response
    pub protocol: Cow<'a, str>,
    /// Any headers supplied by the server in the response
    pub headers: HashMap<Cow<'a, str>, Cow<'a, str>>,
    /// The response body
    pub body: Cow<'a, str>,
}

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

impl<'a> HttpResponse<'a> {
    pub fn new<S>(status: HttpResponseStatus, protocol: S) -> HttpResponse<'a>
        where S: Into<Cow<'a, str>>
    {
        HttpResponse::new_with_body(status, protocol, Cow::Borrowed(""))
    }

    pub fn new_with_body<S, T>(status: HttpResponseStatus, protocol: S, body: T) -> HttpResponse<'a>
        where S: Into<Cow<'a, str>>,
              T: Into<Cow<'a, str>>
    {
        HttpResponse {
            status: status,
            protocol: protocol.into(),
            headers: HashMap::new(),
            body: body.into(),
        }
    }

    pub fn write<T: io::Write>(&self, sink: &mut T) -> io::Result<usize> {
        let header: String = format!("{} {}\r\n", self.protocol, self.status);
        let mut total: usize = 0;
        total += try!(sink.write(header.as_bytes()));
        for (k, v) in &self.headers {
            let line = format!("{}: {}\r\n", k, v);
            total += try!(sink.write(line.as_bytes()));
        }
        total += try!(sink.write(b"\r\n"));
        total += try!(sink.write(self.body.as_bytes()));
        return Ok(total);
    }

    pub fn add_header<S, T>(&mut self, key: S, value: T)
        where S: Into<Cow<'a, str>>,
              T: Into<Cow<'a, str>>
    {
        self.headers.insert(key.into(), value.into());
    }
}

impl fmt::Display for HttpResponseStatus {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Write strictly the first element into the supplied output
        // stream: `f`. Returns `fmt::Result` which indicates whether the
        // operation succeeded or failed. Note that `write!` uses syntax which
        // is very similar to `println!`.
        write!(f, "{} {}", *self as u32, self.as_string())
    }
}

impl HttpResponseStatus {
    pub fn as_string(&self) -> &str {
        match *self {
            HttpResponseStatus::Continue => "Continue",
            HttpResponseStatus::SwitchingProtocols => "Switching Protocols",
            HttpResponseStatus::Processing => "Processing",
            HttpResponseStatus::OK => "OK",
            HttpResponseStatus::Created => "Created",
            HttpResponseStatus::Accepted => "Accepted",
            HttpResponseStatus::NonAuthoritativeInformation => "Non-Authoritative Information",
            HttpResponseStatus::NoContent => "No Content",
            HttpResponseStatus::ResetContent => "Reset Content",
            HttpResponseStatus::PartialContent => "Partial Content",
            HttpResponseStatus::MultiStatus => "Multi Status",
            HttpResponseStatus::AlreadyReported => "Already Reported",
            HttpResponseStatus::ImUsed => "IM Used",
            HttpResponseStatus::MultipleChoices => "Multiple Choices",
            HttpResponseStatus::MovedPermanently => "Moved Permanently",
            HttpResponseStatus::Found => "Found",
            HttpResponseStatus::SeeOther => "See Other",
            HttpResponseStatus::NotModified => "Not Modified",
            HttpResponseStatus::UseProxy => "Use Proxy",
            HttpResponseStatus::SwitchProxy => "Switch Proxy",
            HttpResponseStatus::TemporaryRedirect => "Temporary Redirect",
            HttpResponseStatus::PermanentRedirect => "Permanent Redirect",
            HttpResponseStatus::BadRequest => "Bad Request",
            HttpResponseStatus::Unauthorized => "Unauthorized",
            HttpResponseStatus::PaymentRequired => "Payment Required",
            HttpResponseStatus::Forbidden => "Forbidden",
            HttpResponseStatus::NotFound => "Not Found",
            HttpResponseStatus::MethodNotAllowed => "Method Not Allowed",
            HttpResponseStatus::NotAcceptable => "Not Acceptable",
            HttpResponseStatus::ProxyAuthenticationRequired => "Proxy Authentication Required",
            HttpResponseStatus::RequestTimeout => "Request Timeout",
            HttpResponseStatus::Conflict => "Conflict",
            HttpResponseStatus::Gone => "Gone",
            HttpResponseStatus::LengthRequired => "Length Required",
            HttpResponseStatus::PreconditionFailed => "Precondition Failed",
            HttpResponseStatus::PayloadTooLarge => "Payload Too Large",
            HttpResponseStatus::URITooLong => "URI Too Long",
            HttpResponseStatus::UnsupportedMediaType => "Unsupported Media Type",
            HttpResponseStatus::RangeNotSatisfiable => "Range Not Satisfiable",
            HttpResponseStatus::ExpectationFailed => "Expectation Failed",
            HttpResponseStatus::IAmATeapot => "I'm A Teapot",
            HttpResponseStatus::MisdirectedRequest => "Misdirected Request",
            HttpResponseStatus::UnprocessableEntity => "Unprocessable Entity",
            HttpResponseStatus::Locked => "Locked",
            HttpResponseStatus::FailedDependency => "Failed Dependency",
            HttpResponseStatus::UpgradeRequired => "Upgrade Required",
            HttpResponseStatus::PreconditionRequired => "Precondition Required",
            HttpResponseStatus::TooManyRequests => "Too Many Requests",
            HttpResponseStatus::RequestHeaderFieldsTooLarge => "Request Header Fields Too Large",
            HttpResponseStatus::UnavailableForLegalReasons => "Unavailable For Legal Reasons",
            HttpResponseStatus::InternalServerError => "Internal Server Error",
            HttpResponseStatus::NotImplemented => "Not Implemented",
            HttpResponseStatus::BadGateway => "Bad Gateway",
            HttpResponseStatus::ServiceUnavailable => "Service Unavailable",
            HttpResponseStatus::GatewayTimeout => "Gateway Timeout",
            HttpResponseStatus::HTTPVersionNotSupported => "HTTP Version Not Supported",
            HttpResponseStatus::VariantAlsoNegotiates => "Variant Also Negotiates",
            HttpResponseStatus::InsufficientStorage => "Insufficient Storage",
            HttpResponseStatus::LoopDetected => "Loop Detected",
            HttpResponseStatus::NotExtended => "Not Extended",
            HttpResponseStatus::NetworkAuthenticationRequired => "Network Authentication Required",
        }
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
