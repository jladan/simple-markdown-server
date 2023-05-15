use std::{
    net::TcpListener,
    io::{Write, BufReader}, 
    path::{PathBuf, Path}, 
};

use zettel_web::{
    request::{self, ReqError},
    response::{self, AsBytes}, 
    config::Config,
};

fn main() -> std::io::Result<()> {
    let config = Config::build()
        .source_env()
        .build();
    let listener = TcpListener::bind(config.addr)?;


    for stream in listener.incoming().take(3) {
        let mut stream = stream.unwrap();
        // Read the stream as a request
        let mut buf_reader = BufReader::new(&stream);
        let req = request::from_bufread(&mut buf_reader);
        println!("{:#?}", req);
        if let Ok(req) = req {
            let resp = handle_request(req);
            let encoded = resp.as_bytes().unwrap();
            stream.write_all(&encoded)?;
        } else if let Err(ReqError::IO(e)) = req {
            return Err(e);
        }

    }

    Ok(())
}

fn handle_request(req: http::Request<String>) -> http::Response<String> {
    use http::Method;
    match req.method() {
        &Method::GET => handle_get(req),
        _ => response::unimplemented(),
    }
}

fn handle_get(req: http::Request<String>) -> http::Response<String> {
    let path = PathBuf::from(req.uri().path());
    let path = PathBuf::from("./").join(path.strip_prefix("/").unwrap());
    eprintln!("{path:?}");
    if path.is_dir() {
        is_dir_response(&path)
    } else if path.is_file() {
        is_file_response(&path)
    } else {
        not_found_response(&path)
    }
}

fn not_found_response(path: &Path) -> http::Response<String> {
    let content = format!("File not found: {}", path.to_str().unwrap());
    http::Response::builder()
        .status(404)
        .header("content-length", content.len())
        .body(content)
        .unwrap()
}

fn is_file_response(path: &Path) -> http::Response<String> {
    string_response(format!("File Found: {}", path.to_str().unwrap()))
}

fn is_dir_response(path: &Path) -> http::Response<String> {
    string_response(format!("Directory found: {}", path.to_str().unwrap()))
}

fn string_response(content: String) -> http::Response<String> {
    http::Response::builder()
        .status(200)
        .header("Content-Length", content.len())
        .body(content)
        .unwrap()
}

fn respond_hello_world() -> http::Response<String> {
    string_response(String::from("Hello world!"))
}

