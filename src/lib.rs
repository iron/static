#![crate_name = "static"]
#![deny(missing_docs)]
#![deny(warnings)]
#![feature(core, io, path, std_misc)]

//! Static file-serving handler.

extern crate time;

extern crate hyper;
extern crate iron;
#[macro_use]
extern crate log;
extern crate mount;
extern crate "conduit-mime-types" as mime_types;

pub use static_handler::{Static, Cache};

mod requested_path;
mod static_handler;
