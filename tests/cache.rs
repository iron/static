#![feature(duration)]

extern crate time;

extern crate hyper;
extern crate iron;
extern crate iron_test;
extern crate staticfile;

use time::Timespec;

use iron::{Handler, Url};
use iron::method::Method::Get;
use iron::status::Status;
use iron::headers::HttpDate;
use hyper::header::{IfModifiedSince, CacheControl, CacheDirective, LastModified};
use hyper::buffer::BufReader;
use hyper::net::NetworkStream;
use iron_test::{mock, ProjectBuilder};
use iron_test::mock::MockStream;
use staticfile::Static;
use std::io::Cursor;
use std::time::Duration;

#[test]
fn it_should_return_cache_headers() {
    let p = ProjectBuilder::new("example").file("file1.html", "this is file1");
    p.build();

    let st = Static::new(p.root().clone()).cache(Duration::from_secs(30*24*60*60));
    let mut stream = MockStream::new(Cursor::new("".to_string().into_bytes()));
    let mut reader = BufReader::new(&mut stream as &mut NetworkStream);
    let mut req = mock::request::new(Get,
                                     Url::parse("http://localhost:3000/file1.html").unwrap(),
                                     &mut reader);
    match st.handle(&mut req) {
        Ok(res) => {
            assert!(res.headers.get::<CacheControl>().is_some());
            assert!(res.headers.get::<LastModified>().is_some());
            let cache = res.headers.get::<CacheControl>().unwrap();
            let directives = vec![CacheDirective::Public, CacheDirective::MaxAge(2592000)];
            assert_eq!(*cache, CacheControl(directives));
        },
        Err(e) => panic!("{}", e)
    }
}

#[test]
fn it_should_return_the_file_if_client_sends_no_modified_time() {
    let p = ProjectBuilder::new("example").file("file1.html", "this is file1");
    p.build();

    let st = Static::new(p.root().clone()).cache(Duration::from_secs(30*24*60*60));
    let mut stream = MockStream::new(Cursor::new("".to_string().into_bytes()));
    let mut reader = BufReader::new(&mut stream as &mut NetworkStream);
    let mut req = mock::request::new(Get,
                                     Url::parse("http://localhost:3000/file1.html").unwrap(),
                                     &mut reader);
    match st.handle(&mut req) {
        Ok(res) => assert_eq!(res.status.unwrap(), Status::Ok),
        Err(e) => panic!("{}", e)
    }
}

#[test]
fn it_should_return_the_file_if_client_has_old_version() {
    let p = ProjectBuilder::new("example").file("file1.html", "this is file1");
    p.build();

    let st = Static::new(p.root().clone()).cache(Duration::from_secs(30*24*60*60));
    let mut stream = MockStream::new(Cursor::new("".to_string().into_bytes()));
    let mut reader = BufReader::new(&mut stream as &mut NetworkStream);
    let mut req = mock::request::new(Get,
                                     Url::parse("http://localhost:3000/file1.html").unwrap(),
                                     &mut reader);

    let now = time::get_time();
    let one_hour_ago = Timespec::new(now.sec - 3600, now.nsec);
    req.headers.set(IfModifiedSince(HttpDate(time::at(one_hour_ago))));
    match st.handle(&mut req) {
        Ok(res) => assert_eq!(res.status.unwrap(), Status::Ok),
        Err(e) => panic!("{}", e)
    }
}

#[test]
fn it_should_return_304_if_client_has_file_cached() {
    let p = ProjectBuilder::new("example").file("file1.html", "this is file1");
    p.build();

    let st = Static::new(p.root().clone()).cache(Duration::from_secs(30*24*60*60));
    let mut stream = MockStream::new(Cursor::new("".to_string().into_bytes()));
    let mut reader = BufReader::new(&mut stream as &mut NetworkStream);
    let mut req = mock::request::new(Get,
                                     Url::parse("http://localhost:3000/file1.html").unwrap(),
                                     &mut reader);
    req.headers.set(IfModifiedSince(HttpDate(time::now_utc())));
    match st.handle(&mut req) {
        Ok(res) => assert_eq!(res.status.unwrap(), Status::NotModified),
        Err(e) => panic!("{}", e)
    }
}

#[test]
fn it_should_cache_index_html_for_directory_path() {
    let p = ProjectBuilder::new("example").file("dir/index.html", "this is index");
    p.build();

    let st = Static::new(p.root().clone()).cache(Duration::from_secs(30*24*60*60));
    let mut stream = MockStream::new(Cursor::new("".to_string().into_bytes()));
    let mut reader = BufReader::new(&mut stream as &mut NetworkStream);
    let mut req = mock::request::new(Get,
                                     Url::parse("http://localhost:3000/dir/").unwrap(),
                                     &mut reader);
    req.headers.set(IfModifiedSince(HttpDate(time::now_utc())));
    match st.handle(&mut req) {
        Ok(res) => assert_eq!(res.status.unwrap(), Status::NotModified),
        Err(e) => panic!("{}", e)
    }
}

#[test]
fn it_should_defer_to_static_handler_if_directory_misses_trailing_slash() {
    let p = ProjectBuilder::new("example").file("dir/index.html", "this is index");
    p.build();

    let st = Static::new(p.root().clone()).cache(Duration::from_secs(30*24*60*60));
    let mut stream = MockStream::new(Cursor::new("".to_string().into_bytes()));
    let mut reader = BufReader::new(&mut stream as &mut NetworkStream);
    let mut req = mock::request::new(Get,
                                     Url::parse("http://localhost:3000/dir").unwrap(),
                                     &mut reader);
    req.headers.set(IfModifiedSince(HttpDate(time::now_utc())));
    match st.handle(&mut req) {
        Ok(res) => {
            assert_eq!(res.status.unwrap(), Status::MovedPermanently);
            assert!(res.headers.get::<LastModified>().is_none());
        },
        Err(e) => panic!("{}", e)
    }
}
