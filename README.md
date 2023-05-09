# Zettel-Web

A viewer for files in a zettelkasten system. The idea is that markdown is
dynamically converted to html, so that it can be viewed in the web browser. All
other files will be presented unchanged.

## Features

- [ ] Custom configuration 
    - [ ] directory for the zettelkasten
    - [ ] styling files
    - [ ] headers and footers
    - [ ] localhost or 0.0.0.0 (localhost only allows same-computer connections)
    - [ ] port number
    - [ ] auto-open browser
- [ ] Web server
    - [ ] uri maps to filesystem with ZETTEL_DIR as root
    - [ ] 404 page properly handled
    - [ ] `.md` or `.html` unnecessary
    - [ ] probably some help for navigating on a 404
    - [ ] watch for file changes, and auto-refresh page
- [ ] Document conversion
    - [ ] Convert `.md` to html automatically
    - [ ] Support for images
    - [ ] "Smart links" -- remap links to find closest match
    - [ ] Metadata from headers
- [ ] General Interface
    - [ ] file browser pane
    - [ ] search bar by file name
    - [ ] button to open in neovim
- [ ] ZettelServer integration
    - [ ] Pull file list and links from zettelserver
    - [ ] ...
