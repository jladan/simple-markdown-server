use std::{
    net::TcpListener,
    io::{Write, BufReader}, 
};

use zettel_web::{
    handlers::Handler,
    request::{self, ReqError},
    response::IntoBytes, 
    config::Config,
};


fn main() -> std::io::Result<()> {
    let config = Config::build()
        .source_env()
        .build();
    let listener = TcpListener::bind(config.addr)?;
    let handler: Handler = Handler::new(config);

    for stream in listener.incoming() {
        let mut stream = stream.unwrap();
        // Read the stream as a request
        let mut buf_reader = BufReader::new(&stream);
        let req = request::from_bufread(&mut buf_reader);
        // If the request works, then serve it
        if let Ok(req) = req {
            eprintln!("{req:#?}");
            let resp = handler.handle_request(req)?;
            let encoded = resp.into_bytes();
            stream.write_all(&encoded)?;
        } else if let Err(ReqError::IO(e)) = req {
            return Err(e);
        }

    }

    Ok(())
}

