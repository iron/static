#![crate_name = "static"]
#![deny(missing_docs)]
#![deny(warnings)]
#![feature(phase)]

//! Static file-serving handler.

#[phase(plugin)]
extern crate regex_macros;
extern crate regex;
extern crate time;

extern crate hyper;
extern crate iron;
#[phase(plugin, link)]
extern crate log;
extern crate mount;
extern crate "conduit-mime-types" as mime_types;

pub use cache_handler::StaticWithCache;
pub use static_handler::Static;

mod cache_handler;
mod requested_path;
mod static_handler;
