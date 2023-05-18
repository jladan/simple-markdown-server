# Zettel-Web

A viewer for files in a zettelkasten system. The idea is that markdown is
dynamically converted to html, so that it can be viewed in the web browser. All
other files will be presented unchanged.

## Features

- [ ] Custom configuration 
    - [x] web root and static dir as environment variables
    - [ ] From arguments
        - [ ] directory for the zettelkasten
        - [ ] styling files
        - [ ] headers and footers
        - [ ] localhost or 0.0.0.0 (localhost only allows same-computer connections)
        - [ ] port number
        - [ ] auto-open browser
    - [ ] Config file support
- [ ] Web server
    - [x] uri maps to filesystem with ZETTEL_DIR as root
    - [ ] "virtual" filesystem to check for static files (like css)
    - [ ] show a directory listing for directory requests
    - [ ] HEAD request support
    - [ ] 404 page properly handled
        - [x] return 404 response
        - [ ] include 404 page
        - [ ] probably some help for navigating on a 404
    - [ ] `.md` or `.html` unnecessary
    - [ ] watch for file changes, and auto-refresh page
- [ ] Document conversion
    - [x] Convert `.md` to html automatically
        - [x] strait conversion
        - [x] add header and footer
        - [x] handle "special characters"
    - [x] Support for images
        - [x] return images
    - [ ] "Smart links" -- remap links to find closest match
    - [ ] Metadata from yaml headers
- [ ] General Interface
    - [ ] file browser pane
    - [ ] search bar by file name
    - [ ] button to open in neovim
- [ ] ZettelServer integration
    - [ ] Pull file list and links from zettelserver
    - [ ] ...
