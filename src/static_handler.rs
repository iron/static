use std::path::{PathBuf, Path};
use std::fs;
use std::error::Error;
use std::fmt;

#[cfg(feature = "cache")]
use time::{self, Timespec};
#[cfg(feature = "cache")]
use std::time::Duration;

use iron::prelude::*;
use iron::{Handler, Url, status};
#[cfg(feature = "cache")]
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
    #[cfg(feature = "cache")]
    cache: Option<Cache>,
}

impl Static {
    /// Create a new instance of `Static` with a given root path.
    ///
    /// If `Path::new("")` is given, files will be served from the current directory.
    #[cfg(feature = "cache")]
    pub fn new<P: Into<PathBuf>>(root: P) -> Static {
        Static {
            root: root.into(),
            cache: None
        }
    }

    /// Create a new instance of `Static` with a given root path.
    ///
    /// If `Path::new("")` is given, files will be served from the current directory.
    #[cfg(not(feature = "cache"))]
    pub fn new<P: Into<PathBuf>>(root: P) -> Static {
        Static {
            root: root.into(),
        }
    }

    /// Specify the response's `cache-control` header with a given duration. Internally, this is
    /// a helper function to set a `Cache` on an instance of `Static`.
    ///
    /// ## Example
    ///
    /// ```ignore
    /// let cached_static_handler = Static::new(path).cache(Duration::from_secs(30*24*60*60));
    /// ```
    #[cfg(feature = "cache")]
    pub fn cache(self, duration: Duration) -> Static {
        self.set(Cache::new(duration))
    }

    #[cfg(feature = "cache")]
    fn try_cache<P: AsRef<Path>>(&self, req: &mut Request, path: P) -> IronResult<Response> {
        match self.cache {
            None => Ok(Response::with((status::Ok, path.as_ref()))),
            Some(ref cache) => cache.handle(req, path.as_ref()),
        }
    }
}

impl Handler for Static {
    fn handle(&self, req: &mut Request) -> IronResult<Response> {
        use std::io;

        let requested_path = RequestedPath::new(&self.root, req);

        let metadata = match fs::metadata(&requested_path.path) {
            Ok(meta) => meta,
            Err(e) => {
                let status = match e.kind() {
                    io::ErrorKind::NotFound => status::NotFound,
                    io::ErrorKind::PermissionDenied => status::Forbidden,
                    _ => status::InternalServerError,
                };

                return Err(IronError::new(e, status))
            },
        };

        // If the URL ends in a slash, serve the file directly.
        // Otherwise, redirect to the directory equivalent of the URL.
        if requested_path.should_redirect(&metadata, req) {
            // Perform an HTTP 301 Redirect.
            let mut original_url = match req.extensions.get::<OriginalUrl>() {
                None => &req.url,
                Some(original_url) => original_url,
            }.clone().into_generic_url();

            // Append the trailing slash
            //
            // rust-url automatically turns an empty string in the last
            // slot in the path into a trailing slash.
            original_url.path_segments_mut().unwrap().push("");
            let redirect_path = Url::from_generic_url(original_url).unwrap();

            return Ok(Response::with((status::MovedPermanently,
                                      format!("Redirecting to {}", redirect_path),
                                      Redirect(redirect_path))));
        }

        match requested_path.get_file(&metadata) {
            // If no file is found, return a 404 response.
            None => Err(IronError::new(NoFile, status::NotFound)),
            // Won't panic because we know the file exists from get_file.
            #[cfg(feature = "cache")]
            Some(path) => self.try_cache(req, path),
            #[cfg(not(feature = "cache"))]
            Some(path) => {
                let path: &Path = &path;
                Ok(Response::with((status::Ok, path)))
            },
        }
    }
}

impl Set for Static {}

/// A modifier for `Static` to specify a response's `cache-control`.
#[cfg(feature = "cache")]
#[derive(Clone)]
pub struct Cache {
    /// The length of time the file should be cached for.
    pub duration: Duration,
}

#[cfg(feature = "cache")]
impl Cache {
    /// Create a new instance of `Cache` with a given duration.
    pub fn new(duration: Duration) -> Cache {
        Cache { duration: duration }
    }

    fn handle<P: AsRef<Path>>(&self, req: &mut Request, path: P) -> IronResult<Response> {
        use iron::headers::{IfModifiedSince, HttpDate};

        let path = path.as_ref();

        let last_modified_time = match fs::metadata(path) {
            Err(error) => return Err(IronError::new(error, status::InternalServerError)),
            Ok(metadata) => {
                use filetime::FileTime;

                let time = FileTime::from_last_modification_time(&metadata);
                Timespec::new(time.seconds() as i64, 0)
            },
        };

        let if_modified_since = match req.headers.get::<IfModifiedSince>().cloned() {
            None => return Ok(self.response_with_cache(req, path, last_modified_time)),
            Some(IfModifiedSince(HttpDate(time))) => time.to_timespec(),
        };

        if last_modified_time <= if_modified_since {
            Ok(Response::with(status::NotModified))
        } else {
            Ok(self.response_with_cache(req, path, last_modified_time))
        }
    }

    fn response_with_cache<P: AsRef<Path>>(&self,
                                           req: &mut Request,
                                           path: P,
                                           modified: Timespec) -> IronResult<Response> {
        use iron::headers::{CacheControl, LastModified, CacheDirective, HttpDate};
        use iron::headers::{ContentLength, ContentType};
        use iron::method::Method;
        use iron::mime::{Mime, TopLevel, SubLevel};
        use iron::modifiers::Header;

        let seconds = self.duration.as_secs() as u32;
        let cache = vec![CacheDirective::Public, CacheDirective::MaxAge(seconds)];
        let metadata = fs::metadata(path.as_ref());

        if metadata.is_err() {
            return Err(IronError::new(metadata.unwrap_err(), status::InternalServerError))
        }

        let mut response = if req.method == Method::Head {
            let has_ct = req.headers.get::<ContentType>();
            let cont_type = match has_ct {
                None => ContentType(Mime(TopLevel::Text, SubLevel::Plain, vec![])),
                Some(t) => t.clone()
            };
            Response::with((status::Ok, Header(cont_type), Header(ContentLength(metadata.len()))))
        } else {
            Response::with((status::Ok, path.as_ref()));
        };

        response.headers.set(CacheControl(cache));
        response.headers.set(LastModified(HttpDate(time::at(modified))));

        Ok(response)
    }
}

#[cfg(feature = "cache")]
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
