use std::io::fs::PathExtensions;
use time::{mod, Timespec};

use iron::{status, Handler, IronResult, IronError, Request, Response, Set};
use iron::errors::FileError;
use iron::response::modifiers::Status;

use requested_path::RequestedPath;
use {Static};

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
        match self.static_handler.call(request) {
            Err(error) => Err(error),

            Ok(mut response) => {
                response.headers.cache_control =
                    Some("public, max-age=31536000".to_string());
                response.headers.last_modified =
                    Some(time::at(modified));

                Ok(response)
            },
        }
    }
}

impl Handler for StaticWithCache {
    fn call(&self, request: &mut Request) -> IronResult<Response> {
        let requested_path = RequestedPath::new(&self.static_handler.root_path, request);

        if requested_path.should_redirect(request) {
            return self.static_handler.call(request);
        }

        match requested_path.get_file() {
            Some(file) => {
                let last_modified_time = match file.stat() {
                    Err(error) => return Err(box FileError(error) as IronError),

                    Ok(file_stat) => {
                        Timespec::new((file_stat.modified / 1000) as i64, 0)
                    }
                };

                let if_modified_since = match request.headers.if_modified_since {
                    None => return self.defer_and_cache(request, last_modified_time),
                    Some(tm) => tm.to_timespec(),
                };

                if last_modified_time <= if_modified_since {
                    Ok(Response::new().set(Status(status::NotModified)))
                } else {
                    self.defer_and_cache(request, last_modified_time)
                }
            },

            None => self.static_handler.call(request)
        }
    }
}
