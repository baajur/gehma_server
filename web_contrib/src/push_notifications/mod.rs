use core::errors::ServiceError;

pub type NotifyService = NotificationWrapper;

pub mod firebase;
pub mod testing;

type Name = String;
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
    fn push(&self, _: Vec<(Name, FirebaseToken)>) -> Result<(), ServiceError>;
}
