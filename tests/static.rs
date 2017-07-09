extern crate hyper;
extern crate iron;
extern crate iron_test;
extern crate staticfile;

use iron::headers::{ByteRangeSpec, Headers, Location, Range};
use iron::status::Status;

use iron_test::{request, ProjectBuilder};

use staticfile::Static;

use std::str;

#[test]
fn serves_non_default_file_from_absolute_root_path() {
    let p = ProjectBuilder::new("example").file("file1.html", "this is file1");
    p.build();
    let st = Static::new(p.root().clone());
    match request::get("http://localhost:3000/file1.html", Headers::new(), &st) {
        Ok(res) => {
            let mut body = Vec::new();
            res.body.unwrap().write_body(&mut body).unwrap();
            assert_eq!(str::from_utf8(&body).unwrap(), "this is file1");
        },
        Err(e) => panic!("{}", e)
    }
}

#[test]
fn serves_default_file_from_absolute_root_path() {
    let p = ProjectBuilder::new("example").file("index.html", "this is index");
    p.build();
    let st = Static::new(p.root().clone());
    match request::get("http://localhost:3000/index.html", Headers::new(), &st) {
        Ok(res) => {
            let mut body = Vec::new();
            res.body.unwrap().write_body(&mut body).unwrap();
            assert_eq!(str::from_utf8(&body).unwrap(), "this is index");
        },
        Err(e) => panic!("{}", e)
    }
}

#[test]
fn returns_404_if_file_not_found() {
    let p = ProjectBuilder::new("example");
    p.build();
    let st = Static::new(p.root().clone());
    match request::get("http://localhost:3000", Headers::new(), &st) {
        Ok(res) => panic!("Expected IronError, got Response: {}", res),
        Err(e) => assert_eq!(e.response.status.unwrap(), Status::NotFound)
    }
}

#[test]
fn redirects_if_trailing_slash_is_missing() {
    let p = ProjectBuilder::new("example").file("dir/index.html", "this is index");
    p.build();

    let st = Static::new(p.root().clone());
    match request::get("http://localhost:3000/dir", Headers::new(), &st) {
        Ok(res) => {
            assert_eq!(res.status.unwrap(), Status::MovedPermanently);
            assert_eq!(res.headers.get::<Location>().unwrap(),
                       &Location("http://localhost:3000/dir/".to_string()));
        },
        Err(e) => panic!("{}", e)
    }
}

#[test]
fn decodes_percent_notation() {
    let p = ProjectBuilder::new("example").file("has space.html", "file with funky chars");
    p.build();
    let st = Static::new(p.root().clone());
    match request::get("http://localhost:3000/has space.html", Headers::new(), &st) {
        Ok(res) => {
            let mut body = Vec::new();
            res.body.unwrap().write_body(&mut body).unwrap();
            assert_eq!(str::from_utf8(&body).unwrap(), "file with funky chars");
        },
        Err(e) => panic!("{}", e)
    }
}

#[test]
fn normalizes_path() {
    let p = ProjectBuilder::new("example").file("index.html", "this is index");
    p.build();
    let st = Static::new(p.root().clone());
    match request::get("http://localhost:3000/xxx/../index.html", Headers::new(), &st) {
        Ok(res) => {
            let mut body = Vec::new();
            res.body.unwrap().write_body(&mut body).unwrap();
            assert_eq!(str::from_utf8(&body).unwrap(), "this is index");
        },
        Err(e) => panic!("{}", e)
    }
}

#[test]
fn normalizes_percent_encoded_path() {
    let p = ProjectBuilder::new("example").file("file1.html", "this is file1");
    p.build();
    let st = Static::new(p.root().clone());
    match request::get("http://localhost:3000/xxx/..%2ffile1.html", Headers::new(), &st) {
        Ok(res) => {
            let mut body = Vec::new();
            res.body.unwrap().write_body(&mut body).unwrap();
            assert_eq!(str::from_utf8(&body).unwrap(), "this is file1");
        },
        Err(e) => panic!("{}", e)
    }
}

