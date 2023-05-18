use std::{
    net::TcpListener,
    io::{Write, BufReader, Read}, 
    path::Path, 
    fs::File,
};

use zettel_web::{
    request::{self, ReqError},
    response::{self, Response, IntoBytes}, 
    config::Config,
    uri::{Resolved, Resolver},
};

use pulldown_cmark::{Parser, Options, html};

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
        &Method::GET => handle_get(req, resolver),
        _ => Ok(response::unimplemented()),
    }
}

fn handle_get<T>(req: http::Request<T>, resolver: &Resolver) -> Result<Response<Vec<u8>>, std::io::Error> {
    let resource = resolver.lookup(req.uri());
    eprintln!("Resource Found: {:?}", resource);
    match resource {
        Resolved::File(path) => file_response(&path),
        Resolved::Markdown(path) => markdown_response(&path, resolver.config()),
        Resolved::Directory(path) => Ok(dir_response(&path)),
        Resolved::None => Ok(not_found_response(req.uri().path())),
    }
}

fn not_found_response(path: &str) -> Response<Vec<u8>> {
    let content = format!("File not found: {}", path);
    response::from_string(content)
}

fn file_response(path: &Path) -> Result<Response<Vec<u8>>, std::io::Error> {
    let file = BufReader::new(File::open(path)?);
    let contents: Result<Vec<_>, _> = file.bytes().collect();
    Ok(response::from_bytes(contents?))
}

fn dir_response(path: &Path) -> Response<Vec<u8>> {
    response::from_string(format!("Directory found: {}", path.to_str().unwrap()))
}

fn markdown_response(path: &Path, config: &Config) -> Result<Response<Vec<u8>>, std::io::Error> {
    let mut contents: String = String::new();
    {
        let mut file = BufReader::new(File::open(path)?);
        file.read_to_string(&mut contents)?;
    }
    // Start up parser
    let parser = Parser::new_ext(&contents, Options::all());
    let mut html_out = String::new();

    { // Get header
        let mut file = BufReader::new(File::open(&config.header)?);
        file.read_to_string(&mut html_out)?;
    }

    html::push_html(&mut html_out, parser);

    { // Add Footer
        let mut file = BufReader::new(File::open(&config.footer)?);
        file.read_to_string(&mut html_out)?;
    }

    Ok(response::from_string(html_out))
}
