extern crate iron;
extern crate "static" as static_file;
extern crate mount;

// This example serves the docs from target/doc/static at /doc/
//
// Run `cargo doc && cargo run --example doc_server`, then
// point your browser to http://127.0.0.1:3000/doc/

use iron::Iron;
use static_file::Static;
use mount::Mount;

fn main() {
    let mut mount = Mount::new();

    // Serve the shared documentation at /
    mount.mount("/", Static::new(Path::new("target/doc")));

    Iron::new(mount).listen("127.0.0.1:3000").unwrap();

    println!("Doc server running on http://localhost:3000/doc/");
}
