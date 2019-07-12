use crate::errors::ServiceError;
use phonenumber::Mode;

pub fn phonenumber_to_international(number: &str, country: &str) -> String {
    let number = phonenumber::parse(Some(country.parse().unwrap()), number).unwrap();
    format!("{}", number.format().mode(Mode::International)).replace(" ", "")
}
