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

    pub fn handle_request<T>(&mut self, req: http::Request<T>) 
        -> Result<Response<Vec<u8>>, std::io::Error> {
            #[cfg(debug_assertions)]
            self.tera.full_reload().expect("Error parsing template");
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
            Resolved::Markdown(path) => markdown_response(&path, &self.tera),
            Resolved::Directory(path) => 
                Ok(dir_response(&path, accepts, &self.config, &self.tera)),
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
fn dir_response(path: &Path, accepts: Vec<AcceptFormat>, config: &Config, tera: &Tera) -> Response<Vec<u8>> {
    for af in accepts {
        match af {
            AcceptFormat::Json => return dir_json(path, config),
            AcceptFormat::Html => return dir_html(path, tera),
            AcceptFormat::Any => return dir_html(path, tera),
        }
    }
    // Apparently no preferences?
    return dir_html(path, tera);
}

fn dir_html(path: &Path, tera: &Tera) -> Response<Vec<u8>> {
    if let Ok(contents) = directory::read_contents(path) {
        eprintln!("Read contents {:#?}", serde_json::to_string(&contents).unwrap());
        let mut context = tera::Context::new();
        context.insert("dir_contents", &contents);
        if let Ok(rendered) = tera.render("directory.html", &context) {
            return response::from_string(rendered)
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

/// Convert a markdown document into an HTML response
fn markdown_response(path: &Path, tera: &Tera) -> Result<Response<Vec<u8>>, std::io::Error> {
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

    let dir_contents = directory::read_contents(
        path.parent().expect("There should always be a parent directory to read contents from"))
        .expect("Some error in reading the directory contents");
    // Apply the template
    use tera::Context;
    let mut context = Context::new();
    context.insert("content", &html_out);
    context.insert("dir_contents", &dir_contents);
    let html_out = tera.render(MARKDOWN_TEMPLATE, &context).expect("Template didn't work");
    Ok(response::from_string(html_out))
}

// }}}
