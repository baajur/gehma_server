use actix_web::{error::ResponseError, HttpResponse};
use derive_more::Display;
use diesel::result::{DatabaseErrorKind, Error as DBError};
use std::convert::From;
use uuid::ParseError;

#[derive(Debug, Display)]
pub enum ServiceError {
    #[display(fmt = "Internal Server Error")]
    InternalServerError,

    #[display(fmt = "Internal Server Error: {}", _0)]
    InternalServerError2(String),

    #[display(fmt = "BadRequest: {}", _0)]
    BadRequest(String),

    #[display(fmt = "Already exists: {}", _0)]
    AlreadyExists(String),

    #[display(fmt = "Unauthorized")]
    Unauthorized,

    #[display(fmt = "NotificationError: {}", _0)]
    NotificationError(String),

    #[display(fmt = "SchaumaError: {}", _0)]
    SchaumaError(SchaumaError),

    #[display(fmt = "RateLimit was reached ({})", _0)]
    RateLimit(String),
}

#[derive(Debug, Display)]
pub enum InternalError {
    #[display(fmt = "Invalid Phone Number: {}", _0)]
    InvalidPhoneNumber(String),
    #[display(fmt = "Invalid Country: {}", _0)]
    InvalidCountry(String),
    #[display(fmt = "Cannot generate profile: {}", _0)]
    GenerateImage(std::io::Error),
}

#[derive(Debug, Display)]
pub enum SchaumaError {
    #[display(fmt = "Cannot parse: {}", _0)]
    ParseError(String),
    #[display(fmt = "Datasource error occured: {}", _0)]
    DatasourceError(String),
}

impl ResponseError for ServiceError {
    fn error_response(&self) -> HttpResponse {
        match self {
            ServiceError::InternalServerError => {
                HttpResponse::InternalServerError().json("Internal Server Error")
            }
            ServiceError::InternalServerError2(ref message) => {
                HttpResponse::InternalServerError().json(message)
            }
            ServiceError::BadRequest(ref message) => HttpResponse::BadRequest().json(message),
            ServiceError::AlreadyExists(ref message) => HttpResponse::BadRequest().json(message),
            ServiceError::Unauthorized => HttpResponse::Unauthorized().json("Unauthorized"),
            ServiceError::NotificationError(ref message) => {
                HttpResponse::BadRequest().json(message)
            }
            ServiceError::SchaumaError(ref message) => message.error_response(),
            ServiceError::RateLimit(ref message) => HttpResponse::TooManyRequests().json(message),
        }
    }
}

impl ResponseError for SchaumaError {
    fn error_response(&self) -> HttpResponse {
        match self {
            SchaumaError::ParseError(ref message) => HttpResponse::BadRequest().json(message),
            SchaumaError::DatasourceError(ref message) => {
                HttpResponse::InternalServerError().json(message)
            }
        }
    }
}

impl From<ParseError> for ServiceError {
    fn from(_: ParseError) -> ServiceError {
        ServiceError::BadRequest("Invalid UUID".into())
    }
}

impl From<InternalError> for ServiceError {
    fn from(err: InternalError) -> ServiceError {
        ServiceError::BadRequest(format!("{}", err))
    }
}

impl From<DBError> for ServiceError {
    fn from(error: DBError) -> ServiceError {
        match error {
            DBError::DatabaseError(kind, info) => {
                if let DatabaseErrorKind::UniqueViolation = kind {
                    let message = info.details().unwrap_or_else(|| info.message()).to_string();
                    return ServiceError::AlreadyExists(message);
                }
                ServiceError::InternalServerError
            }
            _ => ServiceError::InternalServerError,
        }
    }
}
