use core::errors::ServiceError;

pub mod firebase;
pub mod testing;

type Name = String;
type FirebaseToken = String;

pub type NotificationService = Box<dyn NotificationServiceTrait>;

pub trait NotificationServiceTrait: Send + Sync {
    fn push(&self, _: Vec<(Name, FirebaseToken)>) -> Result<(), ServiceError>;
}
