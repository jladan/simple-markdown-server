//! Handling directory requests
//!
//! Finds all files in the current directory and forms either html or json to represent them.

use std::{
    io,
    fs,
    path::Path, 
    ffi::OsString,
};

use serde::{Deserialize, Serialize};
use serde_json;

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "path")]
pub enum Entry {
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
    type Error = DirError;

    fn try_from(value: io::Result<fs::DirEntry>) -> Result<Self, Self::Error> {
        let dir_entry = value?;
        dir_entry.try_into()
    }
}

// XXX Must implement as TryFrom, because getting the filetype may result in an error
impl TryFrom<fs::DirEntry> for Entry {
    type Error = DirError;

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

pub fn get_json(path: &Path) -> Result<String, DirError> {
    let entries = read_contents(path)?;
    return serde_json::to_string(&entries).map_err(DirError::from);
}

fn lift<T, E>(r: Result<Option<T>, E>) -> Option<Result<T, E>> {
    match r {
        Ok(Some(inner)) => Some(Ok(inner)),
        Ok(None) => None,
        Err(e) => Some(Err(e)),
    }
}

pub fn read_contents(path: &Path) -> Result<Vec<Entry>, DirError> {
    let mut ret: Result<Vec<Entry>, DirError> = path.read_dir()?
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

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
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

}

#[derive(Debug)]
pub enum DirError {
    IO(io::Error),
    Encode(OsString),
    JSON(serde_json::Error),
}

impl std::error::Error for DirError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            DirError::IO(err) => err.source(),
            DirError::JSON(err) => err.source(),
            DirError::Encode(_) => None,
        }
    }
}

impl std::fmt::Display for DirError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "An directory-scanning error occured: {self:?}")
    }
}

impl From<io::Error> for DirError {
    fn from(value: io::Error) -> Self {
        Self::IO(value)
    }
}

impl From<serde_json::Error> for DirError {
    fn from(value: serde_json::Error) -> Self {
        Self::JSON(value)
    }
}

impl From<OsString> for DirError {
    fn from(value: OsString) -> Self {
        Self::Encode(value)
    }
}

