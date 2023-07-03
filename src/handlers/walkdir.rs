//! Handling directory requests
//!
//! Finds all files in the current directory and forms either html or json to represent them.

use std::{
    path::{Path, PathBuf, StripPrefixError}, 
    ffi::{OsStr, OsString}
};

use walkdir::WalkDir;

use serde::Serialize;

/*
FS-Tree representation
----------------------

In order to send the whole tree over, we need to recursively read in the directory contents, and
arrange it in a Tree data structure. This means, we need to decide how to implement the tree in
rust, and how to read it from the filesystem.

Because we don't need to modify the tree after creation, there is no need to track parents or use references.
All data can be saved as strings, and only directories (and links?) would need children.
*/

/* This format was abandoned, because accessing the file list inside the enum was a pain
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum DirTree {
    Dir { name: String, path: String, files: Vec<DirTree> },
    File { name: String, path: String },
}

pub fn dummy_dir() -> DirTree {
    use DirTree::*;
    Dir { name: "dir1".to_string(), path: "/dir1".to_string(), files: vec![
        Dir { name: "dir1A".to_string(), path: "/dir1/dir1A".to_string(), files: vec![
            File { name: "file1A.A".to_string(), path: "/dir1/dir1A/file1A.A".to_string() },
            File { name: "file1A.B".to_string(), path: "/dir1/dir1A/file1A.B".to_string() },
            File { name: "file1A.C".to_string(), path: "/dir1/dir1A/file1A.C".to_string() },
        ]},
        File { name: "file1.A".to_string(), path: "/dir1/file1.A".to_string() },
        File { name: "file1.B".to_string(), path: "/dir1/file1.B".to_string() },
        File { name: "file1.C".to_string(), path: "/dir1/file1.C".to_string() },
    ]}
}
*/

/* Using Separate structs for files and directories makes it much easier to build from WalkDir
 * In addition, it forces my to store separate lists of subdirectorys and files, which means the
 * values are already sorted by type.
 */
#[derive(Debug, Clone, Serialize)]
pub struct Directory { 
    name: String, 
    // NOTE(jladan): A path is best for creating the tree, but if it is used for links, this will
    // need a `/` prepended to it.
    path: PathBuf, 
    dirs: Vec<Directory>,
    files: Vec<File>,
}

#[derive(Debug, Clone, Serialize)]
pub struct File { 
    name: String, 
    path: String,
}

impl Directory {
    fn new(name: &str, path: &Path) -> Self {
        Self {
            name: name.to_string(),
            path: path.to_path_buf(),
            dirs: Vec::new(),
            files: Vec::new(),
        }
    }
}

impl File {
    fn new(name: &OsStr, path: &Path) -> Self {
        Self { 
            name: name.to_string_lossy().to_string(), 
            path: path.to_string_lossy().to_string(),
        }
    }
}


/*
FS traversal
------------

Walkdir is just a depth first iterator through the fs tree. The entire path is shown during the
walk, so tree-structure can be determined through the path prefix.

Instead of using walkdir, we might manually traverse the file-system. This means we can integrate
the tree formation into the traversal, but we'd also have to manage all the logic of navigating the
tree. This would inherently be recursive.

Because walkdir is almost a depth-first iterator, it will be much easier to build the tree iteratively using a stack.
*/
pub fn walk_dir(path: &Path) -> Result<Directory, StripPrefixError> {
    let prefix = path;      // Prefix to strip from all paths
    let mut dirstack: Vec<Directory> = Vec::new();
    let mut walker = WalkDir::new(prefix).into_iter().filter_map(|e| e.ok());
    let mut curdir: Directory = if let Some(entry) = walker.next() {
        let stripped = entry.path().strip_prefix(prefix)?;
        Directory::new("/", stripped)
    } else {
        Directory::new("/", &PathBuf::from(""))
    };
    for entry in walker {
        let stripped = entry.path().strip_prefix(prefix)?;
        if !stripped.starts_with(&curdir.path) {
            // Left the current directory
            // Need to find parent in the stack
            while let Some(mut prevdir) = dirstack.pop() {
                // Add the current directory to its parent
                format_dir(&mut curdir.path);
                prevdir.dirs.push(curdir);
                curdir = prevdir;
                // Continue until we've found the parent
                if stripped.starts_with(&curdir.path) {
                    break;
                }
            }
        }
        // Now perform logic on current entry
        if entry.file_type().is_file() {
            curdir.files.push(File::new(entry.file_name(), stripped));
        } else if entry.file_type().is_dir() {
            // Push current directory to stack, and start processing next one
            // NOTE(jladan): new directory still needs to be added to "current"
            dirstack.push(curdir);
            curdir = Directory::new(&entry.file_name().to_string_lossy(), stripped);
        }
    }
    // Now, unstack all the way to root
    while let Some(mut prevdir) = dirstack.pop() {
        // Add the current directory to its parent
        format_dir(&mut curdir.path);
        prevdir.dirs.push(curdir);
        curdir = prevdir;
    }
    return Ok(curdir)
}

fn format_dir(a: &mut PathBuf) {
    a.as_mut_os_string().push("/");
}

fn make_abs(a: &PathBuf) -> PathBuf {
    let mut built = OsString::with_capacity(a.as_os_str().len() + 1);
    built.push("/");
    built.push(a.as_os_str());
    built.into()
}

fn concat_osstr(a: &OsStr, b: &OsStr) -> OsString {
    let mut ret = OsString::with_capacity(a.len() + b.len());
    ret.push(a);
    ret.push(b);
    ret
}
