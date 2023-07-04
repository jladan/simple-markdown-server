//! Handlers for incoming requests

use http::StatusCode;
use pulldown_cmark::{Parser, Options, html};

use std::{
    io::{BufReader, Read}, 
    path::Path, 
    fs::File, sync::RwLock,
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
pub mod walkdir;

pub struct Handler {
    config: Config,
    resolver: Resolver,
    tera: RwLock<Tera>,
}

impl Handler {
    pub fn new(config: Config) -> Handler {
        let resolver = Resolver::new(&config);
        let template_glob = config.template_dir.join("**/*.html");
        let tera = match Tera::new(&template_glob.to_str().unwrap()) {
            Ok(t) => RwLock::new(t),
            Err(e) => {eprintln!("{e}"); panic!()},
        };
        Handler {config, resolver, tera}
    }

    pub fn handle_request<T>(&self, req: http::Request<T>) 
        -> Result<Response<Vec<u8>>, std::io::Error> {
            #[cfg(debug_assertions)]
            {
                let mut lock = self.tera.write().unwrap();
                match lock.full_reload() {
                    Ok(_) => (),
                    Err(e) => {eprintln!("{e}"); drop(lock); panic!()},
                }
            }
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
        let tera = self.tera.read().unwrap();
        match resource {
            Resolved::File(path) => file_response(&path),
            Resolved::Markdown(path) => markdown_response(&path, &self.config, &tera),
            Resolved::Directory(path) => 
                Ok(dir_response(&path, accepts, &self.config, &tera)),
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
    if let Ok(dirtree) = walkdir::walk_dir(path, false) {
        use AcceptFormat::*;
        for af in accepts {
            match af {
                Json => return dir_json(dirtree, config),
                Html | Any => return dir_html(dirtree, tera),
            }
        }
        // Apparently no preferences?
        return dir_html(dirtree, tera);
    } else {
        return response::server_error();
    }
}

fn dir_html(dirtree: walkdir::Directory, tera: &Tera) -> Response<Vec<u8>> {
    eprintln!("Read contents {:#?}", serde_json::to_string(&dirtree).unwrap());
    let mut context = tera::Context::new();
    context.insert("dir_contents", &dirtree);
    if let Ok(rendered) = tera.render("directory.html", &context) {
        return response::from_string(rendered)
    }
    return response::server_error()
}

fn dir_json(dirtree: walkdir::Directory,  _config: &Config) -> Response<Vec<u8>> {
    if let Ok(s) = serde_json::to_string(&dirtree) {
        response::from_string(s)
    } else {
        response::server_error()
    }
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
    // NOTE(jladan): disable smart punctuation for latex
    let options = Options::from_bits_truncate(0b1011110);
    let parser = Parser::new_ext(&contents, options);
    let mut html_out = String::new();
    html::push_html(&mut html_out, parser);

    let root_contents = walkdir::walk_dir(&config.rootdir , true)
        .expect("Problem stripping prefix?");
    // Apply the template
    use tera::Context;
    let mut context = Context::new();
    context.insert("content", &html_out);
    context.insert("dirtree", &root_contents);
    match tera.render(MARKDOWN_TEMPLATE, &context) {
        Ok(html_out) => Ok(response::from_string(html_out)),
        Err(e) => {
            eprintln!("{e}");
            Ok(response::server_error())
        }
    }
}

// }}}
