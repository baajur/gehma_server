use super::*;
use actix_web::{test, web, App};

#[test]
fn test_index_get() {
    let mut app = test::init_service(
        App::new()
            .route("/", web::get().to(index)),
    );
    let req = test::TestRequest::get().uri("/").to_request();
    let resp: AppState = test::read_response_json(&mut app, req);

    assert!(resp.count == 4);
}
