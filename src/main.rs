use std::{
    net::{SocketAddr, TcpListener}, 
    io::{Write, BufReader}, 
};

use zettel_web::{
    request::{self, ReqError},
    response,
};

fn main() -> std::io::Result<()> {
    let addr = SocketAddr::from(([0,0,0,0], 7878));
    let listener = TcpListener::bind(addr)?;


    for stream in listener.incoming().take(3) {
        let mut stream = stream.unwrap();
        // Read the stream as a request
        let mut buf_reader = BufReader::new(&stream);
        let req = request::from_bufread(&mut buf_reader);
        println!("{:#?}", req);
        if let Ok(req) = req {
            let resp = handle_request(req);
            let encoded = response::to_string(resp).unwrap();
            println!("{}", encoded);
            stream.write_all(&encoded.as_bytes())?;
        } else if let Err(ReqError::IO(e)) = req {
            return Err(e);
        }

    }

    Ok(())
}

fn handle_request(req: http::Request<String>) -> http::Response<String> {
    use http::Method;
    match req.method() {
        &Method::GET => respond_hello_world(),
        _ => response::unimplemented(),
    }
}

fn respond_hello_world() -> http::Response<String> {
    let content = String::from("Hello world");
    http::Response::builder()
        .status(200)
        .header("Content-Length", content.len())
        .body(String::from("Hello World"))
        .unwrap()
}

