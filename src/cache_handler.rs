use std::old_io::fs::PathExtensions;
use std::old_io::IoError;
use time::{self, Timespec};

use std::error::Error;
use std::fmt::{self, Debug};

use iron::{status, Handler, IronResult, IronError, Request, Response};

use requested_path::RequestedPath;
use {Static};

#[derive(Debug)]
struct FileError(IoError);

impl Error for FileError {
    fn description(&self) -> &str { self.0.description() }
}

impl fmt::Display for FileError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Debug::fmt(self, f)
    }
}

/// Extends the `Static` handler with 304 caching.
///
/// If the client has a cached version of the requested file and the file hasn't
/// been modified since it was cached, this handler returns the
/// "304 Not Modified" response instead of the actual file.
pub struct StaticWithCache {
    static_handler: Static
}

impl StaticWithCache {
    /// Create a new instance of `StaticWithCache` with a given root path.
    ///
    /// If `Path::new("")` is given, files will be served from the current
    /// directory.
    pub fn new(root_path: Path) -> StaticWithCache {
        StaticWithCache { static_handler: Static::new(root_path) }
    }

    // Defers to the static handler, but adds cache headers to the response.
    fn defer_and_cache(&self, request: &mut Request,
                       modified: Timespec) -> IronResult<Response> {
        use hyper::header::{CacheControl, CacheDirective, LastModified};

        match self.static_handler.handle(request) {
            Err(error) => Err(error),

            Ok(mut response) => {
                response.headers.set(CacheControl(vec![CacheDirective::Public, CacheDirective::MaxAge(31536000)]));
                response.headers.set(LastModified(time::at(modified)));
                Ok(response)
            },
        }
    }
}

impl Handler for StaticWithCache {
    fn handle(&self, request: &mut Request) -> IronResult<Response> {
        use hyper::header::IfModifiedSince;

        let requested_path = RequestedPath::new(&self.static_handler.root_path, request);

        if requested_path.should_redirect(request) {
            return self.static_handler.handle(request);
        }

        match requested_path.get_file() {
            Some(file) => {
                let last_modified_time = match file.stat() {
                    Err(error) => return Err((IronError::new(FileError(error), status::Ok))),

                    Ok(file_stat) => {
                        Timespec::new((file_stat.modified / 1000) as i64, 0)
                    }
                };

                let if_modified_since = match request.headers.get::<IfModifiedSince>()
                                                             .cloned() {
                    None => return self.defer_and_cache(request, last_modified_time),
                    Some(tm) => tm.to_timespec(),
                };

                if last_modified_time <= if_modified_since {
                    Ok(Response::with(status::NotModified))
                } else {
                    self.defer_and_cache(request, last_modified_time)
                }
            },

            None => self.static_handler.handle(request)
        }
    }
}
