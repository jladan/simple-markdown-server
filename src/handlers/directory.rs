//! Handling directory requests
//!
//! Finds all files in the current directory and forms either html or json to represent them.

use std::{
    io,
    fs,
    path::{Path, PathBuf}};

use serde::{Deserialize, Serialize};
use serde_json;

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "path")]
enum Entry {
    Dir(PathBuf),
    File(PathBuf),
    Link(PathBuf),
    Unknown,
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
            Ok(Entry::Unknown)
        }
    }
}


fn read_contents(path: &Path) -> std::io::Result<Vec<Entry>> {
    path.read_dir()?.map(|e| Entry::try_from(e)).collect()
}

fn to_json(entries: Vec<Entry>) -> Result<String, serde_json::error::Error> {
    serde_json::to_string(&entries)
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
        println!("{}", to_json(read_contents(&PathBuf::from("src")).unwrap()).unwrap());
    }
}
