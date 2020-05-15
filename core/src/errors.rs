use actix_web::{error::ResponseError, HttpResponse};
use derive_more::Display;
use diesel::result::{DatabaseErrorKind, Error as DBError};
use log::error;
use serde::Serialize;
use std::convert::From;
use uuid::ParseError;

#[derive(Debug, Display)]
pub enum ServiceError {
    #[display(fmt = "Internal Server Error")]
    InternalError,

    #[display(fmt = "Internal Server Error")]
    InternalServerError(InternalServerError),

    #[display(fmt = "BadRequest: {}", _0)]
    BadRequest(String),

    #[display(fmt = "Wrong parameters ({})", _0)]
    InvalidUserInput(InvalidUserInput),

    #[display(fmt = "Entity already exists")]
    AlreadyExists,

    #[display(fmt = "Unauthorized")]
    Unauthorized,

    #[display(fmt = "RateLimit was reached")]
    RateLimit,
}

#[derive(Debug, Display, Serialize)]
pub enum InternalServerError {
    #[display(fmt = "Cannot generate profile")]
    GenerateImageError,
    #[display(fmt = "Database failed {}", _0)]
    DatabaseError(String),
    #[display(fmt = "Notification sending failed")]
    NotificationError,
    #[display(fmt = "Input/Output error {}", _0)]
    IOError(String)
}

#[derive(Debug, Display, Serialize)]
pub enum InvalidUserInput {
    #[display(fmt = "Invalid Phone Number: {}", _0)]
    InvalidPhoneNumber(String),
    #[display(fmt = "Invalid Country: {}", _0)]
    InvalidCountry(String),
    #[display(fmt = "Invalid code")]
    InvalidCode,
}

impl ResponseError for ServiceError {
    fn error_response(&self) -> HttpResponse {
        match self {
            ServiceError::InternalError => {
                error!("InternalError");
                HttpResponse::InternalServerError().json("Internal Server Error")
            }
            ServiceError::InternalServerError(err) => {
                error!("{}", err);
                HttpResponse::InternalServerError().json("Internal Server Error")
            }
            ServiceError::InvalidUserInput(err) => {
                error!("{}", err);
                HttpResponse::BadRequest().json(err)
            }
            ServiceError::BadRequest(ref message) => HttpResponse::BadRequest().json(message),
            ServiceError::AlreadyExists => HttpResponse::BadRequest().json("Entity already exists"),
            ServiceError::Unauthorized => HttpResponse::Unauthorized().json("Unauthorized"),
            ServiceError::RateLimit => HttpResponse::TooManyRequests().json("Too many requests"),
        }
    }
}

impl From<ParseError> for ServiceError {
    fn from(_: ParseError) -> ServiceError {
        ServiceError::BadRequest("Invalid UUID".into())
    }
}

impl From<InvalidUserInput> for ServiceError {
    fn from(err: InvalidUserInput) -> ServiceError {
        ServiceError::InvalidUserInput(err)
    }
}

impl From<InternalServerError> for ServiceError {
    fn from(err: InternalServerError) -> ServiceError {
        ServiceError::InternalServerError(err)
    }
}

impl From<DBError> for ServiceError {
    fn from(error: DBError) -> ServiceError {
        match error {
            DBError::DatabaseError(kind, info) => {
                if let DatabaseErrorKind::UniqueViolation = kind {
                    let message = info.details().unwrap_or_else(|| info.message()).to_string();
                    error!("{}", message);
                    return ServiceError::AlreadyExists;
                }
                error!("kind {:?} info {:?}", kind, info);
                ServiceError::InternalError
            }
            other => {
                error!("{}", other);
                ServiceError::InternalError
            }
        }
    }
}
