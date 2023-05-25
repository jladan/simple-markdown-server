//! Handling directory requests
//!
//! Finds all files in the current directory and forms either html or json to represent them.

use std::{
    io,
    fs,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use serde_json;

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "path")]
enum Entry {
    Dir(PathBuf),
    File(PathBuf),
    Link(PathBuf),
    Other(PathBuf),
}

impl Entry {
    fn strip_prefix(self, base: &Path) -> Option<Self> {
        match self {
            Self::Dir(p) => p.strip_prefix(base).ok().map(|p| Self::Dir(p.to_path_buf())),
            Self::File(p) => p.strip_prefix(base).ok().map(|p| Self::File(p.to_path_buf())),
            Self::Link(p) => p.strip_prefix(base).ok().map(|p| Self::Link(p.to_path_buf())),
            Self::Other(p) => p.strip_prefix(base).ok().map(|p| Self::Other(p.to_path_buf())),
        }
    }

}

// XXX This try-from is required because read_dir returns Result<DirEntry>
impl TryFrom<io::Result<fs::DirEntry>> for Entry {
    type Error = io::Error;

    fn try_from(value: io::Result<fs::DirEntry>) -> Result<Self, Self::Error> {
        let dir_entry = value?;
        dir_entry.try_into()
    }
}

// XXX Must implement as TryFrom, because getting the filetype may result in an error
impl TryFrom<fs::DirEntry> for Entry {
    type Error = std::io::Error;

    fn try_from(value: fs::DirEntry) -> Result<Self, Self::Error> {
        let ftype = value.file_type()?;
        if ftype.is_file() {
            Ok(Entry::File(value.path()))
        } else if ftype.is_dir() {
            Ok(Entry::Dir(value.path()))
        } else if ftype.is_symlink() {
            Ok(Entry::Link(value.path()))
        } else {
            Ok(Entry::Other(value.path()))
        }
    }
}

pub fn get_html(path: &Path) -> io::Result<String> {
    let entries = read_contents(path)?;
    return Ok(to_html(entries));
}

pub fn get_json(path: &Path) -> io::Result<String> {
    let entries = read_contents(path)?;
    return Ok(serde_json::to_string(&entries).expect("Problem in serializing json"));
}

fn lift<T, E>(r: Result<Option<T>, E>) -> Option<Result<T, E>> {
    match r {
        Ok(Some(inner)) => Some(Ok(inner)),
        Ok(None) => None,
        Err(e) => Some(Err(e)),
    }
}

fn read_contents(path: &Path) -> io::Result<Vec<Entry>> {
    let mut ret: io::Result<Vec<Entry>> = path.read_dir()?
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
            Entry::Dir(p) => p,
            Entry::File(p) => p,
            Entry::Link(p) => p,
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
