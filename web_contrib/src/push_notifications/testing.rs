use super::*;
use core::models::{User, Contact};
use core::errors::ServiceError;

pub struct TestingNotificationService;

impl NotificationService for TestingNotificationService {
    fn push(&self, _: Vec<(String, FirebaseToken)>) -> Result<(), ServiceError> {
        Ok(())
    }
}
