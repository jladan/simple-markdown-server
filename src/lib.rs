use std::path::PathBuf;


/// The config object to handle how pages are served
#[derive(Debug, PartialEq, Eq)]
pub struct Config {
    pub rootdir: PathBuf,
    pub header: PathBuf,
    pub footer: PathBuf,
}

impl Config {
    fn default() -> Config {
        Config {
            rootdir: PathBuf::from("./"),
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
    config: Config,
}

impl ConfigBuilder {
    /// Start building a config from the defaults
    pub fn new() -> ConfigBuilder {
        let config = Config::default();
        ConfigBuilder { config }
    }
    
    /// Returns the finished Config
    pub fn build(self) -> Config {
        self.config
    }

    /// Set the root directory for the fileserver
    pub fn set_root(mut self, root: &str) -> ConfigBuilder{
        self.config.rootdir.clear();
        self.config.rootdir.push(root);
        self
    }

    /// Set the header file to be prepended to all markdown pages
    pub fn set_header(mut self, header: &str) -> ConfigBuilder{
        self.config.header.clear();
        self.config.header.push(header);
        self
    }

    /// Set the footer file to be appended to all markdown pages
    pub fn set_footer(mut self, footer: &str) -> ConfigBuilder{
        self.config.footer.clear();
        self.config.footer.push(footer);
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
    fn builder_sets_header() {
        let built = ConfigBuilder::new()
            .set_header("static/header.html")
            .build();
        assert_eq!(PathBuf::from("static/header.html"), built.header);
    }
    
    #[test]
    fn builder_sets_footer() {
        let built = ConfigBuilder::new()
            .set_footer("static/footer.html")
            .build();
        assert_eq!(PathBuf::from("static/footer.html"), built.footer);
    }
}
