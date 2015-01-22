static [![Build Status](https://secure.travis-ci.org/iron/static.png?branch=master)](https://travis-ci.org/iron/static)
====

> Static file-serving handler for the [Iron](https://github.com/iron/iron) web framework.

## Example

This example uses the [mounting handler][mounting-handler] to serve files from the documentation directory.

```rust
let mut mount = Mount::new();

// Serve the documentation at /
mount.mount("/", Static::new(Path::new("target/doc")));

Iron::new(mount).listen("127.0.0.1:3000");
```

Note that `static` is a reserved keyword, so the crate will need to be imported as `extern crate "static" as static_file;`.

See [`examples/doc_server.rs`](examples/doc_server.rs) for a complete example that you can compile.

## Overview

static is a part of Iron's [core bundle](https://github.com/iron/core).

- Serve static files from a given path.

It works well in combination with the [mounting handler][mounting-handler].

## Installation

If you're using a `Cargo.toml` to manage dependencies, just add the `static` package to the toml:

```toml
[dependencies]
static = "*"
```

Otherwise, `cargo build`, and the rlib will be in your `target` directory.

## [Documentation](http://ironframework.io/doc/static)

Along with the [online documentation](http://ironframework.io/doc/static),
you can build a local copy with `cargo doc`.

## Get Help

One of us ([@reem](https://github.com/reem/), [@zzmp](https://github.com/zzmp/),
[@theptrk](https://github.com/theptrk/), [@mcreinhard](https://github.com/mcreinhard))
is usually on `#iron` on the mozilla irc. Come say hi and ask any questions you might have.
We are also usually on `#rust` and `#rust-webdev`.

[mounting-handler]: https://github.com/iron/mount
