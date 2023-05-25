
pub trait IntoBytes {
    fn into_bytes(self) -> Vec<u8>;
    
}

impl IntoBytes for http::Response<String> {
    fn into_bytes(self) -> Vec<u8> {
        let (parts, body) = self.into_parts();
        let h = encode_header(parts);
        return [h, b"\r\n".to_vec(), body.into_bytes()].concat();
    }
}

impl IntoBytes for http::Response<Vec<u8>> {
    fn into_bytes(self) -> Vec<u8> {
        let (parts, body) = self.into_parts();
        let h = encode_header(parts);
        return [h, b"\r\n".to_vec(), body].concat();
    }
}

impl IntoBytes for http::Response<()> {
    fn into_bytes(self) -> Vec<u8> {
        let (parts, _) = self.into_parts();
        encode_header(parts)
    }
}



/// Create an "unimplemented" response for unimplemented requests
pub fn unimplemented() -> http::Response<Vec<u8>> {
    http::Response::builder()
        .status(501)
        .body(Vec::new())
        .unwrap()
}

/// A "not allowed" response for recognized, but not allowed for the resource
pub fn not_allowed() -> http::Response<Vec<u8>> {
    http::Response::builder()
        .status(405)
        .body(Vec::new())
        .unwrap()
}


/// Encode a response header
pub fn encode_header(parts: http::response::Parts) -> Vec<u8> {
    let mut lines: Vec<Vec<u8>> = Vec::new();
    lines.push(statusline(&parts));
    for (k, v) in parts.headers.iter() {
        lines.push(headerline(k, v))
    }
    return lines.concat();
}

/// Create the status line for a response
fn statusline(parts: &http::response::Parts) -> Vec<u8> {
    let status_code = parts.status;
    format!("{:?} {} {}\r\n", 
            parts.version, 
            status_code.as_str(), 
            status_code.canonical_reason().expect("No canonical reason phrase"))
        .into_bytes()
}

/// Create a headerline from parts
fn headerline(k: &http::HeaderName, v: &http::HeaderValue) -> Vec<u8> {
    [ k.as_str().as_bytes(), b": ", v.as_bytes(), b"\r\n", ].concat()
}
