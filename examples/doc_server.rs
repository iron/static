#![feature(path)]

extern crate iron;
extern crate staticfile;
extern crate mount;

// This example serves the docs from target/doc/staticfile at /doc/
//
// Run `cargo doc && cargo test && ./target/doc_server`, then
// point your browser to http://127.0.0.1:3000/doc/

use std::path::Path;

use iron::Iron;
use staticfile::Static;
use mount::Mount;

fn main() {
    let mut mount = Mount::new();

    // Serve the shared JS/CSS at /
    mount.mount("/", Static::new(Path::new("target/doc/")));
    // Serve the static file docs at /doc/
    mount.mount("/doc/", Static::new(Path::new("target/doc/staticfile/")));
    // Serve the source code at /src/
    mount.mount("/src/", Static::new(Path::new("target/doc/src/staticfile/lib.rs.html")));

    println!("Doc server running on http://localhost:3000/doc/");

    Iron::new(mount).http("127.0.0.1:3000").unwrap();
}
