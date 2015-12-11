use iron::Request;
use std::path::{PathBuf, Path};
use std::fs::{self, Metadata};
use std::convert::AsRef;
use url::percent_encoding::percent_decode;

pub struct RequestedPath {
    pub path: PathBuf,
}

#[inline]
fn decode_percents(string: &String) -> String {
    String::from_utf8(percent_decode(string.as_bytes())).unwrap()
}

impl RequestedPath {
    pub fn new<P: AsRef<Path>>(root_path: P, request: &Request) -> RequestedPath {
        let mut path = root_path.as_ref().to_path_buf();
        let decoded_req_path = request.url.path.iter().map(decode_percents);
        path.extend(decoded_req_path);

        RequestedPath { path: path }
    }

    pub fn should_redirect(&self, metadata: &Metadata, request: &Request) -> bool {
        let last_url_element = request.url.path
            .last()
            .map(|s| s.as_ref());

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

        metadata.is_dir() && !has_trailing_slash
    }

    pub fn get_file(self, metadata: &Metadata) -> Option<PathBuf> {
        if metadata.is_file() {
            return Some(self.path);
        }

        let index_path = self.path.join("index.html");

        match fs::metadata(&index_path) {
            Ok(m) =>
                if m.is_file() {
                    Some(index_path)
                } else {
                    None
                },
            Err(_) => None,
        }
    }
}
