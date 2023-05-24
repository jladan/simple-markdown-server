//! Handlers for incoming requests

use http::StatusCode;
use pulldown_cmark::{Parser, Options, html};

use std::{
    io::{BufReader, Read}, 
    path::Path, 
    fs::File,
};

use crate::{
    response::{self, Response},
    config::Config,
    uri::{Resolved, Resolver},
};

mod directory;


pub fn handle_get<T>(req: http::Request<T>, resolver: &Resolver) -> Result<Response<Vec<u8>>, std::io::Error> {
    let resource = resolver.lookup(req.uri());
    eprintln!("Resource Found: {:?}", resource);
    match resource {
        Resolved::File(path) => file_response(&path),
        Resolved::Markdown(path) => markdown_response(&path, resolver.config()),
        Resolved::Directory(path) => 
            Ok(dir_response(&path, req.headers().get("accept"))),
        Resolved::None => Ok(not_found_response(req.uri().path())),
    }
}

pub fn handle_head<T>(req: http::Request<T>, resolver: &Resolver) -> Result<Response<Vec<u8>>, std::io::Error> {
    let mut resp = handle_get(req, resolver)?;
    *resp.body_mut() = Vec::new();
    return Ok(resp);
    
}

// Actual responses to a get request 

/// Respond to a missing file
fn not_found_response(path: &str) -> Response<Vec<u8>> {
    let content = format!("File not found: {}", path);
    let mut resp = response::from_string(content);
    *resp.status_mut() = StatusCode::NOT_FOUND;
    return resp;
}

/// Respond with the contents of a file
fn file_response(path: &Path) -> Result<Response<Vec<u8>>, std::io::Error> {
    let file = BufReader::new(File::open(path)?);
    let contents: Result<Vec<_>, _> = file.bytes().collect();
    Ok(response::from_bytes(contents?))
}

/// Response for a found directory
fn dir_response(path: &Path, t: Option<&http::HeaderValue>) -> Response<Vec<u8>> {
    if let Some(t) = t {
        if t.to_str().unwrap().contains(&"application/json") {
            return dir_json(path);
        }
    }
    return dir_html(path);
}

fn dir_html(path: &Path) -> Response<Vec<u8>> {
    response::from_string(format!("hTML Directory found: {}", path.to_str().unwrap()))
}

fn dir_json(path: &Path) -> Response<Vec<u8>> {
    response::from_string(format!("JSOn Directory found: {}", path.to_str().unwrap()))
}

/// Convert a markdown document into an HTML response
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

// 
