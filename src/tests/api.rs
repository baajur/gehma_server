use super::*;
use ::core::models::*;
use actix_web::dev::Service;
use actix_web::{test, web, App};

use std::fs::File;
use std::io::{Write, Read, Seek, SeekFrom};

fn read_id() -> uuid::Uuid {
    let mut tmpfile: File = tempfile::tempfile().unwrap();
    let mut buf = String::new();
    tmpfile.read_to_string(&mut buf).unwrap();

    uuid::Uuid::parse_str(&buf).unwrap()
}

fn write_id(id: uuid::Uuid) {
    let mut tmpfile: File = tempfile::tempfile().unwrap();
    write!(tmpfile, "{}", id).unwrap();
}

#[test]
fn test_post_user() {
    //use crate::exists_handler::ResponseUser;
    let mut app = test::init_service(App::new().route("/api/user", web::post().to_async(user_handler::add)));
    let req = test::TestRequest::post().uri("/api/user").to_request();
        
    let resp: User = test::read_response_json(&mut app, req);
}
