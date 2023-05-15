use http::header::ToStrError;

/// Create an "unimplemented" response for unimplemented requests
pub fn unimplemented() -> http::Response<()> {
    http::Response::builder()
        .status(501)
        .body(())
        .unwrap()
}

/// A "not allowed" response for recognized, but not allowed for the resource
pub fn not_allowed() -> http::Response<()> {
    http::Response::builder()
        .status(405)
        .body(())
        .unwrap()
}

/// Convert a full response to a string for sending to the client
pub fn to_bytes(resp: http::Response<String>) -> Result<String, ResError> {
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
    use http::StatusCode;
    let status_code = match parts.status {
        StatusCode::OK => Ok("200 OK"),
        StatusCode::METHOD_NOT_ALLOWED => Ok("405 METHOD NOT ALLOWED"),
        StatusCode::NOT_FOUND => Ok("404 NOT FOUND"),
        StatusCode::NOT_IMPLEMENTED => Ok("501 NOT IMPLEMENTED"),
        _ => Err(ResError::Unimplemented)
    }?;
   Ok(format!("{:?} {status_code}\r\n", parts.version))
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

