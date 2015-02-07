use std::old_io::fs::PathExtensions;
use iron::Request;

pub struct RequestedPath {
    pub path: Path,
}

impl RequestedPath {
    pub fn new(root_path: &Path, request: &Request) -> RequestedPath {
        let path = root_path.join(
            Path::new("").join_many(request.url.path.as_slice())
        );

        RequestedPath { path: path }
    }

    pub fn should_redirect(&self, request: &Request) -> bool {
        let last_url_element = request.url.path
            .as_slice()
            .last()
            .map(|s| s.as_slice());

        // As per servo/rust-url/serialize_path, URLs ending in a slash have an
        // empty string stored as the last component of their path. Rust-url
        // even ensures that url.path is non-empty by appending a forward slash
        // to URLs like http://example.com
        // Some middleware may mutate the URL's path to violate this property,
        // so the empty list case is handled as a redirect.
        let has_trailing_slash = match last_url_element {
            Some("") => true,
            _ => false,
        };

        self.path.is_dir() && !has_trailing_slash
    }

    pub fn get_file(self) -> Option<Path> {
        if self.path.is_file() {
            return Some(self.path);
        }

        let index_path = self.path.join("index.html");

        if index_path.is_file() {
            Some(index_path)
        } else {
            None
        }
    }
}
