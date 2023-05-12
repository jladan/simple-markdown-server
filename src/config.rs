//! Configuration Module
//!

use std::path::{PathBuf, Path};


/// The config object to handle how pages are served
///
/// # Properties
/// - `rootdir` the root location of all the files to be served
/// - `staticdir` the directory that holds all "static" files
/// - `header` the file name (relative to `staticdir`) of the header to prepend to all md files
/// - `footer` the file name (relative to `staticdir`) of the footer to append to all md files
#[derive(Debug, PartialEq, Eq)]
pub struct Config {
    pub rootdir: PathBuf,
    pub staticdir: PathBuf,
    pub header: PathBuf,
    pub footer: PathBuf,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            rootdir: PathBuf::from("./"),
            staticdir: PathBuf::from("./"),
            header: PathBuf::from("./header.html"),
            footer: PathBuf::from("./footer.html"),
        }
    }
}

/// Builder for the configuration object
///
/// Only handles setting config values from variables.
/// Reading from commandline arguments or environment must be done separately.
pub struct ConfigBuilder {
    rootdir: PathBuf,
    staticdir: PathBuf,
    header: PathBuf,
    footer: PathBuf,
}

impl ConfigBuilder {
    /// Start building a config from the defaults
    pub fn new() -> ConfigBuilder {
        let config = Config::default();
        ConfigBuilder { 
            rootdir: config.rootdir,
            staticdir: config.staticdir,
            header: config.header,
            footer: config.footer,
        }
    }
    
    /// Returns the finished Config
    pub fn build(self) -> Config {
        let header = Path::join(self.staticdir.as_path(), self.header);
        let footer = Path::join(self.staticdir.as_path(), self.footer);
        Config {
            rootdir: self.rootdir,
            staticdir: self.staticdir,
            header,
            footer,
        }
    }

    /// Set the root directory for the fileserver
    pub fn set_root(mut self, path: &str) -> ConfigBuilder{
        self.rootdir.clear();
        self.rootdir.push(path);
        self
    }

    /// Set the static directory for the fileserver
    pub fn set_static(mut self, path: &str) -> ConfigBuilder{
        self.staticdir.clear();
        self.staticdir.push(path);
        self
    }

    /// Set the header file to be prepended to all markdown pages
    pub fn set_header(mut self, header: &str) -> ConfigBuilder{
        self.header.clear();
        self.header.push(header);
        self
    }

    /// Set the footer file to be appended to all markdown pages
    pub fn set_footer(mut self, footer: &str) -> ConfigBuilder{
        self.footer.clear();
        self.footer.push(footer);
        self
    }

}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builder_uses_default() {
        let def = Config::default();
        let built = ConfigBuilder::new().build();
        assert_eq!(def, built);
    }
    
    #[test]
    fn builder_sets_root() {
        let built = ConfigBuilder::new()
            .set_root("rootdir")
            .build();
        assert_eq!(PathBuf::from("rootdir"), built.rootdir);
    }

    #[test]
    fn builder_sets_longer_root() {
        let built = ConfigBuilder::new()
            .set_root("path/to/rootdir/")
            .build();
        assert_eq!(PathBuf::from("path/to/rootdir"), built.rootdir);
    }
    
    #[test]
    fn builder_sets_header_rel() {
        let built = ConfigBuilder::new()
            .set_static("static")
            .set_header("header.html")
            .build();
        assert_eq!(PathBuf::from("static/header.html"), built.header);
    }
    
    #[test]
    fn builder_sets_footer_rel() {
        // Note that `footer` is set relative to `staticdir`
        let built = ConfigBuilder::new()
            .set_static("static")
            .set_footer("footer.html")
            .build();
        assert_eq!(PathBuf::from("static/footer.html"), built.footer);
    }

    #[test]
    fn builder_sets_header_abs() {
        // If the header file is an absolute path, then staticdir is not used
        let built = ConfigBuilder::new()
            .set_static("static")
            .set_header("/header.html")
            .build();
        assert_ne!(PathBuf::from("static/header.html"), built.header);
        assert_eq!(PathBuf::from("/header.html"), built.header);
    }

    #[test]
    fn builder_sets_footer_abs() {
        // If the footer file is an absolute path, then staticdir is not used
        let built = ConfigBuilder::new()
            .set_static("static")
            .set_footer("/footer.html")
            .build();
        assert_ne!(PathBuf::from("static/footer.html"), built.footer);
        assert_eq!(PathBuf::from("/footer.html"), built.footer);
    }
}
