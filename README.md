staticfile [![Build Status](https://secure.travis-ci.org/iron/staticfile.png?branch=master)](https://travis-ci.org/iron/staticfile)
====

> Static file-serving handler for the [Iron](https://github.com/iron/iron) web framework.

## Example

This example uses the [mounting handler][mounting-handler] to serve files from several directories.

```rust
let mut mount = Mount::new();

// Serve the shared JS/CSS at /
mount.mount("/", Static::new(Path::new("target/doc/")));
// Serve the static file docs at /doc/
mount.mount("/doc/", Static::new(Path::new("target/doc/staticfile/")));
// Serve the source code at /src/
mount.mount("/src/", Static::new(Path::new("target/doc/src/staticfile/lib.rs.html")));

Iron::new(mount).listen(Ipv4Addr(127, 0, 0, 1), 3000).unwrap();
```

See [`examples/doc_server.rs`](examples/doc_server.rs) for a complete example that you can compile.

## Overview

staticfile is a part of Iron's [core bundle](https://github.com/iron/core).

- Serve static files from a given path.

It works well in combination with the [mounting handler][mounting-handler].

## Installation

If you're using a `Cargo.toml` to manage dependencies, just add the `staticfile` package to the toml:

```toml
[dependencies.staticfile]

git = "https://github.com/iron/staticfile.git"
```

Otherwise, `cargo build`, and the rlib will be in your `target` directory.

## [Documentation](http://ironframework.io/doc/staticfile)

Along with the [online documentation](http://ironframework.io/doc/staticfile),
you can build a local copy with `cargo doc`.

## Get Help

One of us ([@reem](https://github.com/reem/), [@zzmp](https://github.com/zzmp/),
[@theptrk](https://github.com/theptrk/), [@mcreinhard](https://github.com/mcreinhard))
is usually on `#iron` on the mozilla irc. Come say hi and ask any questions you might have.
We are also usually on `#rust` and `#rust-webdev`.

[mounting-handler]: https://github.com/iron/mount
