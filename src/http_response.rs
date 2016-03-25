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

//use http::*;

// ****************************************************************************
//
// Public Types
//
// ****************************************************************************

/// An HTTP Response.
/// Fully describes the HTTP response sent from the server to the client.
#[derive(Debug)]
pub struct HttpResponse {
    /// The HTTP result code - @todo should be an enum
    pub result: u32,
    /// The protocol the client is using in the response
    pub protocol: String,
    /// Any headers supplied by the server in the response
    pub headers: HashMap<String, String>,
    /// The response body
    pub body: String
}

/// Contains the internal state for the renderer.
#[derive(Debug)]
pub struct HttpResponseRenderer {
    pub foo: String
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

// None

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