#[test]
fn prevents_from_escaping_root() {
    let p = ProjectBuilder::new("example").file("file1.html", "this is file1");
    p.build();
    let st = Static::new(p.root().clone());

    match request::get("http://localhost:3000/../file1.html", Headers::new(), &st) {
        Ok(res) => {
            let mut body = Vec::new();
            res.body.unwrap().write_body(&mut body).unwrap();
            assert_eq!(str::from_utf8(&body).unwrap(), "this is file1");
        },
        Err(e) => panic!("{}", e)
    }

    match request::get("http://localhost:3000/..%2ffile1.html", Headers::new(), &st) {
        Ok(res) => {
            let mut body = Vec::new();
            res.body.unwrap().write_body(&mut body).unwrap();
            assert_eq!(str::from_utf8(&body).unwrap(), "this is file1");
        },
        Err(e) => panic!("{}", e)
    }

    match request::get("http://localhost:3000/xxx/..%2f..%2ffile1.html", Headers::new(), &st) {
        Ok(res) => {
            let mut body = Vec::new();
            res.body.unwrap().write_body(&mut body).unwrap();
            assert_eq!(str::from_utf8(&body).unwrap(), "this is file1");
        },
        Err(e) => panic!("{}", e)
    }    
}

#[test]
fn serves_partial_content_from_to() {
    let p = ProjectBuilder::new("example").file("file1.html", "0123456789");
    p.build();
    let st = Static::new(p.root().clone());

    // FromTo
    let mut headers = Headers::new();
    headers.set(Range::Bytes(vec![ByteRangeSpec::FromTo(2, 7)]));
    let res = request::get("http://localhost:3000/file1.html", headers, &st).unwrap();
    assert_eq!(res.status, Some(Status::PartialContent));
    let mut body = Vec::new();
    res.body.unwrap().write_body(&mut body).unwrap();
    assert_eq!(str::from_utf8(&body).unwrap(), "234567");

    // Implicit end of range
    let mut headers = Headers::new();
    headers.set(Range::Bytes(vec![ByteRangeSpec::FromTo(5, 100)]));
    let res = request::get("http://localhost:3000/file1.html", headers, &st).unwrap();
    assert_eq!(res.status, Some(Status::PartialContent));
    let mut body = Vec::new();
    res.body.unwrap().write_body(&mut body).unwrap();
    assert_eq!(str::from_utf8(&body).unwrap(), "56789");

    // Range out of bounds
    let mut headers = Headers::new();
    headers.set(Range::Bytes(vec![ByteRangeSpec::FromTo(11, 12)]));
    let res = request::get("http://localhost:3000/file1.html", headers, &st).unwrap();
    assert_eq!(res.status, Some(Status::RangeNotSatisfiable));

    // Backwards range
    let mut headers = Headers::new();
    headers.set(Range::Bytes(vec![ByteRangeSpec::FromTo(8, 5)]));
    let res = request::get("http://localhost:3000/file1.html", headers, &st).unwrap();
    assert_eq!(res.status, Some(Status::Ok));
    let mut body = Vec::new();
    res.body.unwrap().write_body(&mut body).unwrap();
    assert_eq!(str::from_utf8(&body).unwrap(), "0123456789");
}

#[test]
fn serves_partial_content_last() {
    let p = ProjectBuilder::new("example").file("file1.html", "0123456789");
    p.build();
    let st = Static::new(p.root().clone());

    let mut headers = Headers::new();
    headers.set(Range::Bytes(vec![ByteRangeSpec::Last(3)]));
    let res = request::get("http://localhost:3000/file1.html", headers, &st).unwrap();
    assert_eq!(res.status, Some(Status::PartialContent));
    
    let mut body = Vec::new();
    res.body.unwrap().write_body(&mut body).unwrap();
    assert_eq!(str::from_utf8(&body).unwrap(), "789");
}

#[test]
fn serves_partial_content_all_from() {
    let p = ProjectBuilder::new("example").file("file1.html", "0123456789");
    p.build();
    let st = Static::new(p.root().clone());

    let mut headers = Headers::new();
    headers.set(Range::Bytes(vec![ByteRangeSpec::AllFrom(5)]));
    let res = request::get("http://localhost:3000/file1.html", headers, &st).unwrap();
    assert_eq!(res.status, Some(Status::PartialContent));
    let mut body = Vec::new();
    res.body.unwrap().write_body(&mut body).unwrap();
    assert_eq!(str::from_utf8(&body).unwrap(), "56789");

    // Range out of bounds
    let mut headers = Headers::new();
    headers.set(Range::Bytes(vec![ByteRangeSpec::AllFrom(11)]));
    let res = request::get("http://localhost:3000/file1.html", headers, &st).unwrap();
    assert_eq!(res.status, Some(Status::RangeNotSatisfiable));
}
