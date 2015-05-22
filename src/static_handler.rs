use std::path::{PathBuf, Path};
use std::fs::PathExt;
use std::time::Duration;
use std::error::Error;
use std::fmt;
use time::{self, Timespec};
use filetime::FileTime;

use iron::prelude::*;
use iron::{Handler, status};
use iron::modifier::Modifier;
use iron::modifiers::Redirect;
use mount::OriginalUrl;
use requested_path::RequestedPath;

/// The static file-serving `Handler`.
///
/// This handler serves files from a single filesystem path, which may be absolute or relative.
/// Incoming requests are mapped onto the filesystem by appending their URL path to the handler's
/// root path. If the filesystem path corresponds to a regular file, the handler will attempt to
/// serve it. Otherwise, if the path corresponds to a directory containing an `index.html`,
/// the handler will attempt to serve that instead.
///
/// ## Errors
///
/// If the path doesn't match any real object in the filesystem, the handler will return
/// a Response with `status::NotFound`. If an IO error occurs whilst attempting to serve
/// a file, `FileError(IoError)` will be returned.
#[derive(Clone)]
pub struct Static {
    /// The path this handler is serving files from.
    pub root: PathBuf,
    cache: Option<Cache>,
}

impl Static {
    /// Create a new instance of `Static` with a given root path.
    ///
    /// If `Path::new("")` is given, files will be served from the current directory.
    pub fn new<P: AsRef<Path>>(root: P) -> Static {
        Static { root: root.as_ref().to_path_buf(), cache: None }
    }

    /// Specify the response's `cache-control` header with a given duration. Internally, this is
    /// a helper function to set a `Cache` on an instance of `Static`.
    ///
    /// ## Example
    ///
    /// ```ignore
    /// let cached_static_handler = Static::new(path).cache(Duration::days(30));
    /// ```
    pub fn cache(self, duration: Duration) -> Static {
        self.set(Cache::new(duration))
    }

    fn try_cache<P: AsRef<Path>>(&self, req: &mut Request, path: P) -> IronResult<Response> {
        match self.cache {
            None => Ok(Response::with((status::Ok, path.as_ref()))),
            Some(ref cache) => cache.handle(req, path.as_ref()),
        }
    }
}

impl Handler for Static {
    fn handle(&self, req: &mut Request) -> IronResult<Response> {
        let requested_path = RequestedPath::new(&self.root, req);

        // If the URL ends in a slash, serve the file directly.
        // Otherwise, redirect to the directory equivalent of the URL.
        if requested_path.should_redirect(req) {
            // Perform an HTTP 301 Redirect.
            let mut redirect_path = match req.extensions.get::<OriginalUrl>() {
                None => &req.url,
                Some(original_url) => original_url,
            }.clone();

            // Append the trailing slash
            //
            // rust-url automatically turns an empty string in the last
            // slot in the path into a trailing slash.
            redirect_path.path.push("".to_string());

            return Ok(Response::with((status::MovedPermanently,
                                      format!("Redirecting to {}", redirect_path),
                                      Redirect(redirect_path))));
        }

        match requested_path.get_file() {
            // If no file is found, return a 404 response.
            None => Err(IronError::new(NoFile, status::NotFound)),
            // Won't panic because we know the file exists from get_file.
            Some(path) => self.try_cache(req, path),
        }
    }
}

impl Set for Static {}

/// A modifier for `Static` to specify a response's `cache-control`.
#[derive(Clone)]
pub struct Cache {
    /// The length of time the file should be cached for.
    pub duration: Duration,
}

impl Cache {
    /// Create a new instance of `Cache` with a given duration.
    pub fn new(duration: Duration) -> Cache {
        Cache { duration: duration }
    }

    fn handle<P: AsRef<Path>>(&self, req: &mut Request, path: P) -> IronResult<Response> {
        use iron::headers::{IfModifiedSince, HttpDate};
        let path = path.as_ref();

        let last_modified_time = match path.metadata() {
            Err(error) => return Err(IronError::new(error, status::InternalServerError)),
            Ok(metadata) => {
                let time = FileTime::from_last_modification_time(&metadata);
                Timespec::new(time.seconds() as i64, time.nanoseconds() as i32)
            },
        };

        let if_modified_since = match req.headers.get::<IfModifiedSince>().cloned() {
            None => return Ok(self.response_with_cache(path, last_modified_time)),
            Some(IfModifiedSince(HttpDate(time))) => time.to_timespec(),
        };

        if last_modified_time <= if_modified_since {
            Ok(Response::with(status::NotModified))
        } else {
            Ok(self.response_with_cache(path, last_modified_time))
        }
    }

    fn response_with_cache<P: AsRef<Path>>(&self, path: P, modified: Timespec) -> Response {
        use iron::headers::{CacheControl, LastModified, CacheDirective, HttpDate};

        let mut response = Response::with((status::Ok, path.as_ref()));
        let seconds = self.duration.secs() as u32;
        let cache = vec![CacheDirective::Public, CacheDirective::MaxAge(seconds)];
        response.headers.set(CacheControl(cache));
        response.headers.set(LastModified(HttpDate(time::at(modified))));
        response
    }
}

impl Modifier<Static> for Cache {
    fn modify(self, static_handler: &mut Static) {
        static_handler.cache = Some(self);
    }
}

/// Thrown if no file is found. It is always accompanied by a NotFound response.
#[derive(Debug)]
pub struct NoFile;

impl Error for NoFile {
    fn description(&self) -> &str { "File not found" }
}

impl fmt::Display for NoFile {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.description())
    }
}
