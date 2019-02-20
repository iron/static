use std::path::{PathBuf, Path};
use std::fs;
use std::error::Error;
use std::fmt;

#[cfg(feature = "cache")]
use time::{self, Timespec};
#[cfg(feature = "cache")]
use std::time::Duration;

use iron::{Handler, Url, StatusCode};
use iron::prelude::*;
#[cfg(feature = "cache")]
use iron::modifier::Modifier;
use iron::modifiers::Redirect;
use mount::OriginalUrl;
use requested_path::RequestedPath;
use url;

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
/// a Response with `StatusCode::NOT_FOUND`. If an IO error occurs whilst attempting to serve
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
            None => Ok(Response::with((StatusCode::OK, path.as_ref()))),
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
                    io::ErrorKind::NotFound => StatusCode::NOT_FOUND,
                    io::ErrorKind::PermissionDenied => StatusCode::FORBIDDEN,
                    _ => StatusCode::INTERNAL_SERVER_ERROR,
                };

                return Err(IronError::new(e, status))
            },
        };

        // If the URL ends in a slash, serve the file directly.
        // Otherwise, redirect to the directory equivalent of the URL.
        if requested_path.should_redirect(&metadata, req) {
            // Perform an HTTP 301 Redirect.
            let mut original_url: url::Url = match req.extensions.get::<OriginalUrl>() {
                None => &req.url,
                Some(original_url) => original_url,
            }.clone().into();

            // Append the trailing slash
            //
            // rust-url automatically turns an empty string in the last
            // slot in the path into a trailing slash.
            original_url.path_segments_mut().unwrap().push("");
            let redirect_path = Url::from_generic_url(original_url).unwrap();

            return Ok(Response::with((StatusCode::MOVED_PERMANENTLY,
                                      format!("Redirecting to {}", redirect_path),
                                      Redirect(redirect_path))));
        }

        match requested_path.get_file(&metadata) {
            // If no file is found, return a 404 response.
            None => Err(IronError::new(NoFile, StatusCode::NOT_FOUND)),
            // Won't panic because we know the file exists from get_file.
            #[cfg(feature = "cache")]
            Some(path) => self.try_cache(req, path),
            #[cfg(not(feature = "cache"))]
            Some(path) => {
                let path: &Path = &path;
                Ok(Response::with((StatusCode::OK, path)))
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
        use iron::headers::IF_MODIFIED_SINCE;
        use httpdate::HttpDate;
        use std::str::FromStr;

        let path = path.as_ref();

        let (size, last_modified_time) = match fs::metadata(path) {
            Err(error) => return Err(IronError::new(error, StatusCode::INTERNAL_SERVER_ERROR)),
            Ok(metadata) => {
                use filetime::FileTime;

                let time = FileTime::from_last_modification_time(&metadata);
                (metadata.len(), Timespec::new(time.seconds() as i64, 0))
            },
        };

        let if_modified_since = match req.headers.get(IF_MODIFIED_SINCE).cloned() {
            None => return self.response_with_cache(req, path, size, last_modified_time),
            // TODO: Error handling for to_str() & from_str()
            Some(time) => HttpDate::from_str(time.to_str().unwrap()).unwrap().0.to_timespec(), 
        };

        if last_modified_time <= if_modified_since {
            Ok(Response::with(StatusCode::NOT_MODIFIED))
        } else {
            self.response_with_cache(req, path, size, last_modified_time)
        }
    }

    fn response_with_cache<P: AsRef<Path>>(&self,
                                           req: &mut Request,
                                           path: P,
                                           size: u64,
                                           modified: Timespec) -> IronResult<Response> {
        use iron::headers::{CACHE_CONTROL, LAST_MODIFIED, CONTENT_LENGTH, CONTENT_TYPE, ETAG};
        use httpdate::HttpDate;
        use iron::method::Method;
        use iron::mime;

        let seconds = self.duration.as_secs() as u32;
        let metadata = fs::metadata(path.as_ref());

        let metadata = try!(metadata.map_err(|e| IronError::new(e, StatusCode::INTERNAL_SERVER_ERROR)));

        let mut response = if req.method == Method::HEAD {
            let has_ct = req.headers.get(CONTENT_TYPE);
            let cont_type = match has_ct {
                None => mime::TEXT_PLAIN.as_ref().parse().unwrap(),
                Some(t) => t.clone()
            };
            let mut response = Response::with(StatusCode::OK);
            response.headers.insert(CONTENT_TYPE, cont_type);
            response.headers.insert(CONTENT_LENGTH, metadata.len().to_string().parse().unwrap());
            response
                
        } else {
            Response::with((StatusCode::OK, path.as_ref()))
        };

        response.headers.insert(CACHE_CONTROL, "public".parse().unwrap());;
        response.headers.insert(CACHE_CONTROL, format!("max-age={}", seconds).parse().unwrap());
        response.headers.insert(LAST_MODIFIED, format!("{}",HttpDate(time::at(modified))).parse().unwrap());
        response.headers.insert(ETAG, format!("{0:x}-{1:x}.{2:x}", size, modified.sec, modified.nsec).parse().unwrap());

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
