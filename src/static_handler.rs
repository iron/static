use iron::{Request, Response, Url, Handler, IronResult, Set};
use iron::response::modifiers::{Status, Body, Redirect, ContentType};
use iron::status;
use mount::OriginalUrl;
use requested_path::RequestedPath;
use mime_types::Types;

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
pub struct Static {
    /// The path this handler is serving files from.
    pub root_path: Path,
    /// The stored mime-type mapping
    pub types: Types,
}

impl Static {
    /// Create a new instance of `Static` with a given root path.
    ///
    /// If `Path::new("")` is given, files will be served from the current directory.
    pub fn new(root_path: Path) -> Static {
        Static { root_path: root_path, types: Types::new().unwrap() }
    }
}

impl Handler for Static {
    fn call(&self, req: &mut Request) -> IronResult<Response> {
        let requested_path = RequestedPath::new(&self.root_path, req);

        // If the URL ends in a slash, serve the file directly.
        // Otherwise, redirect to the directory equivalent of the URL.
        if requested_path.should_redirect(req) {
            // Perform an HTTP 301 Redirect.
            let mut redirect_path = match req.extensions.get::<OriginalUrl, Url>() {
                Some(original_url) => original_url,
                None => &req.url
            }.clone();

            // Append the trailing slash
            //
            // rust-url automatically turns an empty string in the last
            // slot in the path into a trailing slash.
            redirect_path.path.push("".into_string());

            return Ok(Response::new().set(Status(status::MovedPermanently))
                          .set(Body(format!("Redirecting to {}", redirect_path)))
                          .set(Redirect(redirect_path)));
        }

        match requested_path.get_file() {
            Some(path) => {
                let mime = from_str(self.types.mime_for_path(&path))
                    .unwrap_or_else(|| from_str("text/plain").unwrap());
                Ok(Response::new()
                       .set(Status(status::Ok))
                       // Won't panic because we know the file exists from get_file
                       .set(Body(path))
                       .set(ContentType(mime)))
            }
            None =>
                // If no file is found, return a 404 response.
                Ok(Response::new().set(Status(status::NotFound)).set(Body("File not found")))
        }
    }
}
