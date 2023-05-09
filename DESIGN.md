# Design decisions

I am choosing to build this as a learning exercise, so the code is intended to
be fairly low-level (DIY). However, using external crates is also valuable to
learn, so where appropriate, I will use those.

## HTTP server

In the rust book, they already build a very rudimentary HTTP server for a "hello
world" application. However, implementing a working HTTP1/HTTP2 server is
tedious and complicated -- particularly managing all of the requests and
headers.

Looking at established rust programs (like cargo), the **hyper** crate appears
to be the best http server/client package.

## Concurrency / Asynchronous runtime

**Tokio** appears to be the defacto standard for network applications using an
async/await runtime. This could be the *easy* way to implement my app, because
there are so many examples of it -- the `hyper` docs use tokio in all of their
examples.

However, I think I will learn more by initially **not** using Tokio. This is
approprate, because I'm building a single-client webserver intended for only
localhost connections. As noted in the (Tokio tutorial)[https://tokio.rs/tokio/tutorial], 

> **When not to use tokio**...
> Sending a single web request. The place where Tokio gives you an advantage is
> when you need to do many things at the same time. If you need to use a library
> intended for asynchronous Rust such as reqwest, but you don't need to do a lot
> of things at once, you should prefer the blocking version of that library, as
> it will make your project simpler. Using Tokio will still work, of course, but
> provides no real advantage over the blocking API.

They also note that when a project "simply needs to read a lot of files, Tokio
provides no advantage compared to an ordinary threadpool." So, I'll use a
threadpool. This should give me experience in both threadpools, and learning how
to bridge the gap between blocking and asynchronous libraries.

### Threadpool

The most used threadpool crate appears to be `threadpool`. It's very basic, and
stable (last updated 3 years ago). There is also an option for a "scoped
threadpool", but these appear to be older, poorly documented, and I don't expect
to need the extra functionality.

There is also a threadpool provided by **Rayon**, but that project is focused on
data-parallelism, which certainly is not my use case. There is no need to import
all of Rayon to simply use a threadpool. That crate would be more
useful for other projects that I have in mind.

## Markdown

Looting through options for markdown, the most used and best supported seems to be
(pulldown-cmark)[https://crates.io/crates/pulldown-cmark]. As an actual parser,
it seems a little odd (it's a pull parser, so it reports events rather than
creating an AST), but I shouldn't need, and it has a built-in html generator, so meh.
