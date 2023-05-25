//! Handlers for incoming requests

use http::StatusCode;
use pulldown_cmark::{Parser, Options, html};

use std::{
    io::{self, BufReader, Read}, 
    path::Path, 
    fs::File,
};

use crate::{
    response::{self, Response},
    config::Config,
    uri::{Resolved, Resolver},
};

mod directory;

enum AcceptFormat {
    Html,
    Json,
    Any,
}

fn preferred_format(headers: &http::HeaderMap) -> Vec<AcceptFormat> {
    if let Some(value) = headers.get("accept") {
        value.to_str().expect("accept header could not be converted to string?")
            .split(',').filter_map(|e| {
                if e.contains("json") {
                    Some(AcceptFormat::Json)
                } else if e.contains("html") {
                    Some(AcceptFormat::Html)
                } else if e.contains("*/*") {
                    Some(AcceptFormat::Any)
                } else {
                    None
                }
            }).collect()
    } else {
        vec![AcceptFormat::Any]
    }
}

pub fn handle_get<T>(req: http::Request<T>, resolver: &Resolver) -> Result<Response<Vec<u8>>, std::io::Error> {
    let resource = resolver.lookup(req.uri());
    let accepts = preferred_format(&req.headers());
    eprintln!("Resource Found: {:?}", resource);
    match resource {
        Resolved::File(path) => file_response(&path),
        Resolved::Markdown(path) => markdown_response(&path, resolver.config()),
        Resolved::Directory(path) => 
            Ok(dir_response(&path, accepts)),
        Resolved::None => Ok(not_found_response(req.uri().path())),
    }
}

pub fn handle_head<T>(req: http::Request<T>, resolver: &Resolver) -> Result<Response<Vec<u8>>, std::io::Error> {
    let mut resp = handle_get(req, resolver)?;
    *resp.body_mut() = Vec::new();
    return Ok(resp);
    
}

// Actual responses to a get request {{{

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
fn dir_response(path: &Path, accepts: Vec<AcceptFormat>) -> Response<Vec<u8>> {
    for af in accepts {
        match af {
            AcceptFormat::Json => return dir_json(path),
            AcceptFormat::Html => return dir_html(path),
            AcceptFormat::Any => return dir_html(path),
        }
    }
    // Apparently no preferences?
    return dir_html(path);
}

fn dir_html(path: &Path) -> Response<Vec<u8>> {
    if let Ok(s) = directory::get_html(path) {
        response::from_string(s)
    } else {
        response::server_error()
    }
}

fn dir_json(path: &Path) -> Response<Vec<u8>> {
    if let Ok(s) = directory::get_json(path) {
        response::from_string(s)
    } else {
        response::server_error()
    }
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

// }}}
