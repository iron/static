#![crate_name = "static"]
#![deny(missing_docs)]
#![deny(warnings)]
#![feature(phase)]
#[plugin]
#[no_link]

//! Static file-serving handler.

#[feature(plugin)]
extern crate regex_macros;
extern crate regex;
extern crate time;

extern crate hyper;
extern crate iron;
#[feature(plugin, link)]
extern crate log;
extern crate mount;


pub use cache_handler::StaticWithCache;
pub use static_handler::Static;

mod cache_handler;
mod requested_path;
mod static_handler;
