use std::{
    net::TcpListener,
    io::{Write, BufReader}, 
};

use zettel_web::{
    handlers,
    request::{self, ReqError},
    response::{self, Response, IntoBytes}, 
    uri::Resolver,
    config::Config,
};


fn main() -> std::io::Result<()> {
    let config = Config::build()
        .source_env()
        .build();
    let resolver = Resolver::new(&config);
    let listener = TcpListener::bind(config.addr)?;

    for stream in listener.incoming() {
        let mut stream = stream.unwrap();
        // Read the stream as a request
        let mut buf_reader = BufReader::new(&stream);
        let req = request::from_bufread(&mut buf_reader);
        // If the request works, then serve it
        if let Ok(req) = req {
            let resp = handle_request(req, &resolver)?;
            let encoded = resp.into_bytes();
            stream.write_all(&encoded)?;
        } else if let Err(ReqError::IO(e)) = req {
            return Err(e);
        }

    }

    Ok(())
}

fn handle_request<T>(req: http::Request<T>, resolver: &Resolver) 
        -> Result<Response<Vec<u8>>, std::io::Error> {
    use http::Method;
    match req.method() {
        &Method::GET => handlers::handle_get(req, resolver),
        &Method::HEAD => handlers::handle_head(req, resolver),
        _ => Ok(response::unimplemented()),
    }
}
