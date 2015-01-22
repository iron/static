//! This example shows how to serve static files at specific
//! mount points, and then delegate the rest of the paths to a router.
//!
//! It serves the docs from target/doc at the /docs/ mount point
//! and delegates the rest to a router, which itself defines a
//! handler for route /hello
//!
//! Make sure to generate the docs first with `cargo doc`,
//! then build the tests with `cargo test`,
//! then run the example with `cargo run --example router`
//!
//! Visit http://127.0.0.1:3000/hello to view the routed path.
//!
//! Visit http://127.0.0.1:3000/docs/mount/ to view the mounted docs.

extern crate iron;
extern crate mount;
extern crate router;
extern crate "static" as static_file;

use iron::status;
use iron::{Iron, Request, Response, IronResult};
use mount::Mount;
use router::Router;
use static_file::Static;

fn say_hello(req: &mut Request) -> IronResult<Response> {
    println!("Running send_hello handler, URL path: {}", req.url);
    Ok(Response::with((status::Ok, "This request was routed!")))
}

fn main() {
    let mut router = Router::new();
    router.get("/hello", say_hello);

    let mut mount = Mount::new();
    mount.mount("/", router)
         .mount("/docs/", Static::new(Path::new("target/doc")));

    Iron::new(mount).listen("192.168.1.41:3000").unwrap();
}
