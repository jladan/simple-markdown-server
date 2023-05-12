use std::{
    net::{SocketAddr, TcpListener}, 
    io::{Write, BufReader, BufRead}, 
};

use http::header::ToStrError;
use zettel_web::request::{self, ReqError};

fn main() -> std::io::Result<()> {
    let addr = SocketAddr::from(([0,0,0,0], 7878));
    let listener = TcpListener::bind(addr)?;


    for stream in listener.incoming().take(3) {
        let mut stream = stream.unwrap();
        let mut buf_reader = BufReader::new(&stream);
        // let mut buf: Vec<u8> = Vec::new();
        let mut buf: String = String::new();
        // stream.read_to_end(&mut buf)?;
        let request = loop {
            buf_reader.read_line(&mut buf)?;
            let r = request::parse_request(&buf.as_bytes());
            match r {
                Err(ReqError::Incomplete) => continue,
                _ => break r,
            }
        };
        if let Ok(req) = request {
            process_request(req);
        }
        let encoded = response_to_bytes(respond_hello_world()).unwrap();
        println!("{}", encoded);
        stream.write_all(&encoded.as_bytes())?;
    }

    Ok(())
}

fn process_request(request: http::Request<String>) {
    if request.method() == http::Method::GET {
        let resp = respond_hello_world();
        println!("{}", response_to_bytes(resp).unwrap());
    }
}

fn response_to_bytes(resp: http::Response<String>) -> Result<String, ResError> {
    use http::StatusCode;
    let mut encoded = String::new();
    let (parts, body) = resp.into_parts();
    let status_code = match parts.status {
        StatusCode::OK => Ok("200 OK"),
        StatusCode::NOT_FOUND => Ok("404 NOT FOUND"),
        _ => Err(ResError::Unimplemented)
    }?;
    encoded.push_str(&format!("{:?} {status_code}\r\n", parts.version));
    for (k, v) in parts.headers.iter() {
        encoded.push_str(&format!("{}: {}\r\n", k, v.to_str()?));
    }
    encoded.push_str(&format!("\r\n{body}"));

    Ok(encoded)
}

fn respond_hello_world() -> http::Response<String> {
    let content = String::from("Hello world");
    http::Response::builder()
        .status(200)
        .header("Content-Length", content.len())
        .body(String::from("Hello World"))
        .unwrap()
}

#[derive(Debug)]
enum ResError {
    Unimplemented,
    Encoding(ToStrError),
}

impl From<ToStrError> for ResError {
    fn from(value: ToStrError) -> Self {
        ResError::Encoding(value)
    }
}

