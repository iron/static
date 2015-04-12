#![feature(io)]

extern crate hyper;
extern crate iron;
extern crate "iron-test" as iron_test;
extern crate staticfile;

use hyper::header::Location;
use iron::method::Method::Get;
use iron::{Url, Handler};
use iron::status::Status;
use iron_test::{mock, ProjectBuilder};
use staticfile::Static;
use std::io;

#[test]
fn serves_non_default_file_from_absolute_root_path() {
    let p = ProjectBuilder::new("example").file("file1.html", "this is file1");
    p.build();
    let st = Static::new(p.root().clone());
    let mut reader = io::empty();
    let mut req = mock::request::new(Get,
                                     Url::parse("http://localhost:3000/file1.html").unwrap(),
                                     &mut reader);
    match st.handle(&mut req) {
        Ok(res) => {
            let mut str = String::new();
            res.body.unwrap().read_to_string(&mut str).unwrap();
            assert_eq!(str, "this is file1".to_string())
        },
        Err(e) => panic!("{}", e)
    }
}

#[test]
fn serves_default_file_from_absolute_root_path() {
    let p = ProjectBuilder::new("example").file("index.html", "this is index");
    p.build();
    let st = Static::new(p.root().clone());
    let mut reader = io::empty();
    let mut req = mock::request::new(Get,
                                     Url::parse("http://localhost:3000").unwrap(),
                                     &mut reader);
    match st.handle(&mut req) {
        Ok(res) => {
            let mut str = String::new();
            res.body.unwrap().read_to_string(&mut str).unwrap();
            assert_eq!(str, "this is index".to_string())
        },
        Err(e) => panic!("{}", e)
    }
}

#[test]
fn returns_404_if_file_not_found() {
    let p = ProjectBuilder::new("example");
    p.build();
    let st = Static::new(p.root().clone());
    let mut reader = io::empty();
    let mut req = mock::request::new(Get,
                                     Url::parse("http://localhost:3000").unwrap(),
                                     &mut reader);

    match st.handle(&mut req) {
        Ok(res) => panic!("Expected IronError, got Response: {}", res),
        Err(e) => assert_eq!(e.response.status.unwrap(), Status::NotFound)
    }
}

#[test]
fn redirects_if_trailing_slash_is_missing() {
    let p = ProjectBuilder::new("example").file("dir/index.html", "this is index");
    p.build();

    let st = Static::new(p.root().clone());
    let mut reader = io::empty();
    let mut req = mock::request::new(Get,
                                     Url::parse("http://localhost:3000/dir").unwrap(),
                                     &mut reader);

    match st.handle(&mut req) {
        Ok(res) => {
            assert_eq!(res.status.unwrap(), Status::MovedPermanently);
            assert_eq!(res.headers.get::<Location>().unwrap(),
                       &Location("http://localhost:3000/dir/".to_string()));
        },
        Err(e) => panic!("{}", e)
    }
}
