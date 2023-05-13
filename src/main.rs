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
        // Read the stream as a request
        let mut buf_reader = BufReader::new(&stream);
        let req = request::from_bufread(&mut buf_reader);
        println!("{:#?}", req);
        if let Ok(_req) = req {
            let encoded = response_to_bytes(respond_hello_world()).unwrap();
            println!("{}", encoded);
            stream.write_all(&encoded.as_bytes())?;
        } else if let Err(ReqError::IO(e)) = req {
                return Err(e);
        }

    }

    Ok(())
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

