//! Module for creating responses 
//!

use http::{
    HeaderName, 
    HeaderValue, 
    response::Parts};

// Export http::Response, because it's used so much
pub use http::Response;

/// Trait to represent Responses that can be encoded into Bytes
///
/// This allows for encoding `resp: Response<T>` for transmission simply with
/// `resp.into_bytes()` for `T` in {`String`, `()`, `Vec<u8>`}
pub trait IntoBytes {
    fn into_bytes(self) -> Vec<u8>;
    
}

// IntoBytes implementations {{{

impl IntoBytes for Response<String> {
    fn into_bytes(self) -> Vec<u8> {
        let (parts, body) = self.into_parts();
        let h = encode_header(parts);
        return [h, b"\r\n".to_vec(), body.into_bytes()].concat();
    }
}

impl IntoBytes for Response<Vec<u8>> {
    fn into_bytes(self) -> Vec<u8> {
        let (parts, body) = self.into_parts();
        let h = encode_header(parts);
        return [h, b"\r\n".to_vec(), body].concat();
    }
}

impl IntoBytes for Response<()> {
    fn into_bytes(self) -> Vec<u8> {
        let (parts, _) = self.into_parts();
        encode_header(parts)
    }
}

// }}}

// General responses {{{

/// Create an "unimplemented" response for unimplemented requests
pub fn unimplemented() -> Response<Vec<u8>> {
    Response::builder()
        .status(501)
        .body(Vec::new())
        .unwrap()
}

/// A "not allowed" response for recognized, but not allowed for the resource
pub fn not_allowed() -> Response<Vec<u8>> {
    Response::builder()
        .status(405)
        .body(Vec::new())
        .unwrap()
}

/// If any error occurs on the server side
pub fn server_error() -> Response<Vec<u8>> {
    Response::builder()
        .status(500)
        .body(Vec::new())
        .unwrap()
}

/// Create a 200 response from a String
///
/// Can also be used for other responses, by mutating the status code afterwards, like
/// ```
/// use zettel_web::response;
/// let mut resp = response::from_string(String::from("Not found"));
/// *resp.status_mut() = http::StatusCode::NOT_FOUND;
/// assert_eq!(resp.status(), http::StatusCode::NOT_FOUND);
/// ```
pub fn from_string(content: String) -> Response<Vec<u8>> {
    Response::builder()
        .status(200)
        .header("Content-Length", content.len())
        .body(content.into_bytes())
        .unwrap()
}

/// Create a 200 response from a Vec<u8>
pub fn from_bytes(content: Vec<u8>) -> Response<Vec<u8>> {
    Response::builder()
        .status(200)
        .header("Content-Length", content.len())
        .body(content)
        .unwrap()
    
}


// }}}

// Actual Encoding of responses {{{

/// Encode a response header
fn encode_header(parts: Parts) -> Vec<u8> {
    let mut lines: Vec<Vec<u8>> = Vec::new();
    lines.push(statusline(&parts));
    for (k, v) in parts.headers.iter() {
        lines.push(headerline(k, v))
    }
    return lines.concat();
}

/// Create the status line for a response
fn statusline(parts: &Parts) -> Vec<u8> {
    let status_code = parts.status;
    format!("{:?} {} {}\r\n", 
            parts.version, 
            status_code.as_str(), 
            status_code.canonical_reason().expect("No canonical reason phrase"))
        .into_bytes()
}

/// Create a headerline from parts
fn headerline(k: &HeaderName, v: &HeaderValue) -> Vec<u8> {
    [ k.as_str().as_bytes(), b": ", v.as_bytes(), b"\r\n", ].concat()
}

// }}}
