use http::header::ToStrError;

/// Create an "unimplemented" response for unimplemented requests
pub fn unimplemented() -> http::Response<String> {
    http::Response::builder()
        .status(501)
        .body(String::new())
        .unwrap()
}

/// A "not allowed" response for recognized, but not allowed for the resource
pub fn not_allowed() -> http::Response<String> {
    http::Response::builder()
        .status(405)
        .body(String::new())
        .unwrap()
}

/// Convert a full response to a string for sending to the client
pub fn to_string(resp: http::Response<String>) -> Result<String, ResError> {
    let mut encoded = String::new();
    let (parts, body) = resp.into_parts();
    encoded.push_str(&statusline(&parts)?);
    for (k, v) in parts.headers.iter() {
        encoded.push_str(&format!("{}: {}\r\n", k, v.to_str()?));
    }
    encoded.push_str(&format!("\r\n{body}"));

    Ok(encoded)
}

/// Create the status line for a response
fn statusline(parts: &http::response::Parts) -> Result<String, ResError> {
    let status_code = parts.status;
   Ok(format!("{:?} {} {}\r\n", 
              parts.version, 
              status_code.as_str(), 
              status_code.canonical_reason().expect("No canonical reason phrase")))
}

#[derive(Debug)]
pub enum ResError {
    Unimplemented,
    Encoding(ToStrError),
}

impl From<ToStrError> for ResError {
    fn from(value: ToStrError) -> Self {
        ResError::Encoding(value)
    }
}

