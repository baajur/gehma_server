use super::*;
use actix_web::dev::Service;
use actix_web::{test, web, App};

#[test]
fn test_index_ok() {
    let mut app = test::init_service(App::new().route("/", web::get().to(load_index_file)));
    let req = test::TestRequest::get().uri("/").to_request();
    let resp = test::block_on(app.call(req)).unwrap();

    assert!(resp.status().is_success());
}

#[test]
fn test_load_file() {
    let mut app =
        test::init_service(App::new().route("/static/{filename:.*}", web::get().to(load_file)));
    let req = test::TestRequest::get()
        .uri("/static/Android.jpg")
        .to_request();
    let resp = test::block_on(app.call(req)).unwrap();

    assert!(resp.status().is_success());
}

#[test]
fn test_load_datenschutz_HTML() {
    let mut app =
        test::init_service(App::new().route("/static/{filename:.*}", web::get().to(load_file)));
    let req = test::TestRequest::get()
        .uri("/static/datenschutz.html")
        .to_request();
    let resp = test::block_on(app.call(req)).unwrap();

    assert!(resp.status().is_success());
}

#[test]
fn test_load_datenschutz_pdf() {
    let mut app =
        test::init_service(App::new().route("/static/{filename:.*}", web::get().to(load_file)));
    let req = test::TestRequest::get()
        .uri("/static/datenschutz.pdf")
        .to_request();
    let resp = test::block_on(app.call(req)).unwrap();

    assert!(resp.status().is_success());
}
