use std::{
    net::{SocketAddr, TcpListener}, 
    io::Read, 
    string,
};
use httparse;

const MAX_HEADERS: usize = 100;

fn main() -> std::io::Result<()> {
    let addr = SocketAddr::from(([0,0,0,0], 7878));
    let listener = TcpListener::bind(addr)?;


    let mut request: Result<http::Request<String>, ReqError> = Err(ReqError::Incomplete);
    for stream in listener.incoming().take(1) {
        let mut buf: Vec<u8> = Vec::new();
        stream.unwrap().read_to_end(&mut buf)?;
        request = parse_request(&buf);
    }
    if let Ok(req) = request {
        println!("{:#?}", req);
    }

    Ok(())
}

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
