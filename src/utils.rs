use crate::errors::{ServiceError, InternalError};
use phonenumber::Mode;

pub fn phonenumber_to_international(number: &str, country: &str) -> Result<String, InternalError> {

    let parsed_country = Some(country.parse().map_err(|_| InternalError::InvalidCountry(country.to_string()))?);

    let number = phonenumber::parse(parsed_country, number).map_err(|_| InternalError::InvalidPhoneNumber(number.to_string()))?;
    Ok(format!("{}", number.format().mode(Mode::International)).replace(" ", ""))
}
