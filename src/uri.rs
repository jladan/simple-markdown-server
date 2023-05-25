//! URI lookup library
//!
//! This works like a virtual filesystem for the webserver, so that files can be
//! returned that don't perfectly match those in webroot.
//!
//! - Priority is to match existing files under $WEB_ROOT/
//! - If the file does not exist, check to see if one exists with ".md"
//! - Finally, look in $STATIC_DIR/ for the file
//!
//! # Possible improvements
//!
//! - cache filenames (particularly for css)
//! - A resolver struct for passing around the config

use std::{
    path::PathBuf, 
    ffi::{OsStr, OsString},
};

use crate::config::Config;


pub struct Resolver<'a> {
    config: &'a Config,
}

#[derive(Debug)]
pub enum Resolved {
    File(PathBuf),
    Markdown(PathBuf),
    Directory(PathBuf),
    None,
}

impl Resolver<'_> {
    pub fn new(config: &Config) -> Resolver {
        Resolver { config }
    }

    pub fn lookup(&self, uri: &http::Uri) -> Resolved {
        let mdext: &OsStr = OsStr::new("md");
        let relpath = force_relative(uri.path());
        // Check under webroot
        let mut path = self.config.rootdir.join(&relpath);
        if path.is_dir() {
            return Resolved::Directory(path);
        } else if path.is_file() {
            return if path.extension() == Some(mdext) { 
                Resolved::Markdown(path)
            } else {
                Resolved::File(path)
            };
        }
        // Check with extra markdown extension
        if let Some(fname) = path.file_name() {
            // XXX This would be nicer: path.as_mut_os_string().push(MDEXT);
            let mut tmp = OsString::with_capacity(fname.len() + mdext.len() + 1);
            tmp.push(fname);
            tmp.push(".");
            tmp.push(mdext);
            path.set_file_name(tmp);
            if path.is_file() {
                return Resolved::Markdown(path);
            }
        }
        // Look in staticdir
        let path = self.config.staticdir.join(&relpath);
        if path.is_file() {
            return Resolved::File(path);
        }
        // Finally, nothing is found
        return Resolved::None;
    }

    pub fn config(&self) -> &Config {
        return self.config;
    }
}

fn force_relative(uri: &str) -> PathBuf {
    assert!(uri.starts_with('/'), 
            "The uri path for a request should always be absolute");
    PathBuf::from(&uri[1..])
}
