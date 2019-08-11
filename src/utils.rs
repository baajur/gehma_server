pub fn set_response_headers(response: &mut actix_web::HttpResponse) {
    use actix_web::http::header::{*};

    response.headers_mut()
          .insert(STRICT_TRANSPORT_SECURITY, HeaderValue::from_static("max-age=31536000; includeSubDomains"));

    response.headers_mut()
        .insert(CONTENT_SECURITY_POLICY, HeaderValue::from_static("script-src 'self'"));

    response.headers_mut()
        .insert(X_FRAME_OPTIONS, HeaderValue::from_static("SAMEORIGIN"));

    response.headers_mut()
        .insert(X_CONTENT_TYPE_OPTIONS, HeaderValue::from_static("nosniff"));

    response.headers_mut()
        .insert(REFERRER_POLICY, HeaderValue::from_static("strict-origin-when-cross-origin"));
}
