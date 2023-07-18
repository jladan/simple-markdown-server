use std::{
    net::{TcpListener, TcpStream},
    io::{Write, BufReader}, 
    sync::Arc,
};

use zettel_web::{
    handlers::Handler,
    request::{self, ReqError},
    response::IntoBytes, 
    config::Config,
};

use threadpool::ThreadPool;

const N_WORKERS: usize = 4;

fn main() -> std::io::Result<()> {
    let config = Config::build()
        .source_env()
        .build();
    println!("{config:#?}");
    let listener = TcpListener::bind(config.addr)?;


    let pool = ThreadPool::new(N_WORKERS);
    // Has to be an Arc to ensure lifetimes
    let handler: Arc<Handler> = Arc::new(Handler::new(config));
    for stream in listener.incoming() {
        let stream = stream.unwrap();
        let handler = handler.clone();

        pool.execute(move || {
            if let Err(e) =  handle_connection(stream, handler) {
                eprintln!("{e}");
            }
        });
    }

    Ok(())
}

/// Parses the stream as a request, then hands it off to the request handler
fn handle_connection(mut stream: TcpStream, handler: Arc<Handler>) -> std::io::Result<()>{
        let mut buf_reader = BufReader::new(&stream);
        let req = request::from_bufread(&mut buf_reader);
        // If the request works, then serve it
        if let Ok(req) = req {
            // eprintln!("{req:#?}");
            let resp = handler.handle_request(req)?;
            let encoded = resp.into_bytes();
            stream.write_all(&encoded)?;
        } else if let Err(ReqError::IO(e)) = req {
            return Err(e);
        }
        Ok(())
}
