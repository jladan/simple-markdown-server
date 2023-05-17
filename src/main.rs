use std::{
    net::TcpListener,
    io::{Write, BufReader, Read}, 
    path::{PathBuf, Path}, fs::File, ffi::OsStr, 
};

use zettel_web::{
    request::{self, ReqError},
    response::{self, IntoBytes}, 
    config::Config,
};

use pulldown_cmark::{Parser, Options, html};

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
            let resp = handle_request(req)?;
            let encoded = resp.into_bytes();
            stream.write_all(&encoded)?;
        } else if let Err(ReqError::IO(e)) = req {
            return Err(e);
        }

    }

    Ok(())
}

fn handle_request<T>(req: http::Request<T>) -> Result<http::Response<Vec<u8>>, std::io::Error> {
    use http::Method;
    match req.method() {
        &Method::GET => handle_get(req),
        _ => Ok(response::unimplemented()),
    }
}

fn handle_get<T>(req: http::Request<T>) -> Result<http::Response<Vec<u8>>, std::io::Error> {
    let path = PathBuf::from(req.uri().path());
    let path = PathBuf::from("./").join(path.strip_prefix("/").unwrap());
    eprintln!("{path:?}");
    if path.is_dir() {
        Ok(is_dir_response(&path))
    } else if path.is_file() {
        is_file_response(&path)
    } else {
        Ok(not_found_response(&path))
    }
}

fn not_found_response(path: &Path) -> http::Response<Vec<u8>> {
    let content = format!("File not found: {}", path.to_str().unwrap());
    http::Response::builder()
        .status(404)
        .header("content-length", content.len())
        .body(content.into_bytes())
        .unwrap()
}

fn is_file_response(path: &Path) -> Result<http::Response<Vec<u8>>, std::io::Error> {
    let mdext = OsStr::new("md");
    match path.extension() {
        Some(e) if e == mdext => markdown_response(path),
        _ => {
            let file = BufReader::new(File::open(path)?);
            let contents: Result<Vec<_>, _> = file.bytes().collect();
            Ok(bytes_response(contents?))
        }
    }
    // Ok(string_response(format!("File Found: {}", path.to_str().unwrap())))
}

fn is_dir_response(path: &Path) -> http::Response<Vec<u8>> {
    string_response(format!("Directory found: {}", path.to_str().unwrap()))
}

fn string_response(content: String) -> http::Response<Vec<u8>> {
    http::Response::builder()
        .status(200)
        .header("Content-Length", content.len())
        .body(content.into_bytes())
        .unwrap()
}

fn bytes_response(content: Vec<u8>) -> http::Response<Vec<u8>> {
    http::Response::builder()
        .status(200)
        .header("Content-Length", content.len())
        .body(content)
        .unwrap()
    
}

fn respond_hello_world() -> http::Response<Vec<u8>> {
    string_response(String::from("Hello world!"))
}

fn markdown_response(path: &Path) -> Result<http::Response<Vec<u8>>, std::io::Error> {
    let mut file = BufReader::new(File::open(path)?);
    let mut contents: String = String::new();
    file.read_to_string(&mut contents)?;
    let parser = Parser::new_ext(&contents, Options::all());

    let mut html_out = String::new();
    html::push_html(&mut html_out, parser);

    Ok(string_response(html_out))
}
