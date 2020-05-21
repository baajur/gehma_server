use core::errors::ServiceError;

pub mod firebase;
pub mod one_signal;
pub mod testing;

use mockall::*;

type Name = String;
type Token = String;

pub type NotificationService = Box<dyn NotificationServiceTrait>;

#[automock]
pub trait NotificationServiceTrait {
    fn push(&self, contacts: Vec<(Name, Token)>) -> Result<(), ServiceError>;
}
