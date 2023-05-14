//! Submodule for parsing and managing HTTP requests
//!

use std::{string, io::BufRead};

const MAX_HEADERS: usize = 100;

pub fn from_bufread(buf_reader: &mut impl BufRead) 
        -> Result<http::Request<String>, ReqError> {
    let mut buf: String = String::new();
    // stream.read_to_end(&mut buf)?;
    let mut request = loop {
        buf_reader.read_line(&mut buf)?;
        let r = parse_headers(&buf.as_bytes());
        match r {
            Err(ReqError::Incomplete) => continue,
            _ => break r,
        };
    }?;
    // At this point, the buffer should be at start of body
    // TODO read the body if there is one
    Ok(request)
}

/// Parse a buffer into an `http::Request<String>`
///
/// # Errors
///   - If the request is incomplete, `ReqError::Incomplete`
///   - If the parser fails, `ReqError::Parse(httparse::Error)`
///   - If the body is not UTF8, `ReqError::Encoding(string::FromUTF8Error)`
///   - If the converting to an `http::Request` fails,  `ReqError::Convert(http::Error)`
pub fn parse_headers(buf: &[u8]) -> Result<http::Request<String>, ReqError>  {
    let mut headers = [httparse::EMPTY_HEADER; MAX_HEADERS];
    let mut preq = httparse::Request::new(&mut headers);
    
    let result = preq.parse(&buf)?;
    println!("parse result: {:?}", result);
    if let httparse::Status::Complete(body_start) = result {
        assert!(buf.len() == body_start, 
                "Header should end with \"\\r\\n\\r\\n\"");
        let request: http::request::Builder = http::Request::builder()
            .method(preq.method.unwrap())
            .uri(preq.path.unwrap());
        let request = preq.headers.iter()
            .fold(request, |r, h| r.header(h.name, h.value));
        // The `.to_vec()` performs the memory copy
        let body = String::from_utf8(buf[body_start..].to_vec())?;

        return request.body(body)
            .map_err(|e| ReqError::Convert(e))
    }
    Err(ReqError::Incomplete)
}

#[derive(Debug)]
pub enum ReqError {
    Incomplete,
    IO(std::io::Error),
    Parse(httparse::Error),
    Convert(http::Error),
    Encoding(string::FromUtf8Error),
}

impl From<std::io::Error> for ReqError {
    fn from(value: std::io::Error) -> Self {
        ReqError::IO(value)
    }
}

impl From<httparse::Error> for ReqError  {
    fn from(value: httparse::Error) -> Self {
        ReqError::Parse(value)
    }
}
impl From<http::Error> for ReqError  {
    fn from(value: http::Error) -> Self {
        ReqError::Convert(value)
    }
}
impl From<string::FromUtf8Error> for ReqError  {
    fn from(value: string::FromUtf8Error) -> Self {
        ReqError::Encoding(value)
    }
}
