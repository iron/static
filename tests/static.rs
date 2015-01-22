extern crate hyper;
extern crate iron;
extern crate "iron-test" as iron_test;
extern crate "static" as static_file;

use hyper::header::Location;
use iron::method::Method::Get;
use iron::{Url, Handler};
use iron::status::Status;
use iron_test::{mock, ProjectBuilder};
use static_file::Static;

#[test]
fn serves_non_default_file_from_absolute_root_path() {
    let p = ProjectBuilder::new("example").file("file1.html", "this is file1");
    p.build();
    let st = Static::new(p.root());
    let mut req = mock::request::at(Get, Url::parse("http://localhost:3000/file1.html").unwrap());
    match st.call(&mut req) {
        Ok(res) => assert_eq!(res.body.unwrap().read_to_string().unwrap(), "this is file1".to_string()),
        Err(e) => panic!("{:?}", e)
    }
}

#[test]
fn serves_default_file_from_absolute_root_path() {
    let p = ProjectBuilder::new("example").file("index.html", "this is index");
    p.build();
    let st = Static::new(p.root());
    let mut req = mock::request::at(Get, Url::parse("http://localhost:3000").unwrap());
    match st.call(&mut req) {
        Ok(res) => assert_eq!(res.body.unwrap().read_to_string().unwrap(), "this is index".to_string()),
        Err(e) => panic!("{:?}", e)
    }
}

#[test]
fn returns_404_if_file_not_found() {
    let p = ProjectBuilder::new("example");
    p.build();
    let st = Static::new(p.root());
    let mut req = mock::request::at(Get, Url::parse("http://localhost:3000").unwrap());

    match st.call(&mut req) {
        Ok(res) => {
            assert_eq!(res.status.unwrap(), Status::NotFound);
        },
        Err(e) => panic!("{:?}", e),
    }
}

#[test]
fn redirects_if_trailing_slash_is_missing() {
    let p = ProjectBuilder::new("example").file("dir/index.html", "this is index");
    p.build();

    let st = Static::new(p.root());

    let mut req = mock::request::at(Get, Url::parse("http://localhost:3000/dir").unwrap());

    match st.call(&mut req) {
        Ok(res) => {
            assert_eq!(res.status.unwrap(), Status::MovedPermanently);
            assert_eq!(res.headers.get::<Location>().unwrap(),
                       &Location("http://localhost:3000/dir/".to_string()));
        },
        Err(e) => panic!("{:?}", e)
    }
}
