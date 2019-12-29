use core::errors::ServiceError;
use core::models::{User, Contact};

pub type NotifyService = NotificationWrapper;

pub mod firebase;
pub mod testing;

type FirebaseToken = String;

pub struct NotificationWrapper {
    pub service: Box<dyn NotificationService>,
}

impl NotificationWrapper {
    pub fn new(a: Box<dyn NotificationService>) -> Self {
        Self {
            service: a
        }
    }
}

pub trait NotificationService : Send + Sync {
    fn push(&self, _: Vec<(Contact, FirebaseToken)>) -> Result<(), ServiceError>;
}
