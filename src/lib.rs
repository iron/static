#![crate_name = "static"]
#![deny(missing_docs)]
#![deny(warnings)]
#![allow(unstable)]

//! Static file-serving handler.

extern crate regex;
extern crate time;
extern crate error;
extern crate iron;
extern crate hyper;
extern crate log;
extern crate mount;


pub use cache_handler::StaticWithCache;
pub use static_handler::Static;

mod cache_handler;
mod requested_path;
mod static_handler;
