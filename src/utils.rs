use crate::errors::{InternalError};

pub fn phonenumber_to_international(number: &str, country: &str) -> Result<phonenumber::PhoneNumber, InternalError> {

    let parsed_country = Some(country.parse().map_err(|_| InternalError::InvalidCountry(country.to_string()))?);

    let number = phonenumber::parse(parsed_country, number).map_err(|_| InternalError::InvalidPhoneNumber(number.to_string()))?;

    Ok(number)
    //Ok(format!("{}", number.format().mode(Mode::International)).replace(" ", ""))
}

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
