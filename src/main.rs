use std::{
    net::{SocketAddr, TcpListener, TcpStream}, 
    io::{Read, Write}, 
    string,
};

const MAX_HEADERS: usize = 100;

fn main() -> std::io::Result<()> {
    let addr = SocketAddr::from(([0,0,0,0], 7878));
    let listener = TcpListener::bind(addr)?;


    for stream in listener.incoming().take(1) {
        let mut buf: Vec<u8> = Vec::new();
        let mut stream = stream.unwrap();
        stream.read_to_end(&mut buf)?;
        let request = parse_request(&buf);
        if let Ok(req) = request {
            println!("{:#?}", req);
            process_request(stream, req);
        }
    }

    Ok(())
}

fn process_request(stream: TcpStream, request: http::Request<String>) {
    if request.method() == http::Method::GET {
        let resp = respond_hello_world();
        println!("{}", String::from_utf8(response_to_bytes(resp).unwrap()).unwrap());
    }
}

fn response_to_bytes(resp: http::Response<String>) -> Result<Vec<u8>, ResError> {
    use http::StatusCode;
    let mut encoded = String::new();
    let (parts, body) = resp.into_parts();
    let status_code = match parts.status {
        StatusCode::OK => Ok("200 OK"),
        StatusCode::NOT_FOUND => Ok("404 NOT FOUND"),
        _ => Err(ResError::Unimplemented)
    }?;
    encoded.push_str(&format!("{:?} {status_code}/r/n", parts.version));

    Ok(encoded.as_bytes().to_vec())
}

fn respond_hello_world() -> http::Response<String> {
    let content = String::from("Hello world");
    http::Response::builder()
        .status(200)
        .header("Content-Length", content.len())
        .body(String::from("Hello World"))
        .unwrap()
}

/// Parse a buffer into an `http::Request<String>`
///
/// # Errors
///   - If the request is incomplete, `ReqError::Incomplete`
///   - If the parser fails, `ReqError::Parse(httparse::Error)`
///   - If the body is not UTF8, `ReqError::Encoding(string::FromUTF8Error)`
///   - If the converting to an `http::Request` fails,  `ReqError::Convert(http::Error)`
fn parse_request(buf: &[u8]) -> Result<http::Request<String>, ReqError>  {
    let mut headers = [httparse::EMPTY_HEADER; MAX_HEADERS];
    let mut preq = httparse::Request::new(&mut headers);
    
    let result = preq.parse(&buf)?;
    println!("parse result: {:?}", result);
    if let httparse::Status::Complete(body_start) = result {
        println!("{:#?}", preq);
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

#[derive(Debug, PartialEq, Eq)]
enum ResError {
    Unimplemented,
}
#[derive(Debug)]
enum ReqError {
    Incomplete,
    Parse(httparse::Error),
    Convert(http::Error),
    Encoding(string::FromUtf8Error),
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
