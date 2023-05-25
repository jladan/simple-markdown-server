//! Handling directory requests
//!
//! Finds all files in the current directory and forms either html or json to represent them.

use std::{
    io,
    fs,
    path::{Path, PathBuf}, ffi::OsString,
};

use serde::{Deserialize, Serialize};
use serde_json;

#[derive(Debug)]
pub enum Error {
    IO(io::Error),
    Encode(OsString),
    JSON(serde_json::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "An directory-scanning error occured: {self:?}")
    }
}

impl From<io::Error> for Error {
    fn from(value: io::Error) -> Self {
        Self::IO(value)
    }
}

impl From<serde_json::Error> for Error {
    fn from(value: serde_json::Error) -> Self {
        Self::JSON(value)
    }
}

impl From<OsString> for Error {
    fn from(value: OsString) -> Self {
        Self::Encode(value)
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "path")]
enum Entry {
    Dir(String),
    File(String),
    Link(String),
    Other(String),
}

impl Entry {
    fn strip_prefix(self, base: &Path) -> Option<Self> {
        let base = base.to_str();
        if let Some(base) = base {
            match self {
                Self::Dir(p) => p.strip_prefix(base).map(|p| Self::Dir(p.to_string())),
                Self::File(p) => p.strip_prefix(base).map(|p| Self::File(p.to_string())),
                Self::Link(p) => p.strip_prefix(base).map(|p| Self::Link(p.to_string())),
                Self::Other(p) => p.strip_prefix(base).map(|p| Self::Other(p.to_string())),
            }
        } else {
            None
        }
    }

}

// XXX This try-from is required because read_dir returns Result<DirEntry>
impl TryFrom<io::Result<fs::DirEntry>> for Entry {
    type Error = Error;

    fn try_from(value: io::Result<fs::DirEntry>) -> Result<Self, Self::Error> {
        let dir_entry = value?;
        dir_entry.try_into()
    }
}

// XXX Must implement as TryFrom, because getting the filetype may result in an error
impl TryFrom<fs::DirEntry> for Entry {
    type Error = Error;

    fn try_from(value: fs::DirEntry) -> Result<Self, Self::Error> {
        let ftype = value.file_type()?;
        if ftype.is_file() {
            Ok(Entry::File(value.path().into_os_string().into_string()?))
        } else if ftype.is_dir() {
            let mut path = value.path().into_os_string().into_string()?;
            path.push('/');
            Ok(Entry::Dir(path))
        } else if ftype.is_symlink() {
            Ok(Entry::Link(value.path().into_os_string().into_string()?))
        } else {
            Ok(Entry::Other(value.path().into_os_string().into_string()?))
        }
    }
}

pub fn get_html(path: &Path) -> Result<String, Error> {
    let entries = read_contents(path)?;
    return Ok(to_html(entries));
}

pub fn get_json(path: &Path) -> Result<String, Error> {
    let entries = read_contents(path)?;
    return serde_json::to_string(&entries).map_err(Error::from);
}

fn lift<T, E>(r: Result<Option<T>, E>) -> Option<Result<T, E>> {
    match r {
        Ok(Some(inner)) => Some(Ok(inner)),
        Ok(None) => None,
        Err(e) => Some(Err(e)),
    }
}

fn read_contents(path: &Path) -> Result<Vec<Entry>, Error> {
    let mut ret: Result<Vec<Entry>, Error> = path.read_dir()?
        .map(|e| Entry::try_from(e))
        // XXX any failure to strip prefix throws the entry away
        .map(|r| r.map(|e| e.strip_prefix(&path)))
        .filter_map(|r| lift(r))
        .collect();
    match ret.as_mut() {
        Ok(entries) => entries.sort(),
        Err(_) => ()
    };
    return ret;
}

fn to_html(entries: Vec<Entry>) -> String {
    let mut s = String::from("<ul>\n");
    for e in entries {
        let p = match e {
            Entry::Dir(p) => PathBuf::from(p),
            Entry::File(p) => PathBuf::from(p),
            Entry::Link(p) => PathBuf::from(p),
            Entry::Other(_) => PathBuf::from("")
        };
        s.push_str(&format!("<li><a href=\"{}\">{}</a></li>\n", p.display(), p.file_name().unwrap().to_str().unwrap()))
    }
    s.push_str("</ul>");
    return s
}

#[cfg(test)]
mod tests {
    use super::*;

    // Intended to run with '-- --show-output'
    #[test]
    fn gets_dir_contents() {
        for e in read_contents(&PathBuf::from("src")).unwrap().into_iter() {
            println!("{e:?}");
        }
    }

    #[test]
    fn serializes() {
        println!("{}", serde_json::to_string(&read_contents(&PathBuf::from("src")).unwrap()).unwrap());
    }

    #[test]
    fn makes_html() {
        println!("{}", to_html(read_contents(&PathBuf::from("src")).unwrap()));
    }
}
