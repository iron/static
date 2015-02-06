#![allow(unstable)]
#![crate_name = "static"]
#![deny(missing_docs)]
#![deny(warnings)]

//! Static file-serving handler.

#[macro_use]
extern crate regex;
extern crate time;

extern crate hyper;
extern crate iron;

#[macro_use]
extern crate log;
extern crate mount;


pub use cache_handler::StaticWithCache;
pub use static_handler::Static;

mod cache_handler;
mod requested_path;
mod static_handler;
