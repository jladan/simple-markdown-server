# Simple Markdown Server

A viewer for files in a zettelkasten system. The idea is that markdown is
dynamically converted to html, so that it can be viewed in the web browser. All
other files will be presented unchanged.

This project was made as a learning exercise to make a functional rust program.
As such, it contains a lot of "not invented here"-style code, and a simple
threadpool rather than an asynchronous runtime like Tokio.

## Usage

Directories and markdown files are rendered using
[tera](https://tera.netlify.app/) templates. Static files for rendering
(javascript, css, etc.) can be placed in a separate `STATIC_DIR`, which is
shadowed into the `WEB_ROOT` directory. A sample of static files and templates
is provided in `samples`.

Configuration is done through environment variables. To start serving files, run
the commands,

```bash
export WEB_ROOT="/path/to/files"
export STATIC_DIR="/path/to/sample/static"
export TEMPLATE_DIR="/path/to/sample/templates"
cargo run
```

It is possible to use relative paths, but absolute paths are recommended.

## Features

Features in the server:
- Multithreaded to support multiple connections
- Full (recursive) directory contents serialized as json, or html
- Markdown rendering using `pulldown-cmark`, accessible either
    - inserted into a full document using `tera`, or
    - "raw" by adding an "x-partial: true" header to the GET request

The sample templates and static files implement:
- Latex rendering support with Mathjax,
- Syntax highlighting for code blocks using `highlight.js`
- File tree navigation
    - markdown files inserted into the main content view
    - (history, and non-markdown files currently broken due to this feature)

A full list of desired features to implement: 

- [ ] Custom configuration 
    - [x] web root and static dir as environment variables
    - [ ] From arguments
        - [ ] directory for the zettelkasten
        - [ ] styling files
        - [ ] localhost or 0.0.0.0 (localhost only allows same-computer connections)
        - [ ] port number
        - [ ] auto-open browser
    - [ ] Config file support
- [ ] Web server
    - [x] uri maps to filesystem with ZETTEL_DIR as root
    - [x] "virtual" filesystem to check for static files (like css)
    - [x] show a directory listing for directory requests
    - [x] HEAD request support
    - [ ] 404 page properly handled
        - [x] return 404 response
        - [ ] include 404 page
        - [ ] probably some help for navigating on a 404
    - [x] `.md` unnecessary
    - [ ] or `.html` unnecessary
    - [ ] watch for file changes, and auto-refresh page
- [ ] Document conversion
    - [x] render contents with html templates
    - [x] Convert `.md` to html automatically
        - [x] strait conversion
        - [x] handle "special characters"
    - [x] Support for images
        - [x] return images
    - [ ] "Smart links" -- remap links to find closest match
    - [ ] Metadata from yaml headers or toml header
- [ ] General Interface
    - [x] file browser pane
    - [ ] search bar by file name
    - [ ] button to open in neovim
    - [x] Client-side formatting updates on state-change
        - [x] Syntax highlighting
        - [x] Latex
