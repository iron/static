#![crate_name = "staticfile"]
#![deny(missing_docs)]
#![deny(warnings)]

//! Static file-serving handler.

extern crate time;

#[cfg(feature = "cache")]
extern crate filetime;

extern crate iron;
extern crate mount;
extern crate url;

pub use static_handler::Static;
#[cfg(feature = "cache")]
pub use static_handler::Cache;

mod requested_path;
mod static_handler;
mod partial_file;
