use core::errors::ServiceError;
use std::sync::{Arc, Mutex};

pub type NotifyService = NotificationWrapper;

pub mod firebase;
pub mod testing;

type Name = String;
type FirebaseToken = String;

#[derive(Clone)]
pub struct NotificationWrapper {
    pub service: Arc<Mutex<Box<dyn NotificationService>>>,
}

impl NotificationWrapper {
    pub fn new(a: Box<dyn NotificationService>) -> Self {
        Self {
            service: Arc::new(Mutex::new(a)),
        }
    }
}

pub trait NotificationService: Send + Sync {
    fn push(&self, _: Vec<(Name, FirebaseToken)>) -> Result<(), ServiceError>;
}
