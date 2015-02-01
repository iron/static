use iron::{Request, Response, Handler, IronResult};
use iron::status;
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
    pub root_path: Path
}

impl Static {
    /// Create a new instance of `Static` with a given root path.
    ///
    /// If `Path::new("")` is given, files will be served from the current directory.
    pub fn new(root_path: Path) -> Static {
        Static { root_path: root_path }
    }
}

impl Handler for Static {
    fn handle(&self, req: &mut Request) -> IronResult<Response> {
        let requested_path = RequestedPath::new(&self.root_path, req);

        // If the URL ends in a slash, serve the file directly.
        // Otherwise, redirect to the directory equivalent of the URL.
        if requested_path.should_redirect(req) {
            // Perform an HTTP 301 Redirect.
            let mut redirect_path = match req.extensions.get::<OriginalUrl>() {
                Some(original_url) => original_url,
                None => &req.url
            }.clone();

            // Append the trailing slash
            //
            // rust-url automatically turns an empty string in the last
            // slot in the path into a trailing slash.
            redirect_path.path.push("".to_string());

            return Ok(
                Response::with((
                    status::MovedPermanently,
                    Redirect(redirect_path))
            ))
        }

        match requested_path.get_file() {
            Some(path) => Ok(Response::with((status::Ok, path))),
            None =>
                // If no file is found, return a 404 response.
                Ok(Response::with((status::NotFound, "File not found")))
        }
    }
}
