use core::errors::ServiceError;

pub mod firebase;
pub mod testing;

use mockall::*;

type Name = String;
type FirebaseToken = String;

pub type NotificationService = Box<dyn NotificationServiceTrait>;

#[automock]
pub trait NotificationServiceTrait {
    fn push(&self, contacts: Vec<(Name, FirebaseToken)>) -> Result<(), ServiceError>;
}
