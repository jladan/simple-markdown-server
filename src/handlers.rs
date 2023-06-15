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

// templating using Tera for the markdown handler
extern crate tera;
use tera::Tera;

const MARKDOWN_TEMPLATE: &str = "markdown.html";

mod directory;

pub struct Handler {
    config: Config,
    resolver: Resolver,
    tera: Tera,
}

impl Handler {
    pub fn new(config: Config) -> Handler {
        let resolver = Resolver::new(&config);
        let template_glob = config.template_dir.join("**/*.html");
        let tera = Tera::new(&template_glob.to_str().unwrap()).expect("Error parsing template");
        Handler {config, resolver, tera}
    }

    pub fn handle_request<T>(&self, req: http::Request<T>) 
        -> Result<Response<Vec<u8>>, std::io::Error> {
            use http::Method;
            match req.method() {
                &Method::GET => self.handle_get(req),
                &Method::HEAD => self.handle_head(req),
                _ => Ok(response::unimplemented()),
            }
        }
    pub fn handle_get<T>(&self, req: http::Request<T>) -> Result<Response<Vec<u8>>, std::io::Error> {
        let resource = self.resolver.lookup(req.uri());
        let accepts = preferred_format(&req.headers());
        eprintln!("Resource Found: {:?}", resource);
        match resource {
            Resolved::File(path) => file_response(&path),
            Resolved::Markdown(path) => markdown_response(&path, &self.config, &self.tera),
            Resolved::Directory(path) => 
                Ok(dir_response(&path, accepts, &self.config)),
            Resolved::None => Ok(not_found_response(req.uri().path())),
        }
    }

    pub fn handle_head<T>(&self, req: http::Request<T>) -> Result<Response<Vec<u8>>, std::io::Error> {
        let mut resp = self.handle_get(req)?;
        *resp.body_mut() = Vec::new();
        return Ok(resp);
    }

}

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
fn dir_response(path: &Path, accepts: Vec<AcceptFormat>, config: &Config) -> Response<Vec<u8>> {
    for af in accepts {
        match af {
            AcceptFormat::Json => return dir_json(path, config),
            AcceptFormat::Html => return dir_html(path, config),
            AcceptFormat::Any => return dir_html(path, config),
        }
    }
    // Apparently no preferences?
    return dir_html(path, config);
}

fn dir_html(path: &Path, config: &Config) -> Response<Vec<u8>> {
    if let Ok(s) = directory::get_html(path) {
        if let Ok(wrapped) = wrap_html(s, config) {
            return response::from_string(wrapped);
        }
    }
    return response::server_error()
}

fn dir_json(path: &Path, _config: &Config) -> Response<Vec<u8>> {
    if let Ok(s) = directory::get_json(path) {
        response::from_string(s)
    } else {
        response::server_error()
    }
}

fn wrap_html(contents: String, config: &Config) -> io::Result<String> {
    let mut html_out = String::new();
    { // Get header
        let mut file = BufReader::new(File::open(&config.header)?);
        file.read_to_string(&mut html_out)?;
    }

    html_out.push_str(&contents);

    { // Add Footer
        let mut file = BufReader::new(File::open(&config.footer)?);
        file.read_to_string(&mut html_out)?;
    }

    Ok(html_out)
}

/// Convert a markdown document into an HTML response
fn markdown_response(path: &Path, config: &Config, tera: &Tera) -> Result<Response<Vec<u8>>, std::io::Error> {
    // Load the markdown
    let mut contents: String = String::new();
    {
        let mut file = BufReader::new(File::open(path)?);
        file.read_to_string(&mut contents)?;
    }
    // Parse the markdown
    let parser = Parser::new_ext(&contents, Options::all());
    let mut html_out = String::new();
    html::push_html(&mut html_out, parser);

    // Apply the template
    use tera::Context;
    let mut context = Context::new();
    context.insert("content", &html_out);
    let html_out = tera.render(MARKDOWN_TEMPLATE, &context).expect("Template didn't work");
    Ok(response::from_string(html_out))
}

// }}}
