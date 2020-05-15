use crate::errors::{ServiceError, InvalidUserInput};

pub fn phonenumber_to_international(
    number: &str,
    country: &str,
) -> Result<phonenumber::PhoneNumber, ServiceError> {
    let parsed_country = Some(
        country
            .parse()
            .map_err(|_| InvalidUserInput::InvalidCountry(country.to_string()))?,
    );

    let number = phonenumber::parse(parsed_country, number)
        .map_err(|_| InvalidUserInput::InvalidPhoneNumber(number.to_string()))?;

    Ok(number)
}
