#![crate_name = "staticfile"]
#![deny(missing_docs)]
#![deny(warnings)]
#![feature(std_misc, path_ext)]
#![feature(metadata_ext)]

//! Static file-serving handler.

extern crate time;

extern crate iron;
#[macro_use]
extern crate log;
extern crate mount;

pub use static_handler::{Static, Cache};

mod requested_path;
mod static_handler;
